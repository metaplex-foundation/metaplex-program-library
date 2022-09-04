pub mod error;
pub mod processor;
pub mod state;
pub mod utils;

use crate::{
    error::ErrorCode,
    state::{
        Creator, GatingConfig, Market, PayoutTicket, PrimaryMetadataCreators, SellingResource,
        Store, TradeHistory,
    },
    utils::*,
};
use anchor_lang::{prelude::*, system_program::System, AnchorDeserialize, AnchorSerialize};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

declare_id!("SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo");

#[program]
pub mod fixed_price_sale {
    use super::*;

    pub fn init_selling_resource<'info>(
        ctx: Context<'_, '_, '_, 'info, InitSellingResource<'info>>,
        master_edition_bump: u8,
        vault_owner_bump: u8,
        max_supply: Option<u64>,
    ) -> Result<()> {
        ctx.accounts
            .process(master_edition_bump, vault_owner_bump, max_supply)
    }

    pub fn create_store<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateStore<'info>>,
        name: String,
        description: String,
    ) -> Result<()> {
        ctx.accounts.process(name, description)
    }

    pub fn buy<'info>(
        ctx: Context<'_, '_, '_, 'info, Buy<'info>>,
        _trade_history_bump: u8,
        vault_owner_bump: u8,
    ) -> Result<()> {
        ctx.accounts.process(
            _trade_history_bump,
            vault_owner_bump,
            ctx.remaining_accounts,
        )
    }

    pub fn close_market<'info>(ctx: Context<'_, '_, '_, 'info, CloseMarket<'info>>) -> Result<()> {
        ctx.accounts.process()
    }

    pub fn suspend_market<'info>(
        ctx: Context<'_, '_, '_, 'info, SuspendMarket<'info>>,
    ) -> Result<()> {
        ctx.accounts.process()
    }

    pub fn change_market<'info>(
        ctx: Context<'_, '_, '_, 'info, ChangeMarket<'info>>,
        new_name: Option<String>,
        new_description: Option<String>,
        mutable: Option<bool>,
        new_price: Option<u64>,
        new_pieces_in_one_wallet: Option<u64>,
    ) -> Result<()> {
        ctx.accounts.process(
            new_name,
            new_description,
            mutable,
            new_price,
            new_pieces_in_one_wallet,
        )
    }

    pub fn resume_market<'info>(
        ctx: Context<'_, '_, '_, 'info, ResumeMarket<'info>>,
    ) -> Result<()> {
        ctx.accounts.process()
    }

    pub fn withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>,
        treasury_owner_bump: u8,
        payout_ticket_bump: u8,
    ) -> Result<()> {
        ctx.accounts.process(
            treasury_owner_bump,
            payout_ticket_bump,
            ctx.remaining_accounts,
        )
    }

    pub fn create_market<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateMarket<'info>>,
        _treasury_owner_bump: u8,
        name: String,
        description: String,
        mutable: bool,
        price: u64,
        pieces_in_one_wallet: Option<u64>,
        start_date: u64,
        end_date: Option<u64>,
        gating_config: Option<GatingConfig>,
    ) -> Result<()> {
        ctx.accounts.process(
            _treasury_owner_bump,
            name,
            description,
            mutable,
            price,
            pieces_in_one_wallet,
            start_date,
            end_date,
            gating_config,
            ctx.remaining_accounts,
        )
    }

    pub fn claim_resource<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimResource<'info>>,
        vault_owner_bump: u8,
    ) -> Result<()> {
        ctx.accounts.process(vault_owner_bump)
    }

    pub fn save_primary_metadata_creators<'info>(
        ctx: Context<'_, '_, '_, 'info, SavePrimaryMetadataCreators<'info>>,
        primary_metadata_creators_bump: u8,
        creators: Vec<Creator>,
    ) -> Result<()> {
        ctx.accounts
            .process(primary_metadata_creators_bump, creators)
    }
}

#[derive(Accounts)]
#[instruction(name: String, description: String)]
pub struct CreateStore<'info> {
    #[account(mut)]
    admin: Signer<'info>,
    #[account(init, space=Store::LEN, payer=admin)]
    store: Box<Account<'info, Store>>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(master_edition_bump:u8, vault_owner_bump: u8, max_supply: Option<u64>)]
