use anyhow::Result;
use std::fs::File;
use tracing::error;

use crate::config::data::*;
use crate::config::errors::ConfigError;

pub fn get_config_data(config_path: &String) -> Result<ConfigData, ConfigError> {
    let f = match File::open(config_path) {
        Ok(f) => f,
        Err(_) => {
            let error = ConfigError::FileOpenError(config_path.clone()).into();
            error!("{:?}", error);
            return Err(error);
        }
    };

    let config_data: ConfigData = match serde_json::from_reader(f) {
        Ok(config_data) => config_data,
        Err(err) => {
            let error = ConfigError::ParseError(err.to_string()).into();
            error!("{:?}", error);
            return Err(error);
        }
    };
    Ok(config_data)
}