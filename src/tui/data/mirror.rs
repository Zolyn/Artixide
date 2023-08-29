use std::{fs, mem};

use color_eyre::Result;
use log::error;
use regex::Regex;

use crate::lazy;

lazy! {
    static URL_RE: Regex = Regex::new(r"https?://(?<host>([a-z0-9-]+\.)+[a-z]+)/.*").unwrap()
}

const FIRST_GROUP: &str = "# Default mirrors";

fn get_grouped_mirrors() -> Result<Vec<(String, Vec<String>)>> {
    // TODO:
    let mirror_list = fs::read_to_string("./mirrorlist")?;

    let mut group = FIRST_GROUP.into();
    let mut servers = vec![];
    let mut result = vec![];

    for line in mirror_list
        .lines()
        .skip_while(|line| !line.starts_with(FIRST_GROUP))
        .skip(1)
    {
        if line.starts_with("Server") {
            servers.push(line.into())
        } else if line.starts_with("# ") {
            result.push((
                mem::replace(&mut group, line.into()),
                mem::take(&mut servers),
            ))
        }
    }

    result.push((group, servers));

    Ok(result)
}

fn trim_server_url(url: &str) -> &str {
    let url = &url[9..];
    let range = {
        let caps = URL_RE.captures(url);

        if let Some(caps) = caps {
            caps.name("host").unwrap().range()
        } else {
            error!("Failed to match url {}, fallback to full url", url);
            0..url.len() - 1
        }
    };

    &url[range]
}

type MirrorList = (Vec<String>, Vec<Vec<String>>, Vec<String>, usize);

pub fn get_mirrors() -> Result<MirrorList> {
    let (group, servers): (Vec<_>, Vec<_>) = get_grouped_mirrors()?.into_iter().unzip();

    let default_servers_count = servers[0].len();

    let trim_servers: Vec<String> = servers
        .iter()
        .flatten()
        .skip(default_servers_count)
        .map(|i| trim_server_url(i).into())
        .collect();

    Ok((group, servers, trim_servers, default_servers_count))
}
