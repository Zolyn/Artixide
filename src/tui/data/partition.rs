use std::process::Command;

use color_eyre::Result;
use derive_getters::Getters;
use indexmap::IndexMap;
use itoap::Integer;
use log::debug;

use serde::Deserialize;
use strum::{AsRefStr, EnumCount, EnumDiscriminants, EnumString};
use typed_builder::TypedBuilder;

use crate::extensions::CommandExt;

mod impls;

const ESP_GUID: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";
const BOOT_FLAG: &str = "0x80";
const EXTENDED_TYPE: &str = "0x5";

pub const DEFAULT_ALIGN: u64 = 2048;

#[derive(Debug, EnumDiscriminants, EnumCount)]
#[strum_discriminants(name(ModificationType))]
pub enum Modification {
    Delete,
    Create { start: u64, end: u64 },
}

#[derive(Debug)]
pub struct ModificationSet {
    inner: Vec<Option<Modification>>,
}

// TODO: Better getter/setter impl
#[derive(Debug, TypedBuilder)]
pub struct MemPartition {
    pub number: u16,
    pub start: u64,
    pub end: u64,
    pub sectors: u64,
    pub size: u64,
    #[builder(default = itoa(self.fields.0.0))]
    pub number_string: String,
    #[builder(default = itoa(self.fields.1.0))]
    pub start_string: String,
    #[builder(default = itoa(self.fields.2.0))]
    pub end_string: String,
    #[builder(default = itoa(self.fields.3.0))]
    pub sectors_string: String,
    #[builder(default = format_size(self.fields.4.0))]
    pub size_string: String,
    #[builder(default, setter(skip))]
    pub bootable: bool,
    #[builder(default, setter(skip))]
    pub filesystem: FileSystem,
    #[builder(default, setter(skip))]
    pub label: Option<String>,
    #[builder(default, setter(skip))]
    pub mountpoint: Option<String>,
    #[builder(default, setter(skip))]
    /// Indicate whether the partition is real or in-memory
    /// Only used for validation purpose (in the future)
    uuid: Option<String>,
}

#[derive(Debug)]
pub struct RawSpace {
    start: u64,
    end: u64,
    sectors: u64,
    size: u64,
}

#[derive(Debug, TypedBuilder)]
pub struct DiskSpace {
    pub start: u64,
    pub end: u64,
    pub sectors: u64,
    pub size: u64,
    #[builder(default = itoa(self.fields.0.0))]
    pub start_string: String,
    #[builder(default = itoa(self.fields.1.0))]
    pub end_string: String,
    #[builder(default = itoa(self.fields.2.0))]
    pub sectors_string: String,
    #[builder(default = format_size(self.fields.3.0))]
    pub size_string: String,
}

#[derive(Debug)]
pub enum MemTableEntry {
    Partition(MemPartition),
    Free(DiskSpace),
}

pub struct NumberPool {
    inner: Vec<bool>,
}

#[derive(Debug)]
pub struct CompatDevice {
    pub disk: Disk,
    pub mem_table: Vec<MemTableEntry>,
    pub number_pool: NumberPool,
    pub modification_map: IndexMap<u16, ModificationSet>,
}

#[derive(Debug)]
pub enum Device {
    /// Device with known table
    Compatible(CompatDevice),
    /// Device with unrecognized table
    Incompatible(RawDisk),
}

#[derive(Debug)]
pub enum PartitionType {
    // MBR & GPT
    Primary,
    // MBR
    Extended,
    Logical,
}

#[derive(Debug, Clone, Copy, AsRefStr, EnumString, Default)]
#[strum(serialize_all = "lowercase")]
pub enum FileSystem {
    Ext2,
    Ext3,
    Ext4,
    Btrfs,
    Xfs,
    Swap,
    Fat16,
    Fat32,
    ExFat,
    Ntfs,
    #[default]
    Unknown,
}

#[derive(Debug, Getters)]
pub struct Disk {
    model: String,
    path: String,
    id: String,
    size: u64,
    size_string: String,
    starting_lba: u64,
    ending_lba: u64,
    sector_size: u16,
    is_gpt: bool,
}

#[derive(Debug, Getters)]
pub struct RawDisk {
    model: String,
    path: String,
    size: u64,
    size_string: String,
    sector_size: u16,
}

#[derive(Debug, Deserialize)]
pub struct LsblkResult<'a> {
    #[serde(borrow)]
    pub blockdevices: Vec<BlockDevice<'a>>,
}

/// NOTE: Multiple level children are not supported
#[derive(Debug, Deserialize)]
pub struct ChildBlockDevice<'a> {
    start: u64,
    size: u64,
    #[serde(rename(deserialize = "type"))]
    devtype: &'a str,
    partn: u16,
    partuuid: &'a str,
    parttype: &'a str,
    partflags: Option<&'a str>,
    label: Option<&'a str>,
    fstype: Option<&'a str>,
    mountpoint: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct BlockDevice<'a> {
    size: u64,
    #[serde(rename(deserialize = "log-sec"))]
    log_sec: u16,
    path: &'a str,
    #[serde(rename(deserialize = "type"))]
    devtype: &'a str,
    model: Option<&'a str>,
    pttype: Option<&'a str>,
    ptuuid: Option<&'a str>,
    children: Option<Vec<ChildBlockDevice<'a>>>,
}

pub fn get_devices() -> Result<Vec<Device>> {
    let output = Command::new("lsblk")
        .args([
            "-J",
            "-T",
            "-b",
            "-o",
            "path,label,type,size,log-sec,start,pttype,ptuuid,partn,partuuid,parttype,partflags,fstype,mountpoint,model",
        ])
        .read()?;

    let result: LsblkResult = serde_json::from_str(&output)?;
    debug!("{:#?}", result);

    let devs = result
        .blockdevices
        .into_iter()
        .filter_map(|dev| match Device::new_from(dev) {
            Ok(d) => d.map(Ok),
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>>>()?;

    debug!("Devices: {:#?}", devs);

    Ok(devs)
}

fn format_size(size: u64) -> String {
    const BINARY: bool = true;

    bytesize::to_string(size, BINARY)
}

fn itoa<V: Integer>(n: V) -> String {
    let mut s = String::new();
    itoap::write_to_string(&mut s, n);
    s
}
