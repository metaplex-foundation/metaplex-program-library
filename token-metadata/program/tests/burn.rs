#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    instruction::{builders::BurnBuilder, BurnArgs, InstructionBuilder},
    state::{Creator, Key, TokenStandard},
};
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use spl_token::state::Account as TokenAccount;
use utils::*;

mod burn_assets {

    use super::*;

    #[tokio::test]
    async fn burn_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let mut da = DigitalAsset::new();
        da.create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let args = BurnArgs::V1 {
            authorization_data: None,
            amount: 1,
        };

        da.burn(&mut context, args).await.unwrap();

        // Assert that metadata, edition and token account are closed.
        da.assert_burned(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn burn_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 {
            authorization_data: None,
            amount: 1,
        };

        da.burn(&mut context, args).await.unwrap();

        // Assert that metadata, edition and token account are closed.
        da.assert_burned(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn burn_nonfungible_edition() {
        let mut context = program_test().start_with_context().await;

        let nft = Metadata::new();
        let nft_master_edition = MasterEditionV2::new(&nft);
        let nft_edition_marker = EditionMarker::new(&nft, &nft_master_edition, 1);

        let payer_key = context.payer.pubkey();

        nft.create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            Some(vec![Creator {
                address: payer_key,
                verified: true,
                share: 100,
            }]),
            10,
            false,
            0,
        )
        .await
        .unwrap();

        nft_master_edition
            .create(&mut context, Some(10))
            .await
            .unwrap();

        nft_edition_marker.create(&mut context).await.unwrap();

        let edition_marker = nft_edition_marker.get_data(&mut context).await;
        let print_edition = get_account(&mut context, &nft_edition_marker.new_edition_pubkey).await;

        assert_eq!(edition_marker.ledger[0], 64);
        assert_eq!(edition_marker.key, Key::EditionMarker);
        assert_eq!(print_edition.data[0], 1);

        let args = BurnArgs::V1 {
            authorization_data: None,
            amount: 1,
        };

        let mut builder = BurnBuilder::new();
        builder
            .owner(context.payer.pubkey())
            .metadata(nft_edition_marker.new_metadata_pubkey)
            .edition(nft_edition_marker.new_edition_pubkey)
            .mint(nft_edition_marker.mint.pubkey())
            .token(nft_edition_marker.token.pubkey())
            .parent_mint(nft.mint.pubkey())
            .parent_token(nft.token.pubkey())
            .parent_edition(nft_master_edition.pubkey)
            .edition_marker(nft_edition_marker.pubkey);

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // Metadata, and token account are burned.
        let print_md = context
            .banks_client
            .get_account(nft_edition_marker.new_metadata_pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(nft_edition_marker.token.pubkey())
            .await
            .unwrap();
        let print_edition_account = context
            .banks_client
            .get_account(nft_edition_marker.new_edition_pubkey)
            .await
            .unwrap();

        assert!(print_md.is_none());
        assert!(token_account.is_none());
        assert!(print_edition_account.is_none());
    }

    #[tokio::test]
    async fn burn_fungible() {
        let mut context = program_test().start_with_context().await;

        let initial_amount = 10;
        let burn_amount = 1;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::Fungible,
            None,
            None,
            initial_amount,
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 {
            authorization_data: None,
            amount: burn_amount,
        };

        da.burn(&mut context, args).await.unwrap();

        // We only burned one token, so the token account should still exist.
        let token_account = context
            .banks_client
            .get_account(da.token.unwrap())
            .await
            .unwrap()
            .unwrap();

        let token = TokenAccount::unpack(&token_account.data).unwrap();

        assert_eq!(token.amount, initial_amount - burn_amount);

        let burn_remaining = initial_amount - burn_amount;

        let args = BurnArgs::V1 {
            authorization_data: None,
            amount: burn_remaining,
        };

        da.burn(&mut context, args).await.unwrap();

        // The token account should be closed now.
        let token_account = context
            .banks_client
            .get_account(da.token.unwrap())
            .await
            .unwrap();

        assert!(token_account.is_none());
    }
}

mod burn_pnft {}
