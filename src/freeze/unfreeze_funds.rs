use super::*;

pub struct UnlockFundsArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub config: String,
    pub candy_machine: Option<String>,
}

pub fn process_unfreeze_funds(args: UnlockFundsArgs) -> Result<()> {
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

    if !is_feature_active(&candy_machine_state.data.uuid, FREEZE_LOCK_FEATURE_INDEX) {
        println!(
            "{} {}Candy machine treasury funds are already unfrozen",
            style("[2/2]").bold().dim(),
            COMPLETE_EMOJI
        );
        return Ok(());
    }

    assert_correct_authority(
        &sugar_config.keypair.pubkey(),
        &candy_machine_state.authority,
    )?;

    println!(
        "\n{} {}Unlocking treasury funds. . .",
        style("[2/2]").bold().dim(),
        MONEY_BAG_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Sending unlock funds transaction...");

    let signature = unlock_funds(
        &program,
        &config_data,
        &candy_pubkey,
        candy_machine_state.wallet,
    )?;

    pb.finish_with_message(format!(
        "{} {}",
        style("Unlock funds signature:").bold(),
        signature
    ));

    Ok(())
}

pub fn unlock_funds(
    program: &Program,
    config: &ConfigData,
    candy_machine_id: &Pubkey,
    treasury: Pubkey,
) -> Result<Signature> {
    let (freeze_pda, _) = find_freeze_pda(candy_machine_id);

    let mut additional_accounts = Vec::new();

    // If spl token mint setting is enabled, add the freeze ata to the accounts.
    if let Some(spl_token_mint) = config.spl_token {
        // Add Token program account.
        additional_accounts.push(AccountMeta {
            pubkey: spl_token::id(),
            is_signer: false,
            is_writable: false,
        });

        // Add the freeze ata.
        let freeze_ata = get_associated_token_address(&freeze_pda, &spl_token_mint);

        let freeze_ata_account = AccountMeta {
            pubkey: freeze_ata,
            is_signer: false,
            is_writable: true,
        };
        additional_accounts.push(freeze_ata_account);

        // Add the treasury ata.
        let treasury_ata = if let Some(treasury_ata) = config.spl_token_account {
            treasury_ata
        } else {
            get_associated_token_address(&treasury, &spl_token_mint)
        };
        let treasury = AccountMeta {
            pubkey: treasury_ata,
            is_signer: false,
            is_writable: true,
        };
        additional_accounts.push(treasury);
    }

    let builder = program
        .request()
        .accounts(nft_accounts::UnlockFunds {
            candy_machine: *candy_machine_id,
            authority: program.payer(),
            wallet: treasury,
            freeze_pda,
            system_program: system_program::ID,
        })
        .accounts(additional_accounts) // order matters so we have to add this account at the end
        .args(nft_instruction::UnlockFunds);

    let sig = builder.send()?;

    Ok(sig)
}
