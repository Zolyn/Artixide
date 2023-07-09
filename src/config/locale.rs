#[derive(Debug)]
pub struct LocaleConfig {
    lang: String,
    encoding: String,
    modifier: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            lang: "en_US".to_string(),
            encoding: "UTF-8".to_string(),
            modifier: String::new()

        }
    }
}
