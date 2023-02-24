#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

pub use errors::CandyError;
use instructions::*;
pub use state::*;
pub use utils::*;

pub mod constants;
pub mod errors;
mod instructions;
mod state;
mod utils;

declare_id!("CndyV3LdqHUfDLmE5naZjVN8rBZz4tqhdefbAnjHG3JR");

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

    /// Initialize the candy machine account with the specified data and token standard.
    pub fn initialize_v2(
        ctx: Context<InitializeV2>,
        data: CandyMachineData,
        token_standard: u8,
    ) -> Result<()> {
        instructions::initialize_v2(ctx, data, token_standard)
    }

    /// Mint an NFT.
    ///
    /// Only the candy machine mint authority is allowed to mint.
    pub fn mint<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, Mint<'info>>) -> Result<()> {
        instructions::mint(ctx)
    }

    /// Mint an NFT.
    ///
    /// Only the candy machine mint authority is allowed to mint. This handler mints both
    /// NFTs and Programmable NFts.
    pub fn mint_v2<'info>(ctx: Context<'_, '_, '_, 'info, MintV2<'info>>) -> Result<()> {
        instructions::mint_v2(ctx)
    }

    /// Set a new authority of the candy machine.
    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        instructions::set_authority(ctx, new_authority)
    }

    /// Set the collection mint for the candy machine.
    pub fn set_collection(ctx: Context<SetCollection>) -> Result<()> {
        instructions::set_collection(ctx)
    }

    /// Set the collection mint for the candy machine.
    pub fn set_collection_v2(ctx: Context<SetCollectionV2>) -> Result<()> {
        instructions::set_collection_v2(ctx)
    }

    /// Set a new mint authority of the candy machine.
    pub fn set_mint_authority(ctx: Context<SetMintAuthority>) -> Result<()> {
        instructions::set_mint_authority(ctx)
    }

    /// Set the token standard of the minted NFTs.
    pub fn set_token_standard(ctx: Context<SetTokenStandard>, token_standard: u8) -> Result<()> {
        instructions::set_token_standard(ctx, token_standard)
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
