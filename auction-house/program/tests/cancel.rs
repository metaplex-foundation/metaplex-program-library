#![cfg(feature = "test-bpf")]
pub mod common;
pub mod utils;

use common::*;
use solana_sdk::sysvar;
use utils::{helpers::default_scopes, setup_functions::*};

#[tokio::test]
async fn cancel_listing() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(
        &mut context,
        &test_metadata.token.pubkey(),
        100_000_000_000_000,
    )
    .await
    .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();
    context.warp_to_slot(100).unwrap();
    // Derive Auction House Key
    let ((acc, _), sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, 10, 1);
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();
    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let accounts = mpl_auction_house::accounts::Cancel {
        auction_house: ahkey,
        wallet: test_metadata.token.pubkey(),
        token_account: token,
        authority: ah.authority,
        trade_state: acc.seller_trade_state,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::Cancel {
            buyer_price: 10,
            token_size: 1,
        }
        .data(),
        accounts,
    };

    let (listing_receipt, _) = find_listing_receipt_address(&acc.seller_trade_state);

    let accounts = mpl_auction_house::accounts::CancelListingReceipt {
        receipt: listing_receipt,
        system_program: solana_program::system_program::id(),
        instruction: sysvar::instructions::id(),
    }
    .to_account_metas(None);
    let cancel_listing_receipt_instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::CancelListingReceipt {}.data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction, cancel_listing_receipt_instruction],
        Some(&test_metadata.token.pubkey()),
        &[&test_metadata.token],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;

    let listing_receipt_account = context
        .banks_client
        .get_account(listing_receipt)
        .await
        .expect("getting listing receipt")
        .expect("empty listing receipt data");

    let listing_receipt =
        ListingReceipt::try_deserialize(&mut listing_receipt_account.data.as_ref()).unwrap();

    assert_eq!(listing_receipt.canceled_at, Some(timestamp));
    assert_eq!(listing_receipt.purchase_receipt, None);
}

#[tokio::test]
async fn auction_cancel_listing() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(
        &mut context,
        &test_metadata.token.pubkey(),
        100_000_000_000_000,
    )
    .await
    .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
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

    context.warp_to_slot(100).unwrap();
    // Derive Auction House Key
    let (acc, sell_tx) = auctioneer_sell(
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
    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let accounts = mpl_auction_house::accounts::AuctioneerCancel {
        auction_house: ahkey,
        wallet: test_metadata.token.pubkey(),
        token_account: token,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        trade_state: acc.seller_trade_state,
        ah_auctioneer_pda: auctioneer_pda,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerCancel {
            // NOTE: This needs to be the max value for canceling sales due to the way auctioneer handles sale values
            buyer_price: u64::MAX,
            token_size: 1,
        }
        .data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&test_metadata.token.pubkey()),
        &[&test_metadata.token, &auctioneer_authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}

#[tokio::test]
async fn auction_cancel_listing_missing_scope_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(
        &mut context,
        &test_metadata.token.pubkey(),
        100_000_000_000_000,
    )
    .await
    .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
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

    // Missing Cancel scope so auction_cancel should fail.
    let scopes = vec![AuthorityScope::Sell];

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

    context.warp_to_slot(100).unwrap();
    // Derive Auction House Key
    let (acc, sell_tx) = auctioneer_sell(
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
    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let accounts = mpl_auction_house::accounts::AuctioneerCancel {
        auction_house: ahkey,
        wallet: test_metadata.token.pubkey(),
        token_account: token,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        trade_state: acc.seller_trade_state,
        ah_auctioneer_pda: auctioneer_pda,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerCancel {
            buyer_price: 10,
            token_size: 1,
        }
        .data(),
        accounts,
    };

    let (listing_receipt, _) = find_listing_receipt_address(&acc.seller_trade_state);

    let accounts = mpl_auction_house::accounts::CancelListingReceipt {
        receipt: listing_receipt,
        system_program: solana_program::system_program::id(),
        instruction: sysvar::instructions::id(),
    }
    .to_account_metas(None);
    let cancel_listing_receipt_instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::CancelListingReceipt {}.data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction, cancel_listing_receipt_instruction],
        Some(&test_metadata.token.pubkey()),
        &[&test_metadata.token, &auctioneer_authority],
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
async fn auction_cancel_listing_no_delegate_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 1000000000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();

    let auctioneer_authority = Keypair::new();
    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority.pubkey());

    context.warp_to_slot(100).unwrap();
    let buyer = Keypair::new();
    let price = 1000000000;
    airdrop(&mut context, &buyer.pubkey(), 2000000000)
        .await
        .unwrap();

    let ((acc, _), sell_tx) = sell(&mut context, &ahkey, &ah, &test_metadata, 10, 1);

    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .unwrap();

    let accounts = mpl_auction_house::accounts::AuctioneerCancel {
        auction_house: ahkey,
        wallet: buyer.pubkey(),
        token_account: acc.token_account,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        trade_state: acc.seller_trade_state,
        ah_auctioneer_pda: auctioneer_pda,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerCancel {
            buyer_price: price,
            token_size: 1,
        }
        .data(),
        accounts,
    };

    let (bid_receipt, _) = find_bid_receipt_address(&acc.seller_trade_state);

    let accounts = mpl_auction_house::accounts::CancelBidReceipt {
        receipt: bid_receipt,
        system_program: solana_program::system_program::id(),
        instruction: sysvar::instructions::id(),
    }
    .to_account_metas(None);
    let cancel_bid_receipt_instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::CancelBidReceipt {}.data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction, cancel_bid_receipt_instruction],
        Some(&buyer.pubkey()),
        &[&buyer, &auctioneer_authority],
        context.last_blockhash,
    );
    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error!(error, INVALID_SEEDS);
}

