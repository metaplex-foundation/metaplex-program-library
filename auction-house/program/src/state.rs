use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};

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
    pub has_auctioneer: bool,
}

#[account]
pub struct Auctioneer {
    pub authority: Pubkey,
    pub auction_house: Pubkey,
    pub scopes: Vec<AuthorityScope>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum AuthorityScope {
    Buy,
    PublicBuy,
    ExecuteSale,
    Sell,
    Cancel,
    Withdraw,
}
