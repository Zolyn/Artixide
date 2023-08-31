use std::{fmt::Debug, fs, iter::once, str::FromStr};

use color_eyre::{eyre::ContextCompat, Result};
use gptman::GPT;
use indexmap::IndexMap;

use super::{
    format_size, itoa, BlockDevice, ChildBlockDevice, CompatDevice, Device, Disk, DiskSpace,
    FileSystem, MemPartition, MemTableEntry, NumberPool, RawDisk, RawSpace, Unit, BOOT_FLAG,
    DEFAULT_ALIGN, ESP_GUID,
};

impl MemPartition {
    fn from_raw(dev: ChildBlockDevice, sector_size: u16, is_gpt: bool) -> Option<MemPartition> {
        if dev.devtype != "part" {
            return None;
        }

        let bootable = if is_gpt {
            dev.parttype == ESP_GUID
        } else {
            dev.partflags.filter(|f| *f == BOOT_FLAG).is_some()
        };

        let size = dev.size;
        let sectors = size / sector_size as u64;
        let number = dev.partn;
        let start = dev.start;
        let end = start + sectors - 1;

        Some(MemPartition {
            number_string: itoa(number),
            start_string: itoa(start),
            end_string: itoa(end),
            sectors_string: itoa(sectors),
            size_string: format_size(size),
            filesystem: FileSystem::from_str(dev.fstype.unwrap_or_default()).unwrap_or_default(),
            label: dev.label.map(|v| v.to_string()),
            uuid: Some(dev.partuuid.to_string()),
            mountpoint: None,
            number,
            start,
            end,
            sectors,
            size,
            bootable,
        })
    }

    pub fn number(&self) -> u16 {
        self.number
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    pub fn sectors(&self) -> u64 {
        self.sectors
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn start_string_mut(&mut self) -> &mut String {
        &mut self.start_string
    }

    pub fn end_string_mut(&mut self) -> &mut String {
        &mut self.end_string
    }

    pub fn size_string_mut(&mut self) -> &mut String {
        &mut self.size_string
    }

    pub fn sectors_string_mut(&mut self) -> &mut String {
        &mut self.sectors_string
    }

    pub fn as_raw_space(&self) -> RawSpace {
        RawSpace {
            start: self.start,
            end: self.end,
            sectors: self.sectors,
            size: self.size,
        }
    }
}

impl Device {
    pub fn new_from(dev: BlockDevice) -> Result<Option<Device>> {
        if dev.devtype != "disk" {
            return Ok(None);
        }

        let model = dev.model.unwrap().trim().to_string();
        let path = dev.path.to_string();
        let size_string = format_size(dev.size);
        let size = dev.size;
        let sector_size = dev.log_sec;

        let table = dev.pttype.filter(|table| matches!(*table, "gpt" | "dos"));

        if table.is_none() {
            return Ok(Some(Device::Incompatible(RawDisk {
                model,
                path,
                size,
                size_string,
                sector_size,
            })));
        }

        let id = dev.ptuuid.unwrap().to_string();
        let is_gpt = table.unwrap() == "gpt";
        let starting_lba: u64;
        let ending_lba: u64;

        if is_gpt {
            let mut dev_file = fs::File::open(dev.path)?;
            let mut gpt = GPT::read_from(&mut dev_file, sector_size as u64)?;
            let header = &mut gpt.header;
            header.update_from(&mut dev_file, sector_size as u64)?;

            starting_lba = header.first_usable_lba - 1;
            ending_lba = header.last_usable_lba + 1;
        } else {
            starting_lba = 0;
            ending_lba = size / sector_size as u64;
        };

        let disk = Disk {
            model,
            path,
            size,
            size_string,
            sector_size,
            starting_lba,
            ending_lba,
            is_gpt,
            id,
        };

        let mut number_pool = NumberPool::new();

        let mem_table = dev
            .children
            .unwrap_or_default()
            .into_iter()
            .filter_map(|c| {
                if c.mountpoint.is_some() {
                    return None;
                }

                let part = MemPartition::from_raw(c, sector_size, is_gpt)?;

                number_pool.set_used(part.number as usize - 1);

                Some(MemTableEntry::Partition(part))
            })
            .collect::<Vec<_>>();

        let modifications = IndexMap::with_capacity(mem_table.len());

        let mut dev = CompatDevice {
            disk,
            mem_table,
            number_pool,
            modifications,
        };

        dev.fill_free_space();

        Ok(Some(Device::Compatible(dev)))
    }
}

impl MemTableEntry {
    pub fn start(&self) -> u64 {
        match self {
            Self::Free(free) => free.start,
            Self::Partition(part) => part.start,
        }
    }
}

impl Disk {
    pub fn parse_sectors_from(&self, val: &str) -> Result<u64> {
        if let Ok(v) = val.parse::<u64>() {
            return Ok(v);
        }

        let number = {
            let offset = val
                .char_indices()
                .take_while(|(_, c)| c.is_ascii_digit())
                .last()
                .wrap_err("No digit found")?
                .0;
            &val[..offset]
        };

        let n = number.parse::<u64>()?;

        let suffix = {
            let offset = val
                .char_indices()
                .skip_while(|(_, c)| c.is_whitespace() || c.is_ascii_digit() || *c == '.')
                .last()
                .wrap_err("No unit found")?
                .0;

            &val[..offset]
        };

        let unit = Unit::from_str(suffix)?;

        let sectors = unit.as_bytes(n) / self.sector_size as u64;

        Ok(sectors)
    }
}

impl CompatDevice {
    /// References
    ///
    /// https://docs.rs/gptman/1.0.1/src/gptman/lib.rs.html#958-976
    ///
    /// https://docs.rs/mbrman/0.5.2/src/mbrman/lib.rs.html#692-733
    // TODO: MBR
    pub fn fill_free_space(&mut self) {
        let disk = &self.disk;
        let entries = &mut self.mem_table;
        let len = entries.len();

        let mut positions = once(disk.starting_lba)
            .chain(
                entries
                    .iter()
                    .filter_map(|entry| match entry {
                        MemTableEntry::Partition(part) => Some(part),
                        _ => None,
                    })
                    .flat_map(|part| [part.start, part.end]),
            )
            .chain(once(disk.ending_lba))
            .collect::<Vec<_>>();

        positions.sort_unstable();

        // The len of positions is always even
        let spaces = positions.chunks(2).filter_map(|chunk| {
            let start = chunk[0];
            let end = chunk[1];
            let sectors = end - start - 1;

            // No sectors between start and end
            if sectors == 0 {
                return None;
            }

            let first_usable = start + 1;
            let padding = (start / DEFAULT_ALIGN + 1) * DEFAULT_ALIGN - first_usable;
            let sectors = sectors.saturating_sub(padding);

            // No sectors between aligned start and end
            if sectors == 0 {
                return None;
            }

            let start = first_usable + padding;
            let end = end - 1;
            let size = sectors * disk.sector_size as u64;

            let space = DiskSpace {
                start,
                end,
                sectors,
                size,
                start_string: itoa(start),
                end_string: itoa(end),
                sectors_string: itoa(sectors),
                size_string: format_size(size),
            };

            Some(MemTableEntry::Free(space))
        });

        entries.retain(|entry| matches!(entry, MemTableEntry::Partition(_)));
        entries.extend(spaces);

        if entries.len() != len {
            entries.sort_unstable_by_key(|e| e.start())
        }
    }
}

impl NumberPool {
    pub fn new() -> Self {
        Self {
            inner: [false; 256].to_vec(),
        }
    }

    pub fn find_available_num(&mut self) -> Option<u16> {
        for (index, is_used) in self.inner.iter_mut().enumerate() {
            if !*is_used {
                *is_used = true;
                return Some(index as u16 + 1);
            }
        }

        None
    }

    pub fn set_used(&mut self, index: usize) {
        self.inner[index] = true
    }

    pub fn set_unused(&mut self, index: usize) {
        self.inner[index] = false
    }
}

impl Debug for NumberPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.inner.iter().enumerate().filter_map(|(i, used)| {
                if *used {
                    Some(i + 1)
                } else {
                    None
                }
            }))
            .finish()
    }
}

