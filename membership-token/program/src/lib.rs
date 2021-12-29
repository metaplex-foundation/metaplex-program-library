use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};

pub const STRING_DEFAULT_SIZE: usize = 20;

pub const HOLDER_PREFIX: &str = "holder";
pub const HISTORY_PREFIX: &str = "history";
pub const VAULT_OWNER_PREFIX: &str = "mt_vault";

/// Return `treasury_owner` Pubkey and bump seed.
pub fn find_treasury_owner_address(
    treasury_mint: &Pubkey,
    selling_resource: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            HOLDER_PREFIX.as_bytes(),
            treasury_mint.as_ref(),
            selling_resource.as_ref(),
        ],
        &id(),
    )
}

/// Return `vault_owner` Pubkey and bump seed.
pub fn find_vault_owner_address(resource: &Pubkey, store: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            VAULT_OWNER_PREFIX.as_bytes(),
            resource.as_ref(),
            store.as_ref(),
        ],
        &id(),
    )
}

/// Return `TradeHistory` Pubkey and bump seed.
pub fn find_trade_history_address(wallet: &Pubkey, market: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[HISTORY_PREFIX.as_bytes(), wallet.as_ref(), market.as_ref()],
        &id(),
    )
}

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod membership_token {

    use super::*;

    pub fn create_store<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateStore<'info>>,
        name: String,
        description: String,
    ) -> ProgramResult {
        let admin = &ctx.accounts.admin;
        let store = &mut ctx.accounts.store;

        if !admin.to_account_info().is_signer || !store.to_account_info().is_signer {
            return Err(ErrorCode::NoValidSignerPresent.into());
        }

        if name.len() > STRING_DEFAULT_SIZE || description.len() > STRING_DEFAULT_SIZE {
            return Err(ErrorCode::StringIsTooLong.into());
        }

        store.admin = admin.key();
        store.name = name;
        store.description = description;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, description: String)]
pub struct CreateStore<'info> {
    #[account(mut)]
    admin: Signer<'info>,
    #[account(init, space=Store::LEN, payer=admin)]
    store: Account<'info, Store>,
    system_program: Program<'info, System>,
}

#[account]
pub struct Store {
    pub admin: Pubkey,
    pub name: String,
    pub description: String,
}

impl Store {
    pub const LEN: usize = 32 + STRING_DEFAULT_SIZE * 4 + STRING_DEFAULT_SIZE * 4;
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub enum SellingResourceState {
    Uninitialized,
    Created,
    InUse,
    Exhausted,
    Stopped,
}

#[account]
pub struct SellingResource {
    pub store: Pubkey,
    pub owner: Pubkey,
    pub resource: Pubkey,
    pub vault: Pubkey,
    pub vault_owner: Pubkey,
    pub supply: u64,
    pub max_supply: Option<u64>,
    pub state: SellingResourceState,
}

impl SellingResource {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 32 + 8 + 9 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum MarketState {
    Uninitialized,
    Created,
    Active,
    Ended,
}

#[account]
pub struct Market {
    pub store: Pubkey,
    pub selling_resource: Pubkey,
    pub treasury_mint: Pubkey,
    pub treasury_holder: Pubkey,
    pub treasury_owner: Pubkey,
    pub owner: Pubkey,
    pub name: String,
    pub description: String,
    pub mutable: bool,
    pub price: u64,
    pub pieces_in_one_wallet: Option<u64>,
    pub start_date: u64,
    pub end_date: Option<u64>,
    pub state: MarketState,
}

impl Market {
    pub const LEN: usize = 32
        + 32
        + 32
        + 32
        + 32
        + 32
        + STRING_DEFAULT_SIZE * 4
        + STRING_DEFAULT_SIZE * 4
        + 1
        + 8
        + 9
        + 8
        + 9
        + 1;
}

#[account]
pub struct TradeHistory {
    pub market: Pubkey,
    pub wallet: Pubkey,
    pub already_bought: u64,
}

impl TradeHistory {
    pub const LEN: usize = 32 + 32 + 8;
}

#[error]
pub enum ErrorCode {
    #[msg("No valid signer present")]
    NoValidSignerPresent,
    #[msg("Some string variable is longer than allowed")]
    StringIsTooLong,
}
