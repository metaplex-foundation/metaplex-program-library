pub mod constants;
pub mod errors;
pub mod pda;
pub mod reward_center;

use crate::reward_center::*;

use anchor_lang::prelude::*;

declare_id!("rwdLstiU8aJU1DPdoPtocaNKApMhCFdCg283hz8dd3u");

#[program]
pub mod listing_rewards {
    use super::*;

    pub fn create_reward_center(
        ctx: Context<CreateRewardCenter>,
        reward_center_params: CreateRewardCenterParams,
    ) -> Result<()> {
        reward_center::create_reward_center(ctx, reward_center_params)
    }
}