#[tokio::test]
async fn cancel_bid() {
    let mut context = auction_house_program_test().start_with_context().await;
    // Payer Wallet
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();
    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), 1000000000)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            1,
        )
        .await
        .unwrap();
    context.warp_to_slot(100).unwrap();
    let buyer = Keypair::new();
    // Derive Auction House Key
    let price = 1000000000;
    airdrop(&mut context, &buyer.pubkey(), 2000000000)
        .await
        .unwrap();
    let ((acc, _), buy_tx) = buy(
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
    let accounts = mpl_auction_house::accounts::Cancel {
        auction_house: ahkey,
        wallet: buyer.pubkey(),
        token_account: acc.token_account,
        authority: ah.authority,
        trade_state: acc.buyer_trade_state,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::Cancel {
            buyer_price: price,
            token_size: 1,
        }
        .data(),
        accounts,
    };

    let (bid_receipt, _) = find_bid_receipt_address(&acc.buyer_trade_state);

    let accounts = mpl_auction_house::accounts::CancelBidReceipt {
        receipt: bid_receipt,
        system_program: solana_program::system_program::id(),
        instruction: sysvar::instructions::id(),
    }
    .to_account_metas(None);
    let cancel_bid_receipt_instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::CancelBidReceipt {}.data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction, cancel_bid_receipt_instruction],
        Some(&buyer.pubkey()),
        &[&buyer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;

    let bid_receipt_account = context
        .banks_client
        .get_account(bid_receipt)
        .await
        .expect("getting bid receipt")
        .expect("empty bid receipt data");

    let bid_receipt = BidReceipt::try_deserialize(&mut bid_receipt_account.data.as_ref()).unwrap();

    assert_eq!(bid_receipt.canceled_at, Some(timestamp));
    assert_eq!(bid_receipt.purchase_receipt, None);
}

