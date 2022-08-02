#![cfg(feature = "test-bpf")]

pub mod common;
pub mod utils;

use common::*;
use utils::{helpers::default_scopes, setup_functions::*};

use anchor_lang::{InstructionData, ToAccountMetas};
use mpl_auction_house::pda::find_auctioneer_pda;
use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_sdk::signer::Signer;

use std::assert_eq;

use solana_program::{
    instruction::{Instruction, InstructionError},
    system_program, sysvar,
};

use mpl_auction_house::pda::{
    find_escrow_payment_address, find_program_as_signer_address, find_trade_state_address,
};
use solana_program::program_pack::Pack;
use solana_sdk::{
    signature::Keypair,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account;

#[tokio::test]
async fn auctioneer_execute_sale_success() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    airdrop(&mut context, &auctioneer_authority.pubkey(), ONE_SOL)
        .await
        .unwrap();

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        default_scopes(),
    )
    .await
    .unwrap();

    let (sell_acc, sell_tx) = auctioneer_sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &auctioneer_authority,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let (bid_acc, buy_tx) = auctioneer_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &auctioneer_authority,
        100_000_000,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
    let buyer_token_account =
        get_associated_token_address(&buyer.pubkey(), &test_metadata.mint.pubkey());

    let accounts = mpl_auction_house::accounts::AuctioneerExecuteSale {
        buyer: buyer.pubkey(),
        seller: test_metadata.token.pubkey(),
        auction_house: ahkey,
        metadata: test_metadata.pubkey,
        token_account: sell_acc.token_account,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        seller_trade_state: sell_acc.seller_trade_state,
        buyer_trade_state: bid_acc.buyer_trade_state,
        token_program: spl_token::id(),
        free_trade_state: sell_acc.free_seller_trade_state,
        seller_payment_receipt_account: test_metadata.token.pubkey(),
        buyer_receipt_token_account: buyer_token_account,
        escrow_payment_account: bid_acc.escrow_payment_account,
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
        auction_house_treasury: ah.auction_house_treasury,
        ah_auctioneer_pda: auctioneer_pda,
        treasury_mint: ah.treasury_mint,
        program_as_signer: sell_acc.program_as_signer,
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);
    let (_, free_sts_bump) = find_trade_state_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &sell_acc.token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        0,
        1,
    );
    let (_, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer.pubkey());
    let (_, pas_bump) = find_program_as_signer_address();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerExecuteSale {
            escrow_payment_bump: escrow_bump,
            _free_trade_state_bump: free_sts_bump,
            program_as_signer_bump: pas_bump,
            token_size: 1,
            buyer_price: 100_000_000,
        }
        .data(),
        accounts,
    };
    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
        .await
        .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ah_auth.pubkey()),
        &[&ah_auth, &auctioneer_authority],
        context.last_blockhash,
    );
    let seller_before = context
        .banks_client
        .get_account(test_metadata.token.pubkey())
        .await
        .unwrap()
        .unwrap();
    let buyer_token_before = &context
        .banks_client
        .get_account(buyer_token_account)
        .await
        .unwrap();
    assert!(buyer_token_before.is_none());
    context.banks_client.process_transaction(tx).await.unwrap();

    let seller_after = context
        .banks_client
        .get_account(test_metadata.token.pubkey())
        .await
        .unwrap()
        .unwrap();
    let buyer_token_after = Account::unpack_from_slice(
        context
            .banks_client
            .get_account(buyer_token_account)
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();
    let fee_minus: u64 = 100_000_000 - ((ah.seller_fee_basis_points as u64 * 100_000_000) / 10000);
    assert_eq!(seller_before.lamports + fee_minus, seller_after.lamports);
    assert!(seller_before.lamports < seller_after.lamports);
    assert_eq!(buyer_token_after.amount, 1);
}

