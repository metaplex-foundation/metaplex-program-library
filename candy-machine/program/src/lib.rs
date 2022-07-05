pub mod constants;
pub mod errors;
pub mod processor;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
pub use errors::CandyError;
pub use processor::*;
pub use state::*;
pub use utils::*;
declare_id!("cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ");

#[program]
pub mod candy_machine {

    use super::*;

    pub fn initialize_candy_machine(
        ctx: Context<InitializeCandyMachine>,
        data: CandyMachineData,
    ) -> Result<()> {
        handle_initialize_candy_machine(ctx, data)
    }

    pub fn update_candy_machine(
        ctx: Context<UpdateCandyMachine>,
        data: CandyMachineData,
    ) -> Result<()> {
        handle_update_candy_machine(ctx, data)
    }

    pub fn update_authority(
        ctx: Context<UpdateCandyMachine>,
        new_authority: Option<Pubkey>,
    ) -> Result<()> {
        handle_update_authority(ctx, new_authority)
    }

    pub fn add_config_lines(
        ctx: Context<AddConfigLines>,
        index: u32,
        config_lines: Vec<ConfigLine>,
    ) -> Result<()> {
        handle_add_config_lines(ctx, index, config_lines)
    }

    pub fn set_collection(ctx: Context<SetCollection>) -> Result<()> {
        handle_set_collection(ctx)
    }

    pub fn remove_collection(ctx: Context<RemoveCollection>) -> Result<()> {
        handle_remove_collection(ctx)
    }

    pub fn mint_nft<'info>(
        ctx: Context<'_, '_, '_, 'info, MintNFT<'info>>,
        creator_bump: u8,
    ) -> Result<()> {
        handle_mint_nft(ctx, creator_bump)
    }

    pub fn set_collection_during_mint(ctx: Context<SetCollectionDuringMint>) -> Result<()> {
        handle_set_collection_during_mint(ctx)
    }

    pub fn withdraw_funds<'info>(ctx: Context<WithdrawFunds<'info>>) -> Result<()> {
        handle_withdraw_funds(ctx)
    }
}
