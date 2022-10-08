use anchor_client::{solana_sdk::pubkey::Pubkey, ClientError};
use anyhow::{anyhow, Result};
pub use mpl_candy_machine_core::ID as CANDY_MACHINE_ID;
use mpl_candy_machine_core::{CandyMachine, CandyMachineData};

use crate::{config::data::SugarConfig, setup::setup_client};

// To test a custom candy machine program, comment the mpl_candy_machine::ID line
// above and use the following lines to declare the id to use:
//
//use solana_program::declare_id;
//declare_id!("<YOUR CANDY MACHINE ID>");
//pub use self::ID as CANDY_MACHINE_ID;

#[derive(Debug)]
pub struct ConfigStatus {
    pub index: u32,
    pub on_chain: bool,
}

pub fn get_candy_machine_state(
    sugar_config: &SugarConfig,
    candy_machine_id: &Pubkey,
) -> Result<CandyMachine> {
    let client = setup_client(sugar_config)?;
    let program = client.program(CANDY_MACHINE_ID);

    program.account(*candy_machine_id).map_err(|e| match e {
        ClientError::AccountNotFound => anyhow!("Candy Machine does not exist!"),
        _ => anyhow!(
            "Failed to deserialize Candy Machine account {}: {}",
            candy_machine_id.to_string(),
            e
        ),
    })
}

pub fn get_candy_machine_data(
    sugar_config: &SugarConfig,
    candy_machine_id: &Pubkey,
) -> Result<CandyMachineData> {
    let candy_machine = get_candy_machine_state(sugar_config, candy_machine_id)?;
    Ok(candy_machine.data)
}