#[tokio::test]
async fn auctioneer_execute_sale_bad_trade_state_failure() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    airdrop(&mut context, &auctioneer_authority.pubkey(), ONE_SOL)
        .await
        .unwrap();

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        default_scopes(),
    )
    .await
    .unwrap();

    let (sell_acc, sell_tx) = auctioneer_sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &auctioneer_authority,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer0 = Keypair::new();
    airdrop(&mut context, &buyer0.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let (bid_acc0, buy_tx0) = auctioneer_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer0,
        &auctioneer_authority,
        100_000_000,
    );
    context
        .banks_client
        .process_transaction(buy_tx0)
        .await
        .unwrap();
    let buyer0_token_account =
        get_associated_token_address(&buyer0.pubkey(), &test_metadata.mint.pubkey());

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let (bid_acc1, buy_tx1) = auctioneer_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer1,
        &auctioneer_authority,
        100_000_000,
    );
    context
        .banks_client
        .process_transaction(buy_tx1)
        .await
        .unwrap();

    let accounts = mpl_auction_house::accounts::AuctioneerExecuteSale {
        buyer: buyer0.pubkey(),
        seller: test_metadata.token.pubkey(),
        auction_house: ahkey,
        metadata: test_metadata.pubkey,
        token_account: sell_acc.token_account,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        seller_trade_state: bid_acc1.buyer_trade_state,
        buyer_trade_state: bid_acc0.buyer_trade_state,
        token_program: spl_token::id(),
        free_trade_state: sell_acc.free_seller_trade_state,
        seller_payment_receipt_account: test_metadata.token.pubkey(),
        buyer_receipt_token_account: buyer0_token_account,
        escrow_payment_account: bid_acc0.escrow_payment_account,
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
        auction_house_treasury: ah.auction_house_treasury,
        ah_auctioneer_pda: auctioneer_pda,
        treasury_mint: ah.treasury_mint,
        program_as_signer: sell_acc.program_as_signer,
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);
    let (_, free_sts_bump) = find_trade_state_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &sell_acc.token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        0,
        1,
    );
    let (_, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer0.pubkey());
    let (_, pas_bump) = find_program_as_signer_address();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerExecuteSale {
            escrow_payment_bump: escrow_bump,
            _free_trade_state_bump: free_sts_bump,
            program_as_signer_bump: pas_bump,
            token_size: 1,
            buyer_price: 100_000_000,
        }
        .data(),
        accounts,
    };
    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
        .await
        .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ah_auth.pubkey()),
        &[&ah_auth, &auctioneer_authority],
        context.last_blockhash,
    );
    let result = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_error!(result, INVALID_SEEDS);
}

#[tokio::test]
async fn auctioneer_execute_sale_missing_scope_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    let scopes = vec![AuthorityScope::Sell, AuthorityScope::Buy];

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        scopes.clone(),
    )
    .await
    .unwrap();

    let (sell_acc, sell_tx) = auctioneer_sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &auctioneer_authority,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let (bid_acc, buy_tx) = auctioneer_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &auctioneer_authority,
        100_000_000,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();

    let buyer_token_account =
        get_associated_token_address(&buyer.pubkey(), &test_metadata.mint.pubkey());

    let accounts = mpl_auction_house::accounts::AuctioneerExecuteSale {
        buyer: buyer.pubkey(),
        seller: test_metadata.token.pubkey(),
        auction_house: ahkey,
        metadata: test_metadata.pubkey,
        token_account: sell_acc.token_account,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        seller_trade_state: sell_acc.seller_trade_state,
        buyer_trade_state: bid_acc.buyer_trade_state,
        token_program: spl_token::id(),
        free_trade_state: sell_acc.free_seller_trade_state,
        seller_payment_receipt_account: test_metadata.token.pubkey(),
        buyer_receipt_token_account: buyer_token_account,
        escrow_payment_account: bid_acc.escrow_payment_account,
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
        auction_house_treasury: ah.auction_house_treasury,
        ah_auctioneer_pda: auctioneer_pda,
        treasury_mint: ah.treasury_mint,
        program_as_signer: sell_acc.program_as_signer,
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);
    let (_, free_sts_bump) = find_trade_state_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &sell_acc.token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        0,
        1,
    );
    let (_, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer.pubkey());
    let (_, pas_bump) = find_program_as_signer_address();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerExecuteSale {
            escrow_payment_bump: escrow_bump,
            _free_trade_state_bump: free_sts_bump,
            program_as_signer_bump: pas_bump,
            token_size: 1,
            buyer_price: 100_000_000,
        }
        .data(),
        accounts,
    };
    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
        .await
        .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ah_auth.pubkey()),
        &[&ah_auth, &auctioneer_authority],
        context.last_blockhash,
    );
    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    assert_error!(error, MISSING_AUCTIONEER_SCOPE);
}

