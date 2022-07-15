use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::Serialize;
use thiserror::Error;

use crate::common::*;

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("Error setting up sugar: {0}")]
    SugarSetupError(String),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache file '{0}' not found. Run `sugar upload` to create it or provide it with the --cache option.")]
    CacheFileNotFound(String),

    #[error("Invalid candy machine address: '{0}'. Check your cache file or run deploy to ensure your candy machine was created.")]
    InvalidCandyMachineAddress(String),

    #[error("Failed to open cache file: {0} with error: {1}")]
    FailedToOpenCacheFile(String, String),

    #[error("Failed to parse cache file with error: {0}")]
    CacheFileWrongFormat(String),

    #[error("Invalid cache state found.")]
    InvalidState,
}

#[derive(Debug, Error)]
pub enum CustomCandyError {
    #[error("Payer key '{0}' does not equal the Candy Machine authority pubkey '{1}'")]
    AuthorityMismatch(String, String),
}

#[derive(Debug, Serialize)]
pub struct ValidateError<'a> {
    pub path: &'a PathBuf,
    pub error: String,
}

pub fn log_errors<T: std::fmt::Debug + Serialize>(
    error_type: &str,
    errors: Arc<Mutex<Vec<T>>>,
) -> Result<()> {
    let errors = &*errors.lock().unwrap();
    error!("{error_type}: {errors:?}");
    let f = File::create("validate_errors.json")?;
    serde_json::to_writer_pretty(f, &errors)?;

    Ok(())
}
