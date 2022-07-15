use anchor_lang::prelude::*;

use crate::{constants::*, id};

pub fn find_reward_center_address(auction_house: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REWARD_CENTER.as_bytes(), auction_house.as_ref()], &id())
}

pub fn find_rewardable_collection_address(
    reward_center: &Pubkey,
    collection: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            REWARDABLE_COLLECTION.as_bytes(),
            reward_center.as_ref(),
            collection.as_ref(),
        ],
        &id(),
    )
}

pub fn find_listing_address(
    seller: &Pubkey,
    mint: &Pubkey,
    rewardable_collection: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            LISTING.as_bytes(),
            seller.as_ref(),
            mint.as_ref(),
            rewardable_collection.as_ref(),
        ],
        &id(),
    )
}
