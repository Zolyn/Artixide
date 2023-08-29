#[derive(Debug)]
pub struct LocaleConfig {
    pub lang: String,
    pub encoding: String,
    pub modifier: Option<String>,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            lang: "en_US".to_string(),
            encoding: "UTF-8".to_string(),
            modifier: None,
        }
    }
}
