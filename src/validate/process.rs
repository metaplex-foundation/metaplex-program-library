use anyhow::Result;
use console::style;
use glob::glob;
use rayon::prelude::*;
use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::common::*;
use crate::utils::*;
use crate::validate::*;

pub struct ValidateArgs {
    pub assets_dir: String,
    pub strict: bool,
}

pub fn process_validate(args: ValidateArgs) -> Result<()> {
    // loading assets
    println!(
        "{} {}Loading assets",
        style("[1/1]").bold().dim(),
        ASSETS_EMOJI
    );

    let assets_dir = Path::new(&args.assets_dir);

    // missing or empty assets directory
    if !assets_dir.exists() || assets_dir.read_dir()?.next().is_none() {
        info!("Assets directory is missing or empty.");
        return Err(ValidateError::MissingOrEmptyAssetsDirectory.into());
    }

    let path = assets_dir.join("*.json");
    let pattern = path.to_str().ok_or(ValidateError::InvalidAssetsDirectory)?;

    let (paths, errors): (Vec<_>, Vec<_>) = glob(pattern)?.into_iter().partition(Result::is_ok);

    let pb = spinner_with_style();
    pb.enable_steady_tick(120);
    pb.set_message(format!("Validating {} metadata file(s)...", paths.len()));

    let paths: Vec<_> = paths.into_iter().map(Result::unwrap).collect();
    let path_errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

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

    pb.finish();

    if !path_errors.is_empty() {
        error!("Path errors: {:?}", path_errors);
        return Err(ReadFilesError::PathErrors.into());
    }

    if !file_open_errors.lock().unwrap().is_empty() {
        error!("File open errors: {:?}", file_open_errors);
        return Err(ReadFilesError::FileOpenErrors.into());
    }

    if !deserialize_errors.lock().unwrap().is_empty() {
        error!("Deserialize errors: {:?}", deserialize_errors);
        return Err(ReadFilesError::DeserializeErrors.into());
    }

    if !validate_errors.lock().unwrap().is_empty() {
        error!("Validate errors: {:?}", validate_errors);
        return Err(ReadFilesError::ValidateErrors.into());
    }

    let message = "Validation complete, your metadata file(s) look good.";
    info!("{message}");
    println!("\n{message}");

    Ok(())
}
