use color_eyre::Result;
use walkdir::WalkDir;

pub fn get_keyboard_layouts() -> Result<Vec<String>> {
    let mut layouts = WalkDir::new("/usr/share/kbd/keymaps")
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(Ok(entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_suffix(".map.gz")?
                .to_string())),
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    layouts.sort_unstable();

    Ok(layouts)
}
