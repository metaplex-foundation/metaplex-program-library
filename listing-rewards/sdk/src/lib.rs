pub mod accounts;
pub mod args;

use accounts::{CreateOfferAccounts, SellAccounts};
use anchor_client::solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program, sysvar};
use anchor_lang::{prelude::*, InstructionData};
use args::{CreateOfferData, SellData};
use mpl_listing_rewards::{
    accounts as rewards_accounts, id, instruction, offers::create_offer::CreateOfferParams, pda,
    reward_center::CreateRewardCenterParams,
    rewardable_collection::CreateRewardableCollectionParams, sell::SellParams,
};
use spl_associated_token_account::get_associated_token_address;

pub fn create_reward_center(
    wallet: Pubkey,
    mint: Pubkey,
    auction_house: Pubkey,
    reward_center_params: CreateRewardCenterParams,
) -> Instruction {
    let (reward_center, _) = pda::find_reward_center_address(&auction_house);
    let associated_token_account = get_associated_token_address(&reward_center, &mint);

    let accounts = rewards_accounts::CreateRewardCenter {
        wallet,
        mint,
        auction_house,
        reward_center,
        associated_token_account,
        token_program: spl_token::id(),
        associated_token_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
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

pub fn create_rewardable_collection(
    wallet: Pubkey,
    auction_house: Pubkey,
    reward_center: Pubkey,
    collection: Pubkey,
) -> Instruction {
    let (rewardable_collection, _) =
        pda::find_rewardable_collection_address(&reward_center, &collection);

    let accounts = rewards_accounts::CreateRewardableCollection {
        wallet,
        reward_center,
        rewardable_collection,
        auction_house,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = instruction::CreateRewardableCollection {
        rewardable_collection_params: CreateRewardableCollectionParams { collection },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn sell(
    SellAccounts {
        wallet,
        listing,
        reward_center,
        rewardable_collection,
        token_account,
        metadata,
        authority,
        auction_house,
        seller_trade_state,
        free_seller_trade_state,
    }: SellAccounts,
    SellData {
        price,
        token_size,
        trade_state_bump,
        free_trade_state_bump,
    }: SellData,
) -> Instruction {
    let (auction_house_fee_account, _) =
        mpl_auction_house::pda::find_auction_house_fee_account_address(&auction_house);
    let (ah_auctioneer_pda, _) =
        mpl_auction_house::pda::find_auctioneer_pda(&auction_house, &reward_center);
    let (program_as_signer, program_as_signer_bump) =
        mpl_auction_house::pda::find_program_as_signer_address();

    let accounts = rewards_accounts::Sell {
        auction_house_program: mpl_auction_house::id(),
        listing,
        reward_center,
        rewardable_collection,
        wallet,
        token_account,
        metadata,
        authority,
        auction_house,
        auction_house_fee_account,
        seller_trade_state,
        free_seller_trade_state,
        ah_auctioneer_pda,
        program_as_signer,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = instruction::Sell {
        sell_params: SellParams {
            price,
            token_size,
            trade_state_bump,
            free_trade_state_bump,
            program_as_signer_bump,
        },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn create_offer(
    CreateOfferAccounts {
        auction_house,
        authority,
        buyer_trade_state,
        metadata,
        payment_account,
        reward_center,
        token_account,
        transfer_authority,
        treasury_mint,
        wallet,
        rewardable_collection,
        offer,
    }: CreateOfferAccounts,
    CreateOfferData {
        buyer_price,
        token_size,
        trade_state_bump,
    }: CreateOfferData,
) -> Instruction {
    let (auction_house_fee_account, _) =
        mpl_auction_house::pda::find_auction_house_fee_account_address(&auction_house);
    let (ah_auctioneer_pda, _) =
        mpl_auction_house::pda::find_auctioneer_pda(&auction_house, &reward_center);

    let (escrow_payment_account, escrow_payment_bump) =
        mpl_auction_house::pda::find_escrow_payment_address(&auction_house, &wallet);

    let accounts = rewards_accounts::CreateOffer {
        ah_auctioneer_pda,
        auction_house,
        auction_house_fee_account,
        authority,
        buyer_trade_state,
        metadata,
        payment_account,
        reward_center,
        token_account,
        transfer_authority,
        treasury_mint,
        escrow_payment_account,
        wallet,
        rewardable_collection,
        offer,
        auction_house_program: mpl_auction_house::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = instruction::CreateOffer {
        create_offer_params: CreateOfferParams {
            buyer_price,
            escrow_payment_bump,
            token_size,
            trade_state_bump,
        },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

// pub fn redeem_rewards() -> Instruction {
//     let accounts = accounts::RedeemRewards {
//         auction_house_program: mpl_auction_house::id(),
//         listing,
//         reward_center,
//         rewardable_collection,
//         wallet,
//         token_program: spl_token::id(),
//         system_program: system_program::id(),
//         rent: sysvar::rent::id(),
//     }
//     .to_account_metas(None);

//     let data = instruction::RedeemRewards {}.data();

//     Instruction {
//         program_id: id(),
//         accounts,
//         data,
//     }
// }