impl DiskSpace {
    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    pub fn sectors(&self) -> u64 {
        self.sectors
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn size_string_mut(&mut self) -> &mut String {
        &mut self.size_string
    }

    pub fn expand_right(&mut self, val: RawSpace) {
        assert!(val.start == self.end + 1, "Not a sibling space");

        self.end = val.end;
        self.sectors += val.sectors;
        self.size += val.size;
        self.end_string = itoa(self.end);
        self.sectors_string = itoa(self.sectors);
        self.size_string = format_size(self.size)
    }

    pub fn expand_left(&mut self, val: RawSpace) {
        assert!(val.end == self.start - 1, "Not a sibling space");

        self.start = val.start;
        self.sectors += val.sectors;
        self.size += val.size;
        self.start_string = itoa(self.start);
        self.sectors_string = itoa(self.sectors);
        self.size_string = format_size(self.size)
    }

    pub fn as_raw_space(&self) -> RawSpace {
        RawSpace {
            start: self.start,
            end: self.end,
            sectors: self.sectors,
            size: self.size,
        }
    }
}

// Source: https://github.com/hyunsik/bytesize/blob/8dd9145911c6c82ab23169ab4bd4a5eb96174b70/src/lib.rs
mod units {
    /// byte size for 1 byte
    pub const B: u64 = 1;
    /// bytes size for 1 kilobyte
    pub const KB: u64 = 1_000;
    /// bytes size for 1 megabyte
    pub const MB: u64 = 1_000_000;
    /// bytes size for 1 gigabyte
    pub const GB: u64 = 1_000_000_000;
    /// bytes size for 1 terabyte
    pub const TB: u64 = 1_000_000_000_000;

    /// bytes size for 1 kibibyte
    pub const KIB: u64 = 1_024;
    /// bytes size for 1 mebibyte
    pub const MIB: u64 = 1_048_576;
    /// bytes size for 1 gibibyte
    pub const GIB: u64 = 1_073_741_824;
    /// bytes size for 1 tebibyte
    pub const TIB: u64 = 1_099_511_627_776;
}

impl Unit {
    fn base(&self) -> u64 {
        use units::*;

        match self {
            Self::B => B,
            Self::KB => KB,
            Self::KiB => KIB,
            Self::MB => MB,
            Self::MiB => MIB,
            Self::GB => GB,
            Self::GiB => GIB,
            Self::TB => TB,
            Self::TiB => TIB,
        }
    }

    fn as_bytes(&self, n: u64) -> u64 {
        n * self.base()
    }
}
