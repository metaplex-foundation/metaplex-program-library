pub mod errors;
pub mod instructions;
pub mod state;
pub mod token_metadata_utils;
pub mod token_utils;

use anchor_lang::prelude::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

// Program ID
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod fusion {
    use super::*;

    ///
    /// Create a new fusion formula with variable _ingredients_ and _output_items_
    /// 
    pub fn create_formula<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CreateFormula<'info>>,
        ingredients: Vec<Ingredient>,
        output_items: Vec<Item>,
        bump: u8, // Run `find_program_address` offchain for canonical bump
    ) -> ProgramResult {
        instructions::create_formula::handler(ctx, ingredients, output_items, bump)
    }

    ///
    /// Take ingredients for a formula and output the new tokens/NFTs
    /// 
    pub fn craft<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Craft<'info>>,
        bump: u8,
    ) -> ProgramResult {
        instructions::craft::handler(ctx, bump)
    }
}

/// Size: 32 + 1 + 1 = 34 bytes
#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Ingredient {
    /// Pubkey of the ingredient's token mint
    pub mint: Pubkey,
    /// Amount of the token required to satisy the creation of the ingredient
    pub amount: u8,
    /// Option that burns the ingredient when crafting
    pub burn_on_craft: bool,
}

/// Size: 32 + 1 + 1 + 32 = 66 bytes
#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Item {
    /// Pubkey of the item's token mint
    pub mint: Pubkey,
    /// Amount of the token that will be minted on craft
    pub amount: u8,
    /// Boolean indicating whether or not output mint is a MasterEdition
    pub is_master_edition: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum AuthorityType {
    /// Authority to mint new tokens
    MintTokens,
    /// Authority to freeze any account associated with the Mint
    FreezeAccount,
    /// Owner of a given token account
    AccountOwner,
    /// Authority to close a token account
    CloseAccount,
}

impl From<AuthorityType> for spl_token::instruction::AuthorityType {
    fn from(authority_ty: AuthorityType) -> spl_token::instruction::AuthorityType {
        match authority_ty {
            AuthorityType::MintTokens => spl_token::instruction::AuthorityType::MintTokens,
            AuthorityType::FreezeAccount => spl_token::instruction::AuthorityType::FreezeAccount,
            AuthorityType::AccountOwner => spl_token::instruction::AuthorityType::AccountOwner,
            AuthorityType::CloseAccount => spl_token::instruction::AuthorityType::CloseAccount,
        }
    }
}
