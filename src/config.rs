use self::locale::LocaleConfig;

pub mod locale;

#[derive(Debug, Default)]
pub struct Config {
    pub keyboard_layout: String,
    pub mirror: String,
    pub locale: LocaleConfig,
    pub hostname: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            keyboard_layout: "us".to_string(),
            hostname: "artix".to_string(),
            ..Default::default()
        }
    }
}