#[tokio::test]
pub async fn auctioneer_execute_sale_no_delegate_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    let ((sell_acc, _), sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, 100_000_000, 1);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let ((bid_acc, _), buy_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        100_000_000,
        1,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
    let buyer_token_account =
        get_associated_token_address(&buyer.pubkey(), &test_metadata.mint.pubkey());

    let accounts = mpl_auction_house::accounts::AuctioneerExecuteSale {
        buyer: buyer.pubkey(),
        seller: test_metadata.token.pubkey(),
        auction_house: ahkey,
        metadata: test_metadata.pubkey,
        token_account: sell_acc.token_account,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        seller_trade_state: sell_acc.seller_trade_state,
        buyer_trade_state: bid_acc.buyer_trade_state,
        token_program: spl_token::id(),
        free_trade_state: sell_acc.free_seller_trade_state,
        seller_payment_receipt_account: test_metadata.token.pubkey(),
        buyer_receipt_token_account: buyer_token_account,
        escrow_payment_account: bid_acc.escrow_payment_account,
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
        auction_house_treasury: ah.auction_house_treasury,
        ah_auctioneer_pda: auctioneer_pda,
        treasury_mint: ah.treasury_mint,
        program_as_signer: sell_acc.program_as_signer,
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);
    let (_, free_sts_bump) = find_trade_state_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &sell_acc.token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        0,
        1,
    );
    let (_, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer.pubkey());
    let (_, pas_bump) = find_program_as_signer_address();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerExecuteSale {
            escrow_payment_bump: escrow_bump,
            _free_trade_state_bump: free_sts_bump,
            program_as_signer_bump: pas_bump,
            token_size: 1,
            buyer_price: 100_000_000,
        }
        .data(),
        accounts,
    };
    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
        .await
        .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ah_auth.pubkey()),
        &[&ah_auth, &auctioneer_authority],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error!(error, ACCOUNT_NOT_INITIALIZED);
}

#[tokio::test]
async fn auctioneer_execute_public_sale_success() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    airdrop(&mut context, &auctioneer_authority.pubkey(), ONE_SOL * 2)
        .await
        .unwrap();

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        default_scopes(),
    )
    .await
    .unwrap();

    let price = 100_000_000;
    let fee_minus: u64 = price - ((ah.seller_fee_basis_points as u64 * 100_000_000) / 10000);
    // Create Listing
    let (sell_acc, sell_tx) = auctioneer_sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &auctioneer_authority,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();
    // Create Long Lasting Bid
    let public_bidder = Keypair::new();
    airdrop(&mut context, &public_bidder.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let (public_bid_acc, public_bid_tx) = auctioneer_public_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &public_bidder,
        &auctioneer_authority,
        price,
    );
    context
        .banks_client
        .process_transaction(public_bid_tx)
        .await
        .unwrap();
    // Create first regular private bid
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    let (bid_acc, buy_tx) = auctioneer_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &auctioneer_authority,
        price,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
    let buyer_token_account =
        get_associated_token_address(&buyer.pubkey(), &test_metadata.mint.pubkey());
    let (_es_acc, first_sale_tx) = auctioneer_execute_sale(
        &mut context,
        &ahkey,
        &ah,
        &ah_auth,
        &auctioneer_authority,
        &test_metadata,
        &buyer.pubkey(),
        &test_metadata.token.pubkey(),
        &sell_acc.token_account,
        &sell_acc.seller_trade_state,
        &bid_acc.buyer_trade_state,
        1,
        price,
    );
    airdrop(&mut context, &ah.auction_house_fee_account, ONE_SOL * 10)
        .await
        .unwrap();

    let seller_before = context
        .banks_client
        .get_account(test_metadata.token.pubkey())
        .await
        .unwrap()
        .unwrap();
    let buyer_token_before = &context
        .banks_client
        .get_account(buyer_token_account)
        .await
        .unwrap();

    assert!(buyer_token_before.is_none());
    context
        .banks_client
        .process_transaction(first_sale_tx)
        .await
        .unwrap();

    let seller_after = context
        .banks_client
        .get_account(test_metadata.token.pubkey())
        .await
        .unwrap()
        .unwrap();
    let buyer_token_after = Account::unpack_from_slice(
        context
            .banks_client
            .get_account(buyer_token_account)
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();

    assert_eq!(seller_before.lamports + fee_minus, seller_after.lamports);
    assert!(seller_before.lamports < seller_after.lamports);
    assert_eq!(buyer_token_after.amount, 1);
    let new_seller = buyer;
    let public_bidder_token_account =
        get_associated_token_address(&public_bidder.pubkey(), &test_metadata.mint.pubkey());
    let new_seller_before = context
        .banks_client
        .get_account(new_seller.pubkey())
        .await
        .unwrap()
        .unwrap();
    let public_bidder_token_before = &context
        .banks_client
        .get_account(public_bidder_token_account)
        .await
        .unwrap();
    assert!(public_bidder_token_before.is_none());
    let (second_sell_acc, second_sell_tx) = auctioneer_sell_mint(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata.mint.pubkey(),
        &new_seller,
        &auctioneer_authority,
    );
    context
        .banks_client
        .process_transaction(second_sell_tx)
        .await
        .unwrap();

    let (_public_sale_acc, public_sale_tx) = auctioneer_execute_sale(
        &mut context,
        &ahkey,
        &ah,
        &ah_auth,
        &auctioneer_authority,
        &test_metadata,
        &public_bidder.pubkey(),
        &new_seller.pubkey(),
        &second_sell_acc.token_account,
        &second_sell_acc.seller_trade_state,
        &public_bid_acc.buyer_trade_state,
        1,
        price.to_owned(),
    );

    context
        .banks_client
        .process_transaction(public_sale_tx)
        .await
        .unwrap();

    let new_seller_after = context
        .banks_client
        .get_account(new_seller.pubkey())
        .await
        .unwrap()
        .unwrap();
    let public_bidder_token_after = Account::unpack_from_slice(
        context
            .banks_client
            .get_account(public_bidder_token_account)
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();
    assert!(new_seller_before.lamports < new_seller_after.lamports);
    assert_eq!(public_bidder_token_after.amount, 1);
}

#[tokio::test]
async fn auctioneer_execute_public_sale_missing_scope_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    airdrop(&mut context, &auctioneer_authority.pubkey(), ONE_SOL)
        .await
        .unwrap();

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    let scopes = vec![AuthorityScope::Sell, AuthorityScope::Buy];
    delegate_auctioneer(
        &mut context,
        ahkey,
        &ah_auth,
        auctioneer_authority.pubkey(),
        auctioneer_pda,
        scopes.clone(),
    )
    .await
    .unwrap();

    let price = 100_000_000;

    // Create Listing
    let (sell_acc, sell_tx) = auctioneer_sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &auctioneer_authority,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();
    // Create Long Lasting Bid
    let public_bidder = Keypair::new();
    airdrop(&mut context, &public_bidder.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let (_, public_bid_tx) = auctioneer_public_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &public_bidder,
        &auctioneer_authority,
        price,
    );
    context
        .banks_client
        .process_transaction(public_bid_tx)
        .await
        .unwrap();
    // Create first regular private bid
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    let (bid_acc, buy_tx) = auctioneer_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &auctioneer_authority,
        price,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
    let buyer_token_account =
        get_associated_token_address(&buyer.pubkey(), &test_metadata.mint.pubkey());
    let (_es_acc, first_sale_tx) = auctioneer_execute_sale(
        &mut context,
        &ahkey,
        &ah,
        &ah_auth,
        &auctioneer_authority,
        &test_metadata,
        &buyer.pubkey(),
        &test_metadata.token.pubkey(),
        &sell_acc.token_account,
        &sell_acc.seller_trade_state,
        &bid_acc.buyer_trade_state,
        1,
        price,
    );
    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
        .await
        .unwrap();
    let buyer_token_before = &context
        .banks_client
        .get_account(buyer_token_account)
        .await
        .unwrap();
    assert!(buyer_token_before.is_none());

    let error = context
        .banks_client
        .process_transaction(first_sale_tx)
        .await
        .unwrap_err();
    assert_error!(error, MISSING_AUCTIONEER_SCOPE);
}

