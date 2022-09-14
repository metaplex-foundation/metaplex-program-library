use super::*;

pub struct EnableFreezeArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub config: String,
    pub candy_machine: Option<String>,
    pub freeze_days: Option<u8>,
}

pub fn process_enable_freeze(args: EnableFreezeArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair.clone(), args.rpc_url.clone())?;
    let client = setup_client(&sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);
    let config_data = get_config_data(&args.config)?;

    // The candy machine id specified takes precedence over the one from the cache.
    let candy_machine_id = match args.candy_machine {
        Some(ref candy_machine_id) => candy_machine_id.to_owned(),
        None => {
            let cache = load_cache(&args.cache, false)?;
            cache.program.candy_machine
        }
    };

    let candy_pubkey = Pubkey::from_str(&candy_machine_id)
        .map_err(|_| anyhow!("Failed to parse candy machine id: {}", &candy_machine_id))?;

    // Freeze days specified takes precedence over the one from the config.
    let freeze_time = if let Some(freeze_days) = args.freeze_days {
        if freeze_days > 30 {
            return Err(anyhow!("Freeze days cannot be more than 30"));
        }

        (freeze_days as i64) * 24 * 60 * 60
    } else if let Some(freeze_time) = config_data.freeze_time {
        freeze_time
    } else {
        return Err(anyhow!(
            "No freeze time specified either in config or as argument"
        ));
    };

    println!(
        "{} {}Loading candy machine",
        style("[1/2]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );
    println!("{} {}", style("Candy machine ID:").bold(), candy_machine_id);

    let pb = spinner_with_style();
    pb.set_message("Connecting...");
    let candy_machine_state =
        get_candy_machine_state(&sugar_config, &Pubkey::from_str(&candy_machine_id)?)?;

    pb.finish_with_message("Done");

    if is_feature_active(&candy_machine_state.data.uuid, FREEZE_FEATURE_INDEX) {
        println!(
            "{} {}Freeze feature is already enabled",
            style("[2/2]").bold().dim(),
            COMPLETE_EMOJI
        );

        return Ok(());
    }

    assert_correct_authority(
        &sugar_config.keypair.pubkey(),
        &candy_machine_state.authority,
    )?;

    // Cannot enable freeze if minting has started.
    if candy_machine_state.items_redeemed > 0 {
        return Err(anyhow!("Cannot enable freeze after minting has started"));
    }

    println!(
        "\n{} {}Turning on freeze feature for candy machine",
        style("[2/2]").bold().dim(),
        ICE_CUBE_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Sending Enable Freeze transaction...");

    let signature = enable_freeze(&program, &config_data, &candy_pubkey, freeze_time)?;

    pb.finish_with_message(format!(
        "{} {}",
        style("Enable freeze signature:").bold(),
        signature
    ));

    Ok(())
}

pub fn enable_freeze(
    program: &Program,
    config: &ConfigData,
    candy_machine_id: &Pubkey,
    freeze_time: i64,
) -> Result<Signature> {
    let (freeze_pda, _) = find_freeze_pda(candy_machine_id);

    let mut builder = program.request();
    let mut additional_accounts = Vec::new();

    // If spl token mint setting is enabled, add the freeze ata to the accounts.
    if let Some(spl_token_mint) = config.spl_token {
        let freeze_ata = get_associated_token_address(&freeze_pda, &spl_token_mint);
        let freeze_ata_ix =
            create_associated_token_account(&program.payer(), &freeze_pda, &spl_token_mint);

        let freeze_ata_account = AccountMeta {
            pubkey: freeze_ata,
            is_signer: false,
            is_writable: true,
        };
        additional_accounts.push(freeze_ata_account);

        builder = builder.instruction(freeze_ata_ix);
    }

    builder = builder
        .accounts(nft_accounts::SetFreeze {
            candy_machine: *candy_machine_id,
            authority: program.payer(),
            freeze_pda,
            system_program: system_program::ID,
        })
        .accounts(additional_accounts) // order matters so we have to add this account at the end
        .args(nft_instruction::SetFreeze { freeze_time });

    let sig = builder.send()?;

    Ok(sig)
}
