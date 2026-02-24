use serde::Deserialize;

/// Config data for each language (`docker/data.toml` file)
#[derive(Debug, Clone, Deserialize)]
pub struct LanguageData {
    /// The base Docker image tag
    pub image: String,
    /// The command to run the program
    pub command: String,
    /// Name of the template folder within the `docker/templates` directory
    pub template: String,
    /// The name of the function file
    #[serde(rename = "fn_file")]
    pub fn_filename: String,
    /// The name of the main file to run
    #[serde(rename = "main_file")]
    pub main_filename: String,
}

/// Template data for each language (in the `docker/templates` directory)
pub struct LanguageTemplates {
    /// The Dockerfile template
    pub dockerfile: liquid::Template,
    /// The code of the main file for running functions
    pub main_file: String,
    /// A sample function file showing the correct inputs / outputs for the function
    pub fn_file: String,
}