#[tokio::test]
async fn auctioneer_execute_public_sale_no_delegate_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();

    // Delegate external auctioneer authority.
    let auctioneer_authority = Keypair::new();
    airdrop(&mut context, &auctioneer_authority.pubkey(), ONE_SOL)
        .await
        .unwrap();

    let price = 100_000_000;
    // Create Listing
    let ((sell_acc, _), sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, price, 1);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();
    // Create Long Lasting Bid
    let public_bidder = Keypair::new();
    airdrop(&mut context, &public_bidder.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let ((_acc, _print_bid_receipt_acc), public_bid_tx) = public_buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &public_bidder,
        price,
    );
    context
        .banks_client
        .process_transaction(public_bid_tx)
        .await
        .unwrap();

    // Create first regular private bid
    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    let ((bid_acc, _), buy_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        price,
        1,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
    let (_es_acc, first_sale_tx) = auctioneer_execute_sale(
        &mut context,
        &ahkey,
        &ah,
        &ah_auth,
        &auctioneer_authority,
        &test_metadata,
        &buyer.pubkey(),
        &test_metadata.token.pubkey(),
        &sell_acc.token_account,
        &sell_acc.seller_trade_state,
        &bid_acc.buyer_trade_state,
        1,
        price,
    );

    let error = context
        .banks_client
        .process_transaction(first_sale_tx)
        .await
        .unwrap_err();

    assert_error!(error, ACCOUNT_NOT_INITIALIZED);
}
