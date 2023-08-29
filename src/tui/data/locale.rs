use color_eyre::{eyre::eyre, Result};
use regex::Regex;
use std::{collections::BTreeSet, fs};

pub fn get_locales() -> Result<(Vec<String>, Vec<String>)> {
    let locale_re =
        Regex::new(r"^(?<locale>#?[a-z]+(_[A-Z]+)?(\@[a-z]+)?(\.[^\s]+)?)\s(?<encoding>[^\s]+)")
            .unwrap();
    let locale_gen = fs::read_to_string("/etc/locale.gen")?;

    let mut encoding_set: BTreeSet<&str> = BTreeSet::new();

    let langs = locale_gen
        .lines()
        .enumerate()
        .skip_while(|(_, line)| !locale_re.is_match(line))
        .map(|(i, line)| {
            let caps = locale_re
                .captures(line)
                .ok_or_else(|| eyre!("Failed to match locale: {}(line {})", line, i))?;

            encoding_set.insert(caps.name("encoding").unwrap().as_str());

            Ok(caps
                .name("locale")
                .unwrap()
                .as_str()
                .trim_start_matches('#')
                .to_owned())
        })
        .collect::<Result<Vec<_>>>()?;

    let encodings = encoding_set
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();

    Ok((langs, encodings))
}
