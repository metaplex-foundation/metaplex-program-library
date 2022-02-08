use anyhow::Result;
use glob::glob;
use rayon::prelude::*;
use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
};

pub mod errors;
pub mod format;
pub mod parser;

use crate::common::*;
use errors::{DeserializeError, FileOpenError, ValidateError};
use format::Metadata;

pub struct ValidateArgs {
    pub assets_dir: String,
    pub strict: bool,
}

pub fn process_validate(args: ValidateArgs) -> Result<()> {
    let assets_dir = Path::new(&args.assets_dir);

    // Missing or empty assets directory
    if !assets_dir.exists() || assets_dir.read_dir()?.next().is_none() {
        info!("Assets directory is empty.");
        return Err(ValidateError::MissingOrEmptyAssetsDirectory.into());
    }

    let path = assets_dir.join("*.json");
    let pattern = path.to_str().ok_or(ValidateError::InvalidAssetsDirectory)?;

    let (paths, errors): (Vec<_>, Vec<_>) = glob(pattern)?.into_iter().partition(Result::is_ok);

    let paths: Vec<_> = paths.into_iter().map(Result::unwrap).collect();
    let _path_errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    let file_open_errors = Arc::new(Mutex::new(Vec::new()));
    let deserialize_errors = Arc::new(Mutex::new(Vec::new()));
    let validate_errors = Arc::new(Mutex::new(Vec::new()));

    paths.par_iter().for_each(|path| {
        let file_open_errors = file_open_errors.clone();
        let f = match File::open(path) {
            Ok(f) => f,
            Err(error) => {
                error!("{}: {}", path.display(), error);
                file_open_errors
                    .lock()
                    .unwrap()
                    .push(FileOpenError { path, error });
                return;
            }
        };

        let metadata = match serde_json::from_reader::<File, Metadata>(f) {
            Ok(metadata) => metadata,
            Err(error) => {
                error!("{}: {}", path.display(), error);
                deserialize_errors
                    .lock()
                    .unwrap()
                    .push(DeserializeError { path, error });
                return;
            }
        };

        if args.strict {
            match metadata.validate_strict() {
                Ok(()) => {}
                Err(e) => {
                    error!("{}: {}", path.display(), e);
                    validate_errors.lock().unwrap().push(e);
                }
            }
        } else {
            match metadata.validate() {
                Ok(()) => {}
                Err(e) => {
                    error!("{}: {}", path.display(), e);
                    validate_errors.lock().unwrap().push(e);
                }
            }
        }
    });

    info!("Validate complete, your metadata files look good.");
    Ok(())
}
