use self::locale::LocaleConfig;

mod locale;

#[derive(Debug, Default)]
pub struct Config {
    pub keyboard_layout: String,
    pub mirror: String,
    pub locale: LocaleConfig,
}

impl Config {
    pub fn new() -> Self {
        Self {
            keyboard_layout: "us".to_string(),
            ..Default::default()
        }
    }
}
