use super::constants::{AUCTION_HOUSE, FEE_PAYER, TREASURY};
use anchor_lang::prelude::Pubkey;

pub fn derive_auction_house_key(authority: &Pubkey, mint_key: &Pubkey) -> (Pubkey, u8) {
    let auction_house_seeds = &[
        AUCTION_HOUSE.as_bytes(),
        authority.as_ref(),
        mint_key.as_ref(),
    ];
    Pubkey::find_program_address(auction_house_seeds, &mpl_auction_house::id())
}

pub fn derive_auction_house_fee_account_key(auction_house_key: &Pubkey) -> (Pubkey, u8) {
    let auction_fee_account_seeds = &[
        AUCTION_HOUSE.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
    ];
    Pubkey::find_program_address(auction_fee_account_seeds, &mpl_auction_house::id())
}

pub fn derive_auction_house_treasury_key(auction_house_key: &Pubkey) -> (Pubkey, u8) {
    let auction_house_treasury_seeds = &[
        AUCTION_HOUSE.as_bytes(),
        auction_house_key.as_ref(),
        TREASURY.as_bytes(),
    ];
    Pubkey::find_program_address(auction_house_treasury_seeds, &mpl_auction_house::id())
}
