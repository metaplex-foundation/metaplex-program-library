use anchor_lang::prelude::Pubkey;
use mpl_auction_house::constants::AUCTIONEER;

use crate::{constants::*, id};

pub fn find_listing_config_address(
    wallet: &Pubkey,
    auction_house: &Pubkey,
    token_account: &Pubkey,
    treasury_mint: &Pubkey,
    token_mint: &Pubkey,
    token_size: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            LISTING_CONFIG.as_bytes(),
            wallet.as_ref(),
            auction_house.as_ref(),
            token_account.as_ref(),
            treasury_mint.as_ref(),
            token_mint.as_ref(),
            &token_size.to_le_bytes(),
        ],
        &id(),
    )
}

pub fn find_auctioneer_authority_seeds(auction_house: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[AUCTIONEER.as_bytes(), auction_house.as_ref()], &id())
}
