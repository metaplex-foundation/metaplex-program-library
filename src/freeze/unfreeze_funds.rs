use super::*;

pub struct UnlockFundsArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_machine: Option<String>,
}

pub fn process_unfreeze_funds(args: UnlockFundsArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair.clone(), args.rpc_url.clone())?;
    let client = setup_client(&sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);

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

    let signature = unlock_funds(&program, &candy_pubkey, candy_machine_state.wallet)?;

    pb.finish_with_message(format!(
        "{} {}",
        style("Unlock funds signature:").bold(),
        signature
    ));

    Ok(())
}

pub fn unlock_funds(
    program: &Program,
    candy_machine_id: &Pubkey,
    treasury: Pubkey,
) -> Result<Signature> {
    let (freeze_pda, _) = find_freeze_pda(candy_machine_id);

    let builder = program
        .request()
        .accounts(nft_accounts::UnlockFunds {
            candy_machine: *candy_machine_id,
            authority: program.payer(),
            wallet: treasury,
            freeze_pda,
            system_program: system_program::ID,
        })
        .args(nft_instruction::UnlockFunds);

    let sig = builder.send()?;

    Ok(sig)
}
