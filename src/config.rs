use std::{fs, path::PathBuf};
use anyhow::{Context, Result};
use dirs::{home_dir, config_local_dir};
use tracing::{debug, info};
use crate::{APP_NAME, app::Settings};

/// Handles the app's configuration
#[derive(Debug)]
pub struct Config {
    pub folder_path: PathBuf,
    pub file_path: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        // Try to get the local config directory
        let folder = config_local_dir()
            .or_else(home_dir)
            .context("Failed to find a suitable folder for the config!")?;

        // Then get the file
        let file = folder
            .join(APP_NAME)
            .join("config.toml");

        debug!("The configuration file's path is set to {}", file.display());

        let config = Config {
            folder_path: folder,
            file_path: file,
        };
        Ok(config)
    }

    pub fn read_config(&self) -> Result<Settings> {
        // Read the file's content
        let bytes = fs::read(&self.file_path)
            .context("Failed to read file")?;
        info!("Read configuration from '{}'.", self.file_path.display());

        // Then just deserialize it into the struct
        let settings = toml::from_slice(&bytes)
            .context("Failed to parse the configuration file's content")?;

        Ok(settings)
    }

    pub fn save_config(&self, settings: &Settings) -> Result<()> {
        let toml = toml::to_string_pretty(settings)
            .context("Failed to parse settings into string")?;

        debug!("Parsed settings into toml:\n{}", toml);

        fs::create_dir_all(&self.folder_path)
            .context("Failed to create directories for configuration file")?;

        fs::write(&self.file_path, toml)
            .context("Failed to write configuration to file")?;

        Ok(())
    }
}