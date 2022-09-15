use anchor_lang::prelude::*;

use crate::errors::ListingRewardsError;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct ListingRewardRules {
    // Basis Points to determine reward ratio for seller
    pub seller_reward_payout_basis_points: u16,
    // Payout Divider for determining reward distribution to seller/buyer
    pub payout_divider: u16,
}

#[account]
#[derive(Debug)]
pub struct RewardCenter {
    /// the mint of the token used as rewards
    pub token_mint: Pubkey,
    /// the auction house associated to the reward center
    pub auction_house: Pubkey,
    /// rules for listing rewards
    pub listing_reward_rules: ListingRewardRules,
    /// the bump of the pda
    pub bump: u8,
}

impl RewardCenter {
    pub fn size() -> usize {
        8 + // deliminator
        32 + // token_mint
        32 + // auction_house
        1 + 32 + // optional collection oracle
        2 + 2 + // listing reward rules
        1 // bump
    }

    // TODO: review the effects of decimals on the payouts. The math is clean when the currency token is the same as the reward token.
    pub fn payouts(&self, listing_price: u64) -> Result<(u64, u64)> {
        let total_token_payout = listing_price
            .checked_div(self.listing_reward_rules.payout_divider.into())
            .ok_or(ListingRewardsError::NumericalOverflowError)?
            as u64;

        let seller_share = self.listing_reward_rules.seller_reward_payout_basis_points;

        let seller_payout = (seller_share as u128)
            .checked_mul(total_token_payout as u128)
            .ok_or(ListingRewardsError::NumericalOverflowError)?
            .checked_div(10000)
            .ok_or(ListingRewardsError::NumericalOverflowError)? as u64;

        let buyer_payout = total_token_payout
            .checked_sub(seller_payout)
            .ok_or(ListingRewardsError::NumericalOverflowError)?;

        Ok((seller_payout, buyer_payout))
    }
}

#[account]
pub struct Listing {
    pub is_initialized: bool,
    pub reward_center: Pubkey,
    pub seller: Pubkey,
    pub metadata: Pubkey,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub created_at: i64,
    pub canceled_at: Option<i64>,
    pub purchase_ticket: Option<Pubkey>,
}

impl Listing {
    pub fn size() -> usize {
        8 + // delimiter
        1 + // is_initialized
        32 + // reward_center
        32 + // seller
        32 + // metadata
        8 + // price
        8 + // token_size
        1 + // bump
        8 + // created_at
        1 + 8 + // canceled_at
        1 + 32 // purchase_ticket
    }
}

#[account]
pub struct Offer {
    pub is_initialized: bool,
    pub reward_center: Pubkey,
    pub buyer: Pubkey,
    pub metadata: Pubkey,
    pub price: u64,
    pub token_size: u64,
    pub bump: u8,
    pub created_at: i64,
    pub canceled_at: Option<i64>,
    pub purchase_ticket: Option<Pubkey>,
}

impl Offer {
    pub fn size() -> usize {
        8 + // delimiter
        1 + // is_initialized
        32 + // reward_center
        32 + // buyer
        32 + // metadata
        8 + // price
        8 + // token_size
        1 + // bump
        8 + // created_at
        1 + 8 + // canceled_at
        1 + 32 // purchase_ticket
    }
}

#[account]
pub struct PurchaseTicket {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub metadata: Pubkey,
    pub reward_center: Pubkey,
    pub token_size: u64,
    pub price: u64,
    pub created_at: i64,
}

impl PurchaseTicket {
    pub fn size() -> usize {
        8 + // delimiter
        32 + // buyer
        32 + // seller
        32 + // metadata
        32 + // reward_center
        32 + // token_size
        8 + // price
        8 // created_at
    }
}
