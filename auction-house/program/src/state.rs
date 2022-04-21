use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};

use crate::constants::*;

#[account]
pub struct AuctionHouse {
    pub auction_house_fee_account: Pubkey,
    pub auction_house_treasury: Pubkey,
    pub treasury_withdrawal_destination: Pubkey,
    pub fee_withdrawal_destination: Pubkey,
    pub treasury_mint: Pubkey,
    pub authority: Pubkey,
    pub creator: Pubkey,
    pub bump: u8,
    pub treasury_bump: u8,
    pub fee_payer_bump: u8,
    pub seller_fee_basis_points: u16,
    pub requires_sign_off: bool,
    pub can_change_sale_price: bool,
    pub escrow_payment_bump: u8,
    pub has_auctioneer: bool,
    pub auctioneer_pda_bump: u8,
}

#[account]
pub struct Auctioneer {
    pub auctioneer_authority: Pubkey,
    pub auction_house: Pubkey,
    pub scopes: [bool; MAX_NUM_SCOPES],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
#[repr(u32)]
pub enum AuthorityScope {
    Deposit = 0,
    Buy = 1,
    PublicBuy = 2,
    ExecuteSale = 3,
    Sell = 4,
    Cancel = 5,
    Withdraw = 6,
}
