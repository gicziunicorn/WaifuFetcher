use std::{fmt::format, fs, path::PathBuf};
use dirs::{home_dir, picture_dir, config_local_dir};
use log::{debug, info};

use crate::{APP_NAME, ResultToString, app::{AppData, Settings}};

/// Handles the app's configuration
pub struct Config {
    pub folder: PathBuf,
    pub file: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        // Try to get the local config directory
        let folder = config_local_dir()
            .or_else(home_dir)
            .ok_or_else(|| "Failed to find a suitable folder for the config!".to_string())?;

        // Then get the file
        let file = folder
            .join(APP_NAME)
            .join("config.toml");

        info!("The config file's path is set to {}", file.display());

        let config = Config {
            folder,
            file,
        };
        Ok(config)
    }

    pub fn read_config(&self) -> Result<Settings, String> {
        // Read the file's content
        let bytes = fs::read(&self.file)
            .err_to_string()?;
        info!("Read config from '{}'.", self.file.display());

        // Then just deserialize it into the struct
        toml::from_slice(&bytes).err_to_string()
    }

    pub fn save_config(&self, settings: &Settings) -> Result<(), String> {
        let toml = toml::to_string_pretty(settings)
            .map_err(|e| format!("Couldn't create toml data from settings: {}", e.to_string()) )?;

        debug!("Toml data:\n{}", toml);

        fs::create_dir_all(&self.folder)
            .err_to_string()?;

        info!("File: {:?}", &self.file);
        fs::write(&self.file, toml)
            .err_to_string()?;

        Ok(())
    }
}