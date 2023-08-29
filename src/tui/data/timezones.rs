use color_eyre::Result;
use walkdir::WalkDir;

pub fn get_timezones() -> Result<Vec<String>> {
    let tz = WalkDir::new("/usr/share/zoneinfo")
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| {
            if entry.path().is_dir() {
                return !matches!(entry.file_name().to_str().unwrap(), "posix" | "right");
            }

            entry.path().extension().is_none()
                && !matches!(
                    entry.file_name().to_str().unwrap(),
                    "posixrules" | "SECURITY" | "leapseconds"
                )
        })
        .filter_map(|e| match e {
            Ok(entry) => {
                if entry.path().is_file() {
                    Some(Ok(entry
                        .path()
                        .strip_prefix("/usr/share/zoneinfo")
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()))
                } else {
                    None
                }
            }
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tz)
}
