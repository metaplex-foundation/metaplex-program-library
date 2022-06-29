use anchor_lang::prelude::*;

use crate::{constants::*, id};

pub fn find_reward_center_address(auction_house: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REWARD_CENTER.as_bytes(), auction_house.as_ref()], &id())
}