pub struct InitSellingResource<'info> {
    #[account(has_one=admin)]
    store: Box<Account<'info, Store>>,
    #[account(mut)]
    admin: Signer<'info>,
    #[account(init, payer=admin, space=SellingResource::LEN)]
    selling_resource: Box<Account<'info, SellingResource>>,
    /// CHECK: checked in program
    selling_resource_owner: UncheckedAccount<'info>,
    resource_mint: Box<Account<'info, Mint>>,
    #[account(owner=mpl_token_metadata::id())]
    /// CHECK: checked in program
    master_edition: UncheckedAccount<'info>,
    #[account(owner=mpl_token_metadata::id())]
    /// CHECK: checked in program
    metadata: UncheckedAccount<'info>,
    #[account(mut, has_one=owner)]
    vault: Box<Account<'info, TokenAccount>>,
    #[account(seeds=[VAULT_OWNER_PREFIX.as_bytes(), resource_mint.key().as_ref(), store.key().as_ref()], bump=vault_owner_bump)]
    /// CHECK: checked in program
    owner: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in program
    resource_token: UncheckedAccount<'info>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(treasury_owner_bump: u8, name: String, description: String, mutable: bool, price: u64, pieces_in_one_wallet: Option<u64>, start_date: u64, end_date: Option<u64>, gating_config: Option<GatingConfig>)]
pub struct CreateMarket<'info> {
    #[account(init, space=Market::LEN, payer=selling_resource_owner)]
    market: Box<Account<'info, Market>>,
    store: Box<Account<'info, Store>>,
    #[account(mut)]
    selling_resource_owner: Signer<'info>,
    #[account(mut, has_one=store)]
    selling_resource: Box<Account<'info, SellingResource>>,
    /// CHECK: checked in program
    mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in program
    treasury_holder: UncheckedAccount<'info>,
    #[account(seeds=[HOLDER_PREFIX.as_bytes(), mint.key().as_ref(), selling_resource.key().as_ref()], bump=treasury_owner_bump)]
    /// CHECK: checked in program
    owner: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    // if gating config is set collection mint key should be passed
    // collection_mint: Account<'info, Mint>
}

#[derive(Accounts)]
#[instruction(trade_history:u8, vault_owner_bump: u8)]
pub struct Buy<'info> {
    #[account(mut, has_one=treasury_holder, has_one=selling_resource)]
    market: Box<Account<'info, Market>>,
    #[account(mut)]
    selling_resource: Box<Account<'info, SellingResource>>,
    #[account(mut)]
    /// CHECK: checked in program
    user_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    user_wallet: Signer<'info>,
    #[account(init_if_needed, seeds=[HISTORY_PREFIX.as_bytes(), user_wallet.key().as_ref(), market.key().as_ref()], bump, payer=user_wallet, space=TradeHistory::LEN)]
    trade_history: Box<Account<'info, TradeHistory>>,
    #[account(mut)]
    /// CHECK: checked in program
    treasury_holder: UncheckedAccount<'info>,
    // Will be created by `mpl_token_metadata`
    #[account(mut)]
    /// CHECK: checked in program
    new_metadata: UncheckedAccount<'info>,
    // Will be created by `mpl_token_metadata`
    #[account(mut)]
    /// CHECK: checked in program
    new_edition: UncheckedAccount<'info>,
    #[account(mut, owner=mpl_token_metadata::id())]
    /// CHECK: checked in program
    master_edition: UncheckedAccount<'info>,
    #[account(mut)]
    new_mint: Box<Account<'info, Mint>>,
    // Will be created by `mpl_token_metadata`
    #[account(mut)]
    /// CHECK: checked in program
    edition_marker: UncheckedAccount<'info>,
    #[account(mut, has_one=owner)]
    vault: Box<Account<'info, TokenAccount>>,
    #[account(seeds=[VAULT_OWNER_PREFIX.as_bytes(), selling_resource.resource.as_ref(), selling_resource.store.as_ref()], bump=vault_owner_bump)]
    /// CHECK: checked in program
    owner: UncheckedAccount<'info>,
    #[account(mut, constraint = new_token_account.owner == user_wallet.key())]
    new_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, owner=mpl_token_metadata::id())]
    /// CHECK: checked in program
    master_edition_metadata: UncheckedAccount<'info>,
    clock: Sysvar<'info, Clock>,
    rent: Sysvar<'info, Rent>,
    /// CHECK: checked in program
    token_metadata_program: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    // if gatekeeper set for the collection these accounts also should be passed
    // IMPORTANT: accounts should be passed strictly in this order
    // user_collection_token_account: Account<'info, TokenAccount>
    // token_account_mint: Account<'info, Mint>
    // metadata_account: UncheckedAccount<'info>
}

