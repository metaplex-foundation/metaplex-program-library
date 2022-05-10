#![cfg(feature = "test-bpf")]
pub mod common;
pub mod utils;

use common::*;
use utils::{helpers::default_scopes, setup_functions::*};

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use mpl_auction_house::pda::find_auctioneer_pda;
use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_sdk::signer::Signer;

use std::{assert_eq, time::SystemTime};

use solana_program::{instruction::Instruction, system_program, sysvar};

use solana_program::program_pack::Pack;

use mpl_auction_house::pda::{
    find_escrow_payment_address, find_program_as_signer_address, find_trade_state_address,
};
use solana_sdk::{clock::UnixTimestamp, signature::Keypair, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account;

#[tokio::test]
async fn execute_sale_early_failure() {
    let mut context = auctioneer_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, authority) = existing_auction_house_test_context(&mut context)
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
        )
        .await
        .unwrap();
    let ((sell_acc, listing_config_address, _), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        100_000_000,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 30) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
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
    let ((bid_acc, _), buy_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &sell_acc.wallet,
        &listing_config_address,
        1_000_000_000,
    );
    let result = context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap_err();
    assert_error!(result, AUCTION_NOT_STARTED);
    let buyer_token_account =
        get_associated_token_address(&buyer.pubkey(), &test_metadata.mint.pubkey());

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &mpl_auctioneer::id());
    let accounts = mpl_auctioneer::accounts::AuctioneerExecuteSale {
        auction_house_program: mpl_auction_house::id(),
        listing_config: listing_config_address,
        buyer: buyer.pubkey(),
        seller: test_metadata.token.pubkey(),
        auction_house: ahkey,
        metadata: test_metadata.pubkey,
        token_account: sell_acc.token_account,
        authority: ah.authority,
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
        treasury_mint: ah.treasury_mint,
        program_as_signer: sell_acc.program_as_signer,
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
        auctioneer_authority: mpl_auctioneer::id(),
        ah_auctioneer_pda: auctioneer_pda,
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
        program_id: mpl_auctioneer::id(),
        data: mpl_auctioneer::instruction::ExecuteSale {
            escrow_payment_bump: escrow_bump,
            free_trade_state_bump: free_sts_bump,
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

    let early_tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&authority.pubkey()),
        &[&authority],
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
    assert_eq!(buyer_token_before.is_none(), true);

    let result = context
        .banks_client
        .process_transaction(early_tx)
        .await
        .unwrap_err();
    assert_error!(result, AUCTION_NOT_STARTED);

    // let seller_after = context
    //     .banks_client
    //     .get_account(test_metadata.token.pubkey())
    //     .await
    //     .unwrap()
    //     .unwrap();
    // let buyer_token_after = Account::unpack_from_slice(
    //     &context
    //         .banks_client
    //         .get_account(buyer_token_account)
    //         .await
    //         .unwrap()
    //         .unwrap()
    //         .data
    //         .as_slice(),
    // )
    // .unwrap();
    // let fee_minus: u64 = 100_000_000 - ((ah.seller_fee_basis_points as u64 * 100_000_000) / 10000);
    // assert_eq!(seller_before.lamports + fee_minus, seller_after.lamports);
    // assert_eq!(seller_before.lamports < seller_after.lamports, true);
    // assert_eq!(buyer_token_after.amount, 1);
}

