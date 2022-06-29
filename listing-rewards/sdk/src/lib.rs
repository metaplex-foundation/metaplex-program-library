use anchor_client::solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program};
use anchor_lang::{prelude::*, InstructionData};
use mpl_listing_rewards::{
    accounts, id, instruction, pda, reward_center::CreateRewardCenterParams,
};

pub fn create_reward_center(
    wallet: Pubkey,
    mint: Pubkey,
    auction_house: Pubkey,
    reward_center_params: CreateRewardCenterParams,
) -> Instruction {
    let (reward_center, _) = pda::find_reward_center_address(&auction_house);

    let accounts = accounts::CreateRewardCenter {
        wallet,
        mint,
        auction_house,
        reward_center,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = instruction::CreateRewardCenter {
        reward_center_params,
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}
