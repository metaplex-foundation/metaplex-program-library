use anchor_lang::prelude::*;

use crate::{constants::*, id};

pub fn find_reward_center_address(auction_house: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REWARD_CENTER.as_bytes(), auction_house.as_ref()], &id())
}

pub fn find_purchase_ticket_address(listing: &Pubkey, offer: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PURCHASE_TICKET.as_bytes(), listing.as_ref(), offer.as_ref()],
        &id(),
    )
}

pub fn find_listing_address(
    seller: &Pubkey,
    metadata: &Pubkey,
    reward_center: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            LISTING.as_bytes(),
            seller.as_ref(),
            metadata.as_ref(),
            reward_center.as_ref(),
        ],
        &id(),
    )
}

pub fn find_offer_address(
    buyer: &Pubkey,
    metadata: &Pubkey,
    reward_center: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            OFFER.as_bytes(),
            buyer.as_ref(),
            metadata.as_ref(),
            reward_center.as_ref(),
        ],
        &id(),
    )
}
