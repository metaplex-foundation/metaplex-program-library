use std::sync::{atomic::AtomicBool, Arc};

use anyhow::Result;
use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::{
    common::LAUNCH_EMOJI,
    config::parser::get_config_data,
    create_config::{process_create_config, CreateConfigArgs},
    deploy::{process_deploy, DeployArgs},
    upload::{process_upload, UploadArgs},
    validate::{process_validate, ValidateArgs},
    verify::{process_verify, VerifyArgs},
};

pub struct LaunchArgs {
    pub assets_dir: String,
    pub config: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub strict: bool,
    pub skip_collection_prompt: bool,
    pub interrupted: Arc<AtomicBool>,
}

pub async fn process_launch(args: LaunchArgs) -> Result<()> {
    println!("Starting Sugar launch... {}", LAUNCH_EMOJI);

    let theme = ColorfulTheme {
        prompt_style: Style::new(),
        ..Default::default()
    };

    if let Err(err) = get_config_data(&args.config) {
        // padding
        println!();
        if Confirm::with_theme(&theme)
            .with_prompt("Could not load config file. Would you like to create a new config file?")
            .interact()?
        {
            println!("\n{} sugar create-config\n", style(">>>").magenta());

            let create_config_args = CreateConfigArgs {
                config: Some(args.config.clone()),
                keypair: args.keypair.clone(),
                rpc_url: args.rpc_url.clone(),
                assets_dir: args.assets_dir.clone(),
            };

            process_create_config(create_config_args)?;
        } else {
            return Err(err.into());
        }
    }

    println!("\n{} sugar validate\n", style(">>>").magenta());

    let validate_args = ValidateArgs {
        assets_dir: args.assets_dir.clone(),
        strict: args.strict,
        skip_collection_prompt: args.skip_collection_prompt,
    };

    process_validate(validate_args)?;

    println!("\n{} sugar upload\n", style(">>>").magenta());

    let upload_args = UploadArgs {
        assets_dir: args.assets_dir.clone(),
        config: args.config.clone(),
        keypair: args.keypair.clone(),
        rpc_url: args.rpc_url.clone(),
        cache: args.cache.clone(),
        interrupted: args.interrupted.clone(),
    };

    process_upload(upload_args).await?;

    println!("\n{} sugar deploy\n", style(">>>").magenta());

    let deploy_args = DeployArgs {
        config: args.config.clone(),
        keypair: args.keypair.clone(),
        rpc_url: args.rpc_url.clone(),
        cache: args.cache.clone(),
        interrupted: args.interrupted.clone(),
    };

    process_deploy(deploy_args).await?;

    println!("\n{} sugar verify\n", style(">>>").magenta());

    let verify_args = VerifyArgs {
        keypair: args.keypair.clone(),
        rpc_url: args.rpc_url.clone(),
        cache: args.cache.clone(),
    };

    process_verify(verify_args)?;

    Ok(())
}
