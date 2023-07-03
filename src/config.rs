#[derive(Debug, Default)]
pub struct Config {
    pub keyboard_layout: String,
    pub mirror: String,
    pub locale_lang: String,
    pub locale_encoding: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            keyboard_layout: "us".to_string(),
            ..Default::default()
        }
    }
}
