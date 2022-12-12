use mpl_candy_guard::{
    accounts::Route as RouteAccount, guards::FreezeInstruction, instruction::Route,
    instructions::RouteArgs, state::GuardType,
};

use super::*;

pub struct UnlockFundsArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub cache: String,
    pub config: String,
    pub candy_guard: Option<String>,
    pub candy_machine: Option<String>,
    pub destination: Option<String>,
    pub label: Option<String>,
}

pub fn process_unlock_funds(args: UnlockFundsArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair.clone(), args.rpc_url.clone())?;
    let client = setup_client(&sugar_config)?;
    let program = client.program(mpl_candy_guard::ID);

    // candy guard id specified takes precedence over the one from the cache
    let candy_guard_id = match args.candy_guard {
        Some(ref candy_guard_id) => candy_guard_id.to_owned(),
        None => {
            let cache = load_cache(&args.cache, false)?;
            cache.program.candy_guard
        }
    };

    // candy machine id specified takes precedence over the one from the cache
    let candy_machine_id = match args.candy_machine {
        Some(ref candy_machine_id) => candy_machine_id.to_owned(),
        None => {
            let cache = load_cache(&args.cache, false)?;
            cache.program.candy_machine
        }
    };

    let candy_guard = Pubkey::from_str(&candy_guard_id)
        .map_err(|_| anyhow!("Failed to parse candy guard id: {}", &candy_guard_id))?;

    let candy_machine = Pubkey::from_str(&candy_machine_id)
        .map_err(|_| anyhow!("Failed to parse candy machine id: {}", &candy_guard_id))?;

    println!(
        "{} {}Loading freeze escrow information",
        style("[1/2]").bold().dim(),
        LOOKING_GLASS_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    // destination address specified takes precedence over the one from the cache
    let destination_address = match args.destination {
        Some(ref destination_address) => Pubkey::from_str(destination_address).map_err(|_| {
            anyhow!(
                "Failed to parse destination address: {}",
                &destination_address
            )
        })?,
        None => get_destination(
            &program,
            &candy_guard,
            get_config_data(&args.config)?,
            &args.label,
        )?,
    };

    // sanity check: loads the PDA
    let (freeze_escrow, _) = find_freeze_pda(&candy_guard, &candy_machine, &destination_address);
    let account_data = program
        .rpc()
        .get_account_data(&freeze_escrow)
        .map_err(|_| anyhow!("Could not load freeze escrow"))?;

    if account_data.is_empty() {
        return Err(anyhow!("Freeze escrow account not found"));
    }

    pb.finish_with_message("Done");

    println!(
        "\n{} {}Unlocking treasury funds",
        style("[2/2]").bold().dim(),
        MONEY_BAG_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Sending unlock funds transaction...");

    let signature = unlock_funds(
        &program,
        &candy_guard,
        &candy_machine,
        &destination_address,
        &args.label,
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
    candy_guard_id: &Pubkey,
    candy_machine_id: &Pubkey,
    destination: &Pubkey,
    label: &Option<String>,
) -> Result<Signature> {
    let mut remaining_accounts = Vec::with_capacity(4);
    let (freeze_pda, _) = find_freeze_pda(candy_guard_id, candy_machine_id, destination);
    remaining_accounts.push(AccountMeta {
        pubkey: freeze_pda,
        is_signer: false,
        is_writable: true,
    });
    remaining_accounts.push(AccountMeta {
        pubkey: program.payer(),
        is_signer: true,
        is_writable: false,
    });
    remaining_accounts.push(AccountMeta {
        pubkey: *destination,
        is_signer: false,
        is_writable: true,
    });
    remaining_accounts.push(AccountMeta {
        pubkey: system_program::id(),
        is_signer: false,
        is_writable: false,
    });

    let builder = program
        .request()
        .accounts(RouteAccount {
            candy_guard: *candy_guard_id,
            candy_machine: *candy_machine_id,
            payer: program.payer(),
        })
        .accounts(remaining_accounts)
        .args(Route {
            args: RouteArgs {
                data: vec![FreezeInstruction::UnlockFunds as u8],
                guard: GuardType::FreezeSolPayment,
            },
            label: label.to_owned(),
        });
    let sig = builder.send()?;

    Ok(sig)
}
