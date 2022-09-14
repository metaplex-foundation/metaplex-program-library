use super::*;

pub struct DisableFreezeArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub candy_machine: Option<String>,
}

pub fn process_disable_freeze(args: DisableFreezeArgs) -> Result<()> {
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

    if !is_feature_active(&candy_machine_state.data.uuid, FREEZE_FEATURE_INDEX) {
        println!(
            "{} {}Freeze feature is already disabled",
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
        "\n {}{}Turning off freeze feature for candy machine",
        style("[2/2]").bold().dim(),
        FIRE_EMOJI,
    );

    let pb = spinner_with_style();
    pb.set_message("Sending remove freeze transaction...");

    let signature = disable_freeze(&program, &candy_pubkey)?;

    pb.finish_with_message(format!(
        "{} {}",
        style("Set freeze signature:").bold(),
        signature
    ));

    Ok(())
}

fn disable_freeze(program: &Program, candy_machine_id: &Pubkey) -> Result<Signature> {
    let (freeze_pda, _) = find_freeze_pda(candy_machine_id);

    let builder = program
        .request()
        .accounts(nft_accounts::RemoveFreeze {
            candy_machine: *candy_machine_id,
            authority: program.payer(),
            freeze_pda,
        })
        .args(nft_instruction::RemoveFreeze);

    let sig = builder.send()?;

    Ok(sig)
}
