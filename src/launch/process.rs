use anyhow::Result;
use console::Style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use std::process::exit;

use crate::common::NEW_CONFIG_EMOJI;
use crate::config::parser::get_config_data;
use crate::create_config::process_create_config;
use crate::deploy::{process_deploy, DeployArgs};
use crate::upload::{process_upload, UploadArgs};
use crate::validate::{process_validate, ValidateArgs};
use crate::verify::{process_verify, VerifyArgs};
use std::sync::Arc;

pub struct LaunchArgs {
    pub assets_dir: String,
    pub config: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub strict: bool,
}

pub async fn process_launch(args: LaunchArgs) -> Result<()> {
    let validate_args = Arc::new(&args);

    let validate_args = ValidateArgs {
        assets_dir: validate_args.assets_dir.clone(),
        strict: validate_args.strict,
    };

    if let Err(err) = process_validate(validate_args) {
        println!("Error: {}", err);
        exit(1)
    };

    println!("\n");

    let theme = ColorfulTheme {
        prompt_style: Style::new(),
        ..Default::default()
    };

    if let Err(err) = get_config_data(&args.config) {
        if Confirm::with_theme(&theme)
            .with_prompt(format!(
                "No configuration file found. Would you like to create a new config file? {}",
                NEW_CONFIG_EMOJI
            ))
            .interact()?
        {
            process_create_config()?;
        } else {
            println!("Error: {:?}", err);
            exit(1)
        }
    }

    println!("\n");

    let upload_args = Arc::new(&args);

    let upload_args = UploadArgs {
        assets_dir: upload_args.assets_dir.clone(),
        config: upload_args.config.clone(),
        keypair: upload_args.keypair.clone(),
        rpc_url: upload_args.rpc_url.clone(),
        cache: upload_args.cache.clone(),
    };

    if let Err(err) = process_upload(upload_args).await {
        println!("Error: {}", err);
        exit(1)
    };

    println!("\n");

    let deploy_args = Arc::new(&args);

    let deploy_args = DeployArgs {
        assets_dir: deploy_args.assets_dir.clone(),
        config: deploy_args.config.clone(),
        keypair: deploy_args.keypair.clone(),
        rpc_url: deploy_args.rpc_url.clone(),
        cache: deploy_args.cache.clone(),
    };

    if let Err(err) = process_deploy(deploy_args).await {
        println!("Error: {}", err);
        exit(1)
    };

    let verify_args = Arc::new(&args);

    let verify_args = VerifyArgs {
        keypair: verify_args.keypair.clone(),
        rpc_url: verify_args.rpc_url.clone(),
        cache: verify_args.cache.clone(),
    };

    if let Err(err) = process_verify(verify_args) {
        println!("Error: {}", err);
        exit(1)
    };
    println!("\n");

    Ok(())
}