#[tokio::test]
async fn execute_sale_success() {
    let mut context = auctioneer_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, authority) = existing_auction_house_test_context(&mut context)
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
        )
        .await
        .unwrap();
    let ((sell_acc, listing_config_address, _), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        100_000_000,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
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
    let ((bid_acc, _), buy_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &sell_acc.wallet,
        &listing_config_address,
        100_000_000,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();
    let buyer_token_account =
        get_associated_token_address(&buyer.pubkey(), &test_metadata.mint.pubkey());

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &mpl_auctioneer::id());
    let accounts = mpl_auctioneer::accounts::AuctioneerExecuteSale {
        auction_house_program: mpl_auction_house::id(),
        listing_config: listing_config_address,
        buyer: buyer.pubkey(),
        seller: test_metadata.token.pubkey(),
        auction_house: ahkey,
        metadata: test_metadata.pubkey,
        token_account: sell_acc.token_account,
        authority: ah.authority,
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
        treasury_mint: ah.treasury_mint,
        program_as_signer: sell_acc.program_as_signer,
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
        auctioneer_authority: mpl_auctioneer::id(),
        ah_auctioneer_pda: auctioneer_pda,
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
        program_id: mpl_auctioneer::id(),
        data: mpl_auctioneer::instruction::ExecuteSale {
            escrow_payment_bump: escrow_bump,
            free_trade_state_bump: free_sts_bump,
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
        Some(&authority.pubkey()),
        &[&authority],
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
    assert_eq!(buyer_token_before.is_none(), true);

    let result = context.banks_client.process_transaction(tx).await.unwrap();

    let seller_after = context
        .banks_client
        .get_account(test_metadata.token.pubkey())
        .await
        .unwrap()
        .unwrap();
    let buyer_token_after = Account::unpack_from_slice(
        &context
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
    assert_eq!(seller_before.lamports < seller_after.lamports, true);
    assert_eq!(buyer_token_after.amount, 1);
}

#[tokio::test]
async fn execute_sale_late_failure() {
    let mut context = auctioneer_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, authority) = existing_auction_house_test_context(&mut context)
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
        )
        .await
        .unwrap();
    let ((sell_acc, listing_config_address, _), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        100_000_000,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 120) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
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
    let ((bid_acc, _), buy_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer,
        &sell_acc.wallet,
        &listing_config_address,
        1_000_000_000,
    );
    let result = context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap_err();
    assert_error!(result, AUCTION_ENDED);
    let buyer_token_account =
        get_associated_token_address(&buyer.pubkey(), &test_metadata.mint.pubkey());

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &mpl_auctioneer::id());
    let accounts = mpl_auctioneer::accounts::AuctioneerExecuteSale {
        auction_house_program: mpl_auction_house::id(),
        listing_config: listing_config_address,
        buyer: buyer.pubkey(),
        seller: test_metadata.token.pubkey(),
        auction_house: ahkey,
        metadata: test_metadata.pubkey,
        token_account: sell_acc.token_account,
        authority: ah.authority,
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
        treasury_mint: ah.treasury_mint,
        program_as_signer: sell_acc.program_as_signer,
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
        auctioneer_authority: mpl_auctioneer::id(),
        ah_auctioneer_pda: auctioneer_pda,
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
        program_id: mpl_auctioneer::id(),
        data: mpl_auctioneer::instruction::ExecuteSale {
            escrow_payment_bump: escrow_bump,
            free_trade_state_bump: free_sts_bump,
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

    let early_tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&authority.pubkey()),
        &[&authority],
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
    assert_eq!(buyer_token_before.is_none(), true);

    let result = context
        .banks_client
        .process_transaction(early_tx)
        .await
        .unwrap_err();
    assert_error!(result, AUCTION_ENDED);

    // let seller_after = context
    //     .banks_client
    //     .get_account(test_metadata.token.pubkey())
    //     .await
    //     .unwrap()
    //     .unwrap();
    // let buyer_token_after = Account::unpack_from_slice(
    //     &context
    //         .banks_client
    //         .get_account(buyer_token_account)
    //         .await
    //         .unwrap()
    //         .unwrap()
    //         .data
    //         .as_slice(),
    // )
    // .unwrap();
    // let fee_minus: u64 = 100_000_000 - ((ah.seller_fee_basis_points as u64 * 100_000_000) / 10000);
    // assert_eq!(seller_before.lamports + fee_minus, seller_after.lamports);
    // assert_eq!(seller_before.lamports < seller_after.lamports, true);
    // assert_eq!(buyer_token_after.amount, 1);
}

#[tokio::test]
async fn execute_sale_two_bids_success() {
    let mut context = auctioneer_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, authority) = existing_auction_house_test_context(&mut context)
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
        )
        .await
        .unwrap();
    let ((sell_acc, listing_config_address, _), sell_tx) = sell(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        100_000_000,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
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
    let ((bid0_acc, _), buy0_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer0,
        &sell_acc.wallet,
        &listing_config_address,
        100_000_000,
    );
    context
        .banks_client
        .process_transaction(buy0_tx)
        .await
        .unwrap();
    let buyer0_token_account =
        get_associated_token_address(&buyer0.pubkey(), &test_metadata.mint.pubkey());

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 10_000_000_000)
        .await
        .unwrap();
    let ((bid1_acc, _), buy1_tx) = buy(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        &test_metadata.token.pubkey(),
        &buyer1,
        &sell_acc.wallet,
        &listing_config_address,
        100_000_001,
    );
    context
        .banks_client
        .process_transaction(buy1_tx)
        .await
        .unwrap();
    let buyer1_token_account =
        get_associated_token_address(&buyer1.pubkey(), &test_metadata.mint.pubkey());

    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &mpl_auctioneer::id());
    let accounts = mpl_auctioneer::accounts::AuctioneerExecuteSale {
        auction_house_program: mpl_auction_house::id(),
        listing_config: listing_config_address,
        buyer: buyer1.pubkey(),
        seller: test_metadata.token.pubkey(),
        auction_house: ahkey,
        metadata: test_metadata.pubkey,
        token_account: sell_acc.token_account,
        authority: ah.authority,
        seller_trade_state: sell_acc.seller_trade_state,
        buyer_trade_state: bid1_acc.buyer_trade_state,
        token_program: spl_token::id(),
        free_trade_state: sell_acc.free_seller_trade_state,
        seller_payment_receipt_account: test_metadata.token.pubkey(),
        buyer_receipt_token_account: buyer1_token_account,
        escrow_payment_account: bid1_acc.escrow_payment_account,
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
        auction_house_treasury: ah.auction_house_treasury,
        treasury_mint: ah.treasury_mint,
        program_as_signer: sell_acc.program_as_signer,
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
        auctioneer_authority: mpl_auctioneer::id(),
        ah_auctioneer_pda: auctioneer_pda,
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
    let (_, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer1.pubkey());
    let (_, pas_bump) = find_program_as_signer_address();

    let instruction = Instruction {
        program_id: mpl_auctioneer::id(),
        data: mpl_auctioneer::instruction::ExecuteSale {
            escrow_payment_bump: escrow_bump,
            free_trade_state_bump: free_sts_bump,
            program_as_signer_bump: pas_bump,
            token_size: 1,
            buyer_price: 100_000_001,
        }
        .data(),
        accounts,
    };
    airdrop(&mut context, &ah.auction_house_fee_account, 10_000_000_000)
        .await
        .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&authority.pubkey()),
        &[&authority],
        context.last_blockhash,
    );
    let seller_before = context
        .banks_client
        .get_account(test_metadata.token.pubkey())
        .await
        .unwrap()
        .unwrap();
    let buyer1_token_before = &context
        .banks_client
        .get_account(buyer1_token_account)
        .await
        .unwrap();
    assert_eq!(buyer1_token_before.is_none(), true);

    let result = context.banks_client.process_transaction(tx).await.unwrap();

    let seller_after = context
        .banks_client
        .get_account(test_metadata.token.pubkey())
        .await
        .unwrap()
        .unwrap();
    let buyer1_token_after = Account::unpack_from_slice(
        &context
            .banks_client
            .get_account(buyer1_token_account)
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();
    let fee_minus: u64 = 100_000_001 - ((ah.seller_fee_basis_points as u64 * 100_000_000) / 10000);
    assert_eq!(seller_before.lamports + fee_minus, seller_after.lamports);
    assert_eq!(seller_before.lamports < seller_after.lamports, true);
    assert_eq!(buyer1_token_after.amount, 1);
}