#[tokio::test]
async fn auction_cancel_bid() {
    let mut context = auction_house_program_test().start_with_context().await;
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), ONE_SOL)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
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

    context.warp_to_slot(100).unwrap();
    let buyer = Keypair::new();
    let price = 1000000000;
    airdrop(&mut context, &buyer.pubkey(), 2000000000)
        .await
        .unwrap();

    let (acc, buy_tx) = auctioneer_buy(
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

    let accounts = mpl_auction_house::accounts::AuctioneerCancel {
        auction_house: ahkey,
        wallet: buyer.pubkey(),
        token_account: acc.token_account,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        trade_state: acc.buyer_trade_state,
        ah_auctioneer_pda: auctioneer_pda,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerCancel {
            buyer_price: price,
            token_size: 1,
        }
        .data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&buyer.pubkey()),
        &[&buyer, &auctioneer_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
}

#[tokio::test]
async fn auction_cancel_bid_missing_scope_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    let (ah, ahkey, ah_auth) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), ONE_SOL)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
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

    // Missing Cancel scope so auction_cancel should fail.
    let scopes = vec![AuthorityScope::Buy];

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

    context.warp_to_slot(100).unwrap();
    let buyer = Keypair::new();
    let price = ONE_SOL;
    airdrop(&mut context, &buyer.pubkey(), 2 * ONE_SOL)
        .await
        .unwrap();

    let (acc, buy_tx) = auctioneer_buy(
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

    let accounts = mpl_auction_house::accounts::AuctioneerCancel {
        auction_house: ahkey,
        wallet: buyer.pubkey(),
        token_account: acc.token_account,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        trade_state: acc.buyer_trade_state,
        ah_auctioneer_pda: auctioneer_pda,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerCancel {
            buyer_price: price,
            token_size: 1,
        }
        .data(),
        accounts,
    };

    let (bid_receipt, _) = find_bid_receipt_address(&acc.buyer_trade_state);

    let accounts = mpl_auction_house::accounts::CancelBidReceipt {
        receipt: bid_receipt,
        system_program: solana_program::system_program::id(),
        instruction: sysvar::instructions::id(),
    }
    .to_account_metas(None);
    let cancel_bid_receipt_instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::CancelBidReceipt {}.data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction, cancel_bid_receipt_instruction],
        Some(&buyer.pubkey()),
        &[&buyer, &auctioneer_authority],
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
async fn auction_cancel_bid_no_delegate_fails() {
    let mut context = auction_house_program_test().start_with_context().await;
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    let test_metadata = Metadata::new();
    airdrop(&mut context, &test_metadata.token.pubkey(), ONE_SOL)
        .await
        .unwrap();
    test_metadata
        .create(
            &mut context,
            "Tests".to_string(),
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

    context.warp_to_slot(100).unwrap();
    let buyer = Keypair::new();
    let price = ONE_SOL;
    airdrop(&mut context, &buyer.pubkey(), 2 * ONE_SOL)
        .await
        .unwrap();

    let ((acc, _), buy_tx) = buy(
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

    let accounts = mpl_auction_house::accounts::AuctioneerCancel {
        auction_house: ahkey,
        wallet: buyer.pubkey(),
        token_account: acc.token_account,
        authority: ah.authority,
        auctioneer_authority: auctioneer_authority.pubkey(),
        trade_state: acc.buyer_trade_state,
        ah_auctioneer_pda: auctioneer_pda,
        token_program: spl_token::id(),
        token_mint: test_metadata.mint.pubkey(),
        auction_house_fee_account: ah.auction_house_fee_account,
    }
    .to_account_metas(None);
    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::AuctioneerCancel {
            buyer_price: price,
            token_size: 1,
        }
        .data(),
        accounts,
    };

    let (bid_receipt, _) = find_bid_receipt_address(&acc.buyer_trade_state);

    let accounts = mpl_auction_house::accounts::CancelBidReceipt {
        receipt: bid_receipt,
        system_program: solana_program::system_program::id(),
        instruction: sysvar::instructions::id(),
    }
    .to_account_metas(None);
    let cancel_bid_receipt_instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::CancelBidReceipt {}.data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction, cancel_bid_receipt_instruction],
        Some(&buyer.pubkey()),
        &[&buyer, &auctioneer_authority],
        context.last_blockhash,
    );
    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error!(error, INVALID_SEEDS);
}
