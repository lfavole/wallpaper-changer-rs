//! Utility functions to manage the config.
use log::debug;
use serde::Deserialize;
use std::error::Error;
use std::fs;

use crate::paths::Paths;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
/// The configuration of the program.
pub(crate) struct Config {
    pub(crate) api_key: String,
    pub(crate) font_size: u32,
    pub(crate) images_per_download: u32,
    pub(crate) label_position: String,
    pub(crate) pictures_folder: String,
    pub(crate) search_terms: String,
    pub(crate) use_unsplash: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            font_size: 28,
            images_per_download: 10,
            label_position: "top_right".to_string(),
            pictures_folder: dirs::picture_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            search_terms: String::new(),
            use_unsplash: true,
        }
    }
}

impl Config {
    /// Loads the config from the `config.toml` file.
    ///
    /// # Errors
    /// Fails if the config directory can't be determined or if the file is malformed or can't be read.
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Paths::config_file();
        debug!("Config path: {:?}", config_path);

        if !config_path.exists() {
            debug!("Config file not found, using default values");
            return Ok(Self::default());
        }
        debug!("Loading config");
        let config_contents = fs::read_to_string(config_path)?;
        debug!("Config length: {}", config_contents.len());
        let config = toml::from_str(&config_contents)?;
        debug!("Config loaded: {:?}", config);
        Ok(config)
    }
}
