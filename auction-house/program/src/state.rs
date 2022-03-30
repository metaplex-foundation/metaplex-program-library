use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Auctioneer {
    pub auctioneer_authority: Pubkey,
    pub auction_house: Pubkey,
    pub scopes: Vec<AuthorityScope>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum AuthorityScope {
    Buy,
    PublicBuy,
    ExecuteSale,
    Sell,
    Cancel,
    Withdraw,
}
