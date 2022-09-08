use anchor_lang::prelude::*;

use errors::CandyError;
use instructions::*;
pub use state::*;
pub use utils::*;

pub mod constants;
mod errors;
mod instructions;
mod state;
mod utils;

declare_id!("cndy3CZK71ZHMp9ddpq5NVvQDx33o6cCYDf4JBAWCk7");

#[program]
pub mod candy_machine_core {
    use super::*;

    /// Add the configuration (name + uri) of each NFT to the account data.
    pub fn add_config_lines(
        ctx: Context<AddConfigLines>,
        index: u32,
        config_lines: Vec<ConfigLine>,
    ) -> Result<()> {
        instructions::add_config_lines(ctx, index, config_lines)
    }

    /// Initialize the candy machine account with the specified data.
    pub fn initialize(ctx: Context<Initialize>, data: CandyMachineData) -> Result<()> {
        instructions::initialize(ctx, data)
    }

    /// Mint an NFT. Only the candy machine authority is allowed to mint.
    pub fn mint<'info>(
        ctx: Context<'_, '_, '_, 'info, Mint<'info>>,
        creator_bump: u8,
    ) -> Result<()> {
        instructions::mint(ctx, creator_bump)
    }

    /// Set the collection mint for the candy machine.
    pub fn set_collection(ctx: Context<SetCollection>) -> Result<()> {
        instructions::set_collection(ctx)
    }

    /// Set a new authority of the candy machine. Changing the authority has the
    /// effect of changing who can mint.
    pub fn set_authority(
        ctx: Context<SetAuthority>,
        new_authority: Pubkey,
        new_update_authority: Pubkey,
    ) -> Result<()> {
        instructions::set_authority(ctx, new_authority, new_update_authority)
    }

    /// Update the candy machine configuration.
    pub fn update(ctx: Context<Update>, data: CandyMachineData) -> Result<()> {
        instructions::update(ctx, data)
    }

    /// Withdraw the rent lamports and send them to the authority address.
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }
}
