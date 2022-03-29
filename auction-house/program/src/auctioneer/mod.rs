use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};

pub mod buy;
pub mod sell;
pub use buy::*;
pub use sell::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Auctioneer {
    pub auctioneer_program: Pubkey,
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
