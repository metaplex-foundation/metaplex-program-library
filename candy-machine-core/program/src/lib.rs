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
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account
    ///   1. `[signer]` Candy Machine authority
    pub fn add_config_lines(
        ctx: Context<AddConfigLines>,
        index: u32,
        config_lines: Vec<ConfigLine>,
    ) -> Result<()> {
        instructions::add_config_lines(ctx, index, config_lines)
    }

    /// Initialize the candy machine account with the specified data.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account (must be pre-allocated but zero content)
    ///   1. `[writable]` Authority PDA (seeds `["candy_machine", candy machine id]`)
    ///   2. `[]` Candy Machine authority
    ///   3. `[signer]` Payer
    ///   4. `[]` Collection metadata
    ///   5. `[]` Collection mint
    ///   6. `[]` Collection master edition
    ///   7. `[signer]` Collection update authority
    ///   8. `[writable]` Collection authority record
    ///   9. `[]` Token Metadata program
    ///   10. `[]` System program
    pub fn initialize(ctx: Context<Initialize>, data: CandyMachineData) -> Result<()> {
        instructions::initialize(ctx, data)
    }

    /// Initialize the candy machine account with the specified data and token standard.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account (must be pre-allocated but zero content)
    ///   1. `[writable]` Authority PDA (seeds `["candy_machine", candy machine id]`)
    ///   2. `[]` Candy Machine authority
    ///   3. `[signer]` Payer
    ///   4. `[]` Collection metadata
    ///   5. `[]` Collection mint
    ///   6. `[]` Collection master edition
    ///   7. `[signer]` Collection update authority
    ///   8. `[writable]` Metadata delegate record
    ///   9. `[]` Token Metadata program
    ///   10. `[]` System program
    ///   11. `[]` Instructions sysvar account
    ///   12. `[optional]` Token Authorization Rules program
    ///   13. `[optional]` Token authorization rules account
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
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account (must be pre-allocated but zero content)
    ///   1. `[writable]` Authority PDA (seeds `["candy_machine", candy machine id]`)
    ///   2. `[signer]` Candy Machine mint authority
    ///   3. `[signer]` Payer
    ///   4. `[writable]` Mint account of the NFT
    ///   5. `[signer]` Mint authority of the NFT
    ///   6. `[writable]` Metadata account of the NFT
    ///   7. `[writable]` Master edition account of the NFT
    ///   8. `[optional]` Collection authority record
    ///   9. `[]` Collection mint
    ///   10. `[writable]` Collection metadata
    ///   11. `[]` Collection master edition
    ///   12. `[]` Collection update authority
    ///   13. `[]` Token Metadata program
    ///   14. `[]` SPL Token program
    ///   15. `[]` System program
    ///   16. `[]` SlotHashes sysvar cluster data.
    pub fn mint<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, Mint<'info>>) -> Result<()> {
        instructions::mint(ctx)
    }

    /// Mint an NFT.
    ///
    /// Only the candy machine mint authority is allowed to mint. This handler mints both
    /// NFTs and Programmable NFTs.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account (must be pre-allocated but zero content)
    ///   1. `[writable]` Authority PDA (seeds `["candy_machine", candy machine id]`)
    ///   2. `[signer]` Candy Machine mint authority
    ///   3. `[signer]` Payer
    ///   4. `[writable]` Mint account of the NFT
    ///   5. `[]` Mint authority of the NFT
    ///   6. `[writable]` Metadata account of the NFT
    ///   7. `[writable]` Master edition account of the NFT
    ///   8. `[optional, writable]` Destination token account
    ///   9. `[optional, writable]` Token record
    ///   10. `[optional]` Collection authority record
    ///   11. `[]` Collection mint
    ///   12. `[writable]` Collection metadata
    ///   13. `[]` Collection master edition
    ///   14. `[]` Collection update authority
    ///   15. `[]` Token Metadata program
    ///   16. `[]` SPL Token program
    ///   17. `[optional]` SPL Associated Token program
    ///   18. `[]` System program
    ///   19. `[]` Instructions sysvar account
    ///   20. `[]` SlotHashes sysvar cluster data.
    pub fn mint_v2<'info>(ctx: Context<'_, '_, '_, 'info, MintV2<'info>>) -> Result<()> {
        instructions::mint_v2(ctx)
    }

    /// Set a new authority of the candy machine.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account
    ///   1. `[signer]` Candy Machine authority
    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        instructions::set_authority(ctx, new_authority)
    }

    /// Set the collection mint for the candy machine.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account (must be pre-allocated but zero content)
    ///   1. `[signer]` Candy Machine authority
    ///   2. `[]` Authority PDA (seeds `["candy_machine", candy machine id]`)
    ///   3. `[signer]` Payer
    ///   4. `[]` Collection mint
    ///   5. `[]` Collection metadata
    ///   6. `[writable]` Collection authority record
    ///   7. `[signer]` New collection update authority
    ///   8. `[]` Collection metadata
    ///   9. `[]` Collection mint
    ///   10. `[]` New collection master edition
    ///   11. `[]` New collection authority record
    ///   12. `[]` Token Metadata program
    ///   13. `[]` System program
    pub fn set_collection(ctx: Context<SetCollection>) -> Result<()> {
        instructions::set_collection(ctx)
    }

    /// Set the collection mint for the candy machine.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account (must be pre-allocated but zero content)
    ///   1. `[signer]` Candy Machine authority
    ///   2. `[]` Authority PDA (seeds `["candy_machine", candy machine id]`)
    ///   3. `[signer]` Payer
    ///   4. `[]` Collection update authority
    ///   5. `[]` Collection mint
    ///   6. `[]` Collection metadata
    ///   7. `[optional, writable]` Metadata delegate record
    ///   8. `[optional, writable]` Collection authority record
    ///   9. `[signer]` New collection update authority
    ///   10. `[]` New collection mint
    ///   11. `[]` New collection metadata
    ///   12. `[]` New collection master edition
    ///   13. `[writable]` New metadata delegate record
    ///   14. `[]` Token Metadata program
    ///   15. `[]` System program
    ///   16. `[]` Instructions sysvar account
    ///   17. `[optional]` Token Authorization Rules program
    ///   18. `[optional]` Token authorization rules account
    pub fn set_collection_v2(ctx: Context<SetCollectionV2>) -> Result<()> {
        instructions::set_collection_v2(ctx)
    }

    /// Set a new mint authority of the candy machine.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account
    ///   1. `[signer]` Candy Machine authority
    ///   1. `[signer]` New candy machine authority
    pub fn set_mint_authority(ctx: Context<SetMintAuthority>) -> Result<()> {
        instructions::set_mint_authority(ctx)
    }

    /// Set the token standard of the minted NFTs.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account (must be pre-allocated but zero content)
    ///   1. `[signer]` Candy Machine authority
    ///   2. `[]` Authority PDA (seeds `["candy_machine", candy machine id]`)
    ///   3. `[signer]` Payer
    ///   4. `[optional, writable]` Metadata delegate record
    ///   5. `[]` Collection mint
    ///   6. `[]` Collection metadata
    ///   7. `[optional, writable]` Collection authority record
    ///   8. `[]` Collection update authority
    ///   9. `[]` Token Metadata program
    ///   10. `[]` System program
    ///   11. `[]` Instructions sysvar account
    ///   12. `[optional]` Token Authorization Rules program
    ///   13. `[optional]` Token authorization rules account
    pub fn set_token_standard(ctx: Context<SetTokenStandard>, token_standard: u8) -> Result<()> {
        instructions::set_token_standard(ctx, token_standard)
    }

    /// Update the candy machine configuration.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account
    ///   1. `[signer]` Candy Machine authority
    pub fn update(ctx: Context<Update>, data: CandyMachineData) -> Result<()> {
        instructions::update(ctx, data)
    }

    /// Withdraw the rent lamports and send them to the authority address.
    ///
    /// # Accounts
    ///
    ///   0. `[writable]` Candy Machine account
    ///   1. `[signer]` Candy Machine authority
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }
}
