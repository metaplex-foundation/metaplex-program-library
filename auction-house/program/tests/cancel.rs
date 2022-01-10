#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::AccountDeserialize;

use mpl_auction_house::{pda::*, AuctionHouse};
use mpl_testing_utils::assert_error;
use mpl_testing_utils::solana::{airdrop, create_associated_token_account, create_mint};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, transaction::TransactionError, transport::TransportError,
};
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token;
use std::assert_eq;
use utils::setup_functions;
#[tokio::test]
async fn init_native_success() {
    let mut context = setup_functions::auction_house_program_test()
        .start_with_context()
        .await;
    // Payer Wallet
    let payer_wallet = Keypair::new();

    airdrop(&mut context, &payer_wallet.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    let twd_key = payer_wallet.pubkey();
    let fwd_key = payer_wallet.pubkey();
    let t_mint_key = spl_token::native_mint::id();
    let tdw_ata = twd_key;
    let seller_fee_basis_points: u16 = 100;
    let authority = Keypair::new().pubkey();
    // Derive Auction House Key
    let (auction_house_address, bump) = find_auction_house_address(&authority, &t_mint_key);
    let (auction_fee_account_key, fee_payer_bump) =
        find_auction_house_fee_account_address(&auction_house_address);
    // Derive Auction House Treasury Key
    let (auction_house_treasury_key, treasury_bump) =
        find_auction_house_treasury_address(&auction_house_address);
    let auction_house = setup_functions::create_auction_house(
        &mut context,
        &payer_wallet,
        &twd_key,
        &fwd_key,
        &t_mint_key,
        &tdw_ata,
        &authority,
        &auction_house_address,
        bump,
        &auction_fee_account_key,
        fee_payer_bump,
        &auction_house_treasury_key,
        treasury_bump,
        seller_fee_basis_points,
        false,
        false,
    );

    let auction_house_account = auction_house.await.unwrap();

    
}
