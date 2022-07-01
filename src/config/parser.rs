use std::{
    fs::{metadata, OpenOptions},
    io::ErrorKind,
};

use anyhow::Result;
use tracing::error;

use crate::config::{data::*, errors::ConfigError};

pub fn get_config_data(config_path: &str) -> Result<ConfigData, ConfigError> {
    // checks that the config file exists and it is readable
    let f = match OpenOptions::new().read(true).open(config_path) {
        Ok(f) => f,
        Err(err) => {
            let error = match err.kind() {
                ErrorKind::NotFound => ConfigError::MissingFileError(config_path.to_string()),
                _ => ConfigError::PermissionError(config_path.to_string()),
            };

            error!("{:?}", error);
            return Err(error);
        }
    };
    // checks that the config is a file and not a directory
    if metadata(config_path).unwrap().is_dir() {
        let error = ConfigError::InvalidPathError(config_path.to_string());
        error!("{:?}", error);
        return Err(error);
    }

    let config_data: ConfigData = match serde_json::from_reader(f) {
        Ok(config_data) => config_data,
        Err(err) => {
            let error = ConfigError::ParseError(err.to_string());
            error!("{:?}", error);
            return Err(error);
        }
    };
    Ok(config_data)
}
