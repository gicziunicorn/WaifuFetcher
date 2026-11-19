#![warn(unused)]
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
        let config_folder = config_local_dir()
            .or_else(home_dir)
            .context("Failed to find a suitable folder for the config!")?;

        // Custom folder for the app
        let app_folder = config_folder.join(APP_NAME);

        // The file name
        let file_path = app_folder.join("config.toml");

        Ok(Config {
            folder_path: app_folder,
            file_path,
        })
    }

    pub fn read_config(&self) -> Result<Settings> {
        // Read the file's content
        let bytes = fs::read(&self.file_path)
            .context("Failed to read file")?;

        info!("Read configuration from '{}'.", self.file_path.display());

        // Then deserialize it into the struct
        let settings = toml::from_slice(&bytes)
            .context("Failed to parse the configuration file's content")?;

        Ok(settings)
    }

    pub fn save_config(&self, settings: &Settings) -> Result<()> {
        // Convert the struct into a string in toml format
        let toml = toml::to_string_pretty(settings)
            .context("Failed to parse settings into string")?;

        debug!("Parsed settings into toml:\n{}", toml);

        // Create the file's folder(s)
        fs::create_dir_all(&self.folder_path)
            .context("Failed to create directories for configuration file")?;

        // Save the settings
        fs::write(&self.file_path, toml)
            .context("Failed to write configuration to file")?;

        Ok(())
    }
}