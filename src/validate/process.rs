use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Confirm};
use glob::glob;
use rayon::prelude::*;

use crate::{common::*, utils::*, validate::*};

pub struct ValidateArgs {
    pub assets_dir: String,
    pub strict: bool,
    pub skip_collection_prompt: bool,
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
        return Err(ValidateParserError::MissingOrEmptyAssetsDirectory.into());
    }

    if !args.skip_collection_prompt {
        let collection_path = assets_dir.join("collection.json");
        if !collection_path.is_file() {
            let warning = format!(
                "+----------------------------------------------+\n\
                 | {} MISSING COLLECTION FILES IN ASSETS FOLDER |\n\
                 +----------------------------------------------+",
                WARNING_EMOJI
            );
            println!(
                "\n{}\n{}\n",
                style(warning).bold().yellow(),
                style(
                    "Check https://docs.metaplex.com/tools/sugar/asset-preparation-and-deployment#collection-assets for the requirements \
                    if you want a collection to be set automatically."
                )
                .italic()
                .yellow()
            );

            let theme = ColorfulTheme {
                success_prefix: style("✔".to_string()).yellow().force_styling(true),
                values_style: Style::new().yellow(),
                ..get_dialoguer_theme()
            };

            if !Confirm::with_theme(&theme).with_prompt("Do you want to continue without automatically setting the candy machine collection?").interact()? {
                return Err(anyhow!("Operation aborted"));
            }
            println!();
        }
    }

    let errors = Arc::new(Mutex::new(Vec::new()));

    let path = assets_dir.join("*.json");
    let pattern = path
        .to_str()
        .ok_or(ValidateParserError::InvalidAssetsDirectory)?;

    // Unwrapping here because we know the pattern is valid and GlobErrors should
    // be rare or impossible to produce.
    let paths: Vec<PathBuf> = glob(pattern)
        .unwrap()
        .into_iter()
        .map(Result::unwrap)
        .collect();

    let pb = spinner_with_style();
    pb.enable_steady_tick(120);
    pb.set_message(format!("Validating {} metadata file(s)...", paths.len()));

    paths.par_iter().for_each(|path| {
        let errors = errors.clone();
        let f = match File::open(path) {
            Ok(f) => f,
            Err(error) => {
                error!("{}: {}", path.display(), error);
                errors.lock().unwrap().push(ValidateError {
                    path,
                    error: error.to_string(),
                });
                return;
            }
        };

        let metadata = match serde_json::from_reader::<File, Metadata>(f) {
            Ok(metadata) => metadata,
            Err(error) => {
                error!("{}: {}", path.display(), error);
                errors.lock().unwrap().push(ValidateError {
                    path,
                    error: error.to_string(),
                });
                return;
            }
        };

        // To be replaced with the strict validator once JSON standard is finalized.
        if args.strict {
            match metadata.validate() {
                Ok(()) => {}
                Err(e) => {
                    error!("{}: {}", path.display(), e);
                    errors.lock().unwrap().push(ValidateError {
                        path,
                        error: e.to_string(),
                    });
                }
            }
        } else {
            match metadata.validate() {
                Ok(()) => {}
                Err(e) => {
                    error!("{}: {}", path.display(), e);
                    errors.lock().unwrap().push(ValidateError {
                        path,
                        error: e.to_string(),
                    });
                }
            }
        }
    });

    pb.finish();

    if !errors.lock().unwrap().is_empty() {
        log_errors("validate_errors", errors)?;
        return Err(anyhow!(
            "Validation error: see 'validate_errors.json' file for details"
        ));
    }

    let message = "Validation complete, your metadata file(s) look good.";
    info!("{message}");
    println!("\n{message}");

    Ok(())
}
