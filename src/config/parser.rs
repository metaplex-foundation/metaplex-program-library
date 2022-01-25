use anyhow::Result;
use std::fs::File;

use crate::config::data::*;
use crate::config::errors::ConfigError;

pub fn get_config_data(config_path: &String) -> Result<ConfigData, ConfigError> {
    let f = match File::open(config_path) {
        Ok(f) => f,
        Err(_) => return Err(ConfigError::FileOpenError(config_path.clone())),
    };

    let config_data: ConfigData = match serde_json::from_reader(f) {
        Ok(config_data) => config_data,
        Err(err) => {
            return Err(ConfigError::ParseError(err.to_string()));
        }
    };
    Ok(config_data)
}
