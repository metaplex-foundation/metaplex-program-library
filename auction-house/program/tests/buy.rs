//! Module provide tests for `Buy` instruction.

mod utils;

mod buy {
    use super::utils::{
        clone_keypair, create_associated_token_account, create_mint,
        find_auction_house_fee_account_address, find_escrow_payment_address,
        find_trade_state_address,
        setup_functions::{setup_auction_house, setup_program},
    };
    use me
    use anchor_client::{
        solana_sdk::{
            commitment_config::CommitmentConfig, signature::Keypair, signer::Signer,
            system_program, sysvar,
        },
        Client, Cluster,
    };
    use solana_program_test::*;
    use std::error;

    #[test]
    fn success() -> Result<(), Box<dyn error::Error>> {
        const BUYER_PRICE: u64 = 1;
        const TOKEN_SIZE: u64 = 1;

        let mut context = auction_house_program_test().start_with_context().await;

        // Payer Wallet
        let payer_wallet = Keypair::new();

        airdrop(&mut context, &payer_wallet.pubkey(), 10_000_000_000).await;

        // Derive native(wrapped) sol mint
        let treasury_mint = spl_token::native_mint::id();

        // Token mint for `TokenMetadata`.
        let token_mint = create_mint(context, &wallet)?;

        // Derive / Create associated token account
        let token_account =
            create_associated_token_account(context, &wallet, &token_mint.pubkey())?;

        // Mint tokens
        mint_to(context, &wallet, &token_mint.pubkey(), &token_account, 1)?;

        // Setup AuctionHouse
        let auction_house = setup_auction_house(&program, &wallet_pubkey, &treasury_mint).unwrap();

        // Derive `AuctionHouse` fee account
        let (auction_house_fee_account, _) = find_auction_house_fee_account_address(&auction_house);

        // Derive buyer trade state address
        let (buyer_trade_state, trade_state_bump) = find_trade_state_address(
            &wallet_pubkey,
            &auction_house,
            &token_account,
            &treasury_mint,
            &token_mint.pubkey(),
            BUYER_PRICE,
            TOKEN_SIZE,
        );

        // Derive escrow payment address
        let (escrow_payment_account, escrow_payment_bump) =
            find_escrow_payment_address(&auction_house, &wallet_pubkey);

        // Create `TokenMetadata`
        let metadata = create_token_metadata(
            &connection,
            &wallet,
            &token_mint.pubkey(),
            String::from("TEST"),
            String::from("TST"),
            String::from("https://github.com"),
            5000,
        )?;

        // Transfer enough lamports to create seller trade state
        transfer_lamports(&connection, &wallet, &auction_house_fee_account, 10000000)?;

        // Perform RPC instruction request
        program
            .request()
            .accounts(mpl_auction_house::accounts::Buy {
                wallet: wallet_pubkey,
                payment_account: wallet_pubkey,
                transfer_authority: wallet_pubkey,
                treasury_mint,
                token_account,
                metadata,
                escrow_payment_account,
                authority: wallet_pubkey,
                auction_house,
                auction_house_fee_account,
                buyer_trade_state,
                token_program: spl_token::id(),
                system_program: system_program::id(),
                rent: sysvar::rent::id(),
            })
            .args(mpl_auction_house::instruction::Buy {
                trade_state_bump,
                escrow_payment_bump,
                buyer_price: BUYER_PRICE,
                token_size: TOKEN_SIZE,
            })
            .send()?;

        assert_eq!(
            connection.get_account_data(&buyer_trade_state)?[0],
            trade_state_bump
        );

        Ok(())
    }
}
