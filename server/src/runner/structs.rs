use serde::Deserialize;

/// Config data for each language (`docker/data.toml` file)
#[derive(Debug, Clone, Deserialize)]
pub struct LanguageData {
    pub image: String,
    pub command: String,
    pub template: String,
    pub main_file: String,
}