#[derive(Accounts)]
#[instruction(treasury_owner_bump: u8, payout_ticket_bump: u8)]
pub struct Withdraw<'info> {
    #[account(has_one=treasury_holder, has_one=selling_resource, has_one=treasury_mint)]
    market: Box<Account<'info, Market>>,
    selling_resource: Box<Account<'info, SellingResource>>,
    #[account(owner=mpl_token_metadata::id())]
    /// CHECK: checked in program
    metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in program
    treasury_holder: UncheckedAccount<'info>,
    /// CHECK: checked in program
    treasury_mint: UncheckedAccount<'info>,
    #[account(seeds=[HOLDER_PREFIX.as_bytes(), market.treasury_mint.as_ref(), market.selling_resource.as_ref()], bump=treasury_owner_bump)]
    /// CHECK: checked in program
    owner: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in program
    destination: UncheckedAccount<'info>,
    /// CHECK: checked in program
    funder: UncheckedAccount<'info>,
    #[account(mut)]
    payer: Signer<'info>,
    #[account(init_if_needed, seeds=[PAYOUT_TICKET_PREFIX.as_bytes(), market.key().as_ref(), funder.key().as_ref()], bump, payer=payer, space=PayoutTicket::LEN)]
    payout_ticket: Box<Account<'info, PayoutTicket>>,
    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(vault_owner_bump: u8)]
pub struct ClaimResource<'info> {
    #[account(has_one=selling_resource, has_one=treasury_holder)]
    market: Account<'info, Market>,
    /// CHECK: checked in program
    treasury_holder: UncheckedAccount<'info>,
    #[account(has_one=vault, constraint = selling_resource.owner == selling_resource_owner.key())]
    selling_resource: Account<'info, SellingResource>,
    selling_resource_owner: Signer<'info>,
    #[account(mut, has_one=owner)]
    vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, owner=mpl_token_metadata::id())]
    /// CHECK: checked in program
    metadata: UncheckedAccount<'info>,
    #[account(seeds=[VAULT_OWNER_PREFIX.as_bytes(), selling_resource.resource.as_ref(), selling_resource.store.as_ref()], bump=vault_owner_bump)]
    /// CHECK: checked in program
    owner: UncheckedAccount<'info>,
    #[account(mut)]
    destination: Box<Account<'info, TokenAccount>>,
    clock: Sysvar<'info, Clock>,
    token_program: Program<'info, Token>,
    /// CHECK: checked in program
    token_metadata_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CloseMarket<'info> {
    #[account(mut, has_one=owner)]
    market: Account<'info, Market>,
    owner: Signer<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction()]
pub struct SuspendMarket<'info> {
    #[account(mut, has_one=owner)]
    market: Account<'info, Market>,
    owner: Signer<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction()]
pub struct ResumeMarket<'info> {
    #[account(mut, has_one=owner)]
    market: Account<'info, Market>,
    owner: Signer<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(new_name: Option<String>, new_description: Option<String>, mutable: Option<bool>, new_price: Option<u64>, new_pieces_in_one_wallet: Option<u64>)]
pub struct ChangeMarket<'info> {
    #[account(mut, has_one=owner)]
    market: Account<'info, Market>,
    owner: Signer<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(primary_metadata_creators: u8, creators: Vec<mpl_token_metadata::state::Creator>)]
pub struct SavePrimaryMetadataCreators<'info> {
    #[account(mut)]
    admin: Signer<'info>,
    #[account(mut, owner=mpl_token_metadata::id())]
    /// CHECK: checked in program
    metadata: UncheckedAccount<'info>,
    #[account(init, space=PrimaryMetadataCreators::LEN, payer=admin, seeds=[PRIMARY_METADATA_CREATORS_PREFIX.as_bytes(), metadata.key.as_ref()], bump)]
    primary_metadata_creators: Box<Account<'info, PrimaryMetadataCreators>>,
    system_program: Program<'info, System>,
}
