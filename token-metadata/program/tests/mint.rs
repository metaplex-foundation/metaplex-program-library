#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use solana_sdk::{instruction::InstructionError, signature::Signer, transaction::TransactionError};
use utils::*;

mod mint {

    use mpl_token_metadata::{error::MetadataError, state::TokenStandard};
    use num_traits::FromPrimitive;
    use solana_program::program_pack::Pack;
    use spl_token::state::Account;

    use super::*;

    #[tokio::test]
    async fn mint_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create(&mut context, TokenStandard::ProgrammableNonFungible, None)
            .await
            .unwrap();

        // mints one token

        let payer_pubkey = context.payer.pubkey();

        asset.mint(&mut context, None, None, 1).await.unwrap();

        // asserts

        let account = get_account(&mut context, &asset.token.unwrap()).await;
        let token_account = Account::unpack(&account.data).unwrap();

        assert!(token_account.is_frozen());
        assert_eq!(token_account.amount, 1);
        assert_eq!(token_account.mint, asset.mint.pubkey());
        assert_eq!(token_account.owner, payer_pubkey);
    }

    #[tokio::test]
    async fn mint_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create(&mut context, TokenStandard::NonFungible, None)
            .await
            .unwrap();

        // mints one token

        asset.mint(&mut context, None, None, 1).await.unwrap();

        assert!(asset.token.is_some());

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(!token_account.is_frozen());
            assert_eq!(token_account.amount, 1);
            assert_eq!(token_account.mint, asset.mint.pubkey());
            assert_eq!(token_account.owner, context.payer.pubkey());
        }
    }

    #[tokio::test]
    async fn mint_fungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create(&mut context, TokenStandard::Fungible, None)
            .await
            .unwrap();

        // mints one token

        asset.mint(&mut context, None, None, 100).await.unwrap();

        assert!(asset.token.is_some());

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(!token_account.is_frozen());
            assert_eq!(token_account.amount, 100);
            assert_eq!(token_account.mint, asset.mint.pubkey());
            assert_eq!(token_account.owner, context.payer.pubkey());
        }
    }

    #[tokio::test]
    async fn mint_fungible_asset() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create(&mut context, TokenStandard::FungibleAsset, None)
            .await
            .unwrap();

        // mints one token

        asset.mint(&mut context, None, None, 50).await.unwrap();

        assert!(asset.token.is_some());

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(!token_account.is_frozen());
            assert_eq!(token_account.amount, 50);
            assert_eq!(token_account.mint, asset.mint.pubkey());
            assert_eq!(token_account.owner, context.payer.pubkey());
        }
    }

    #[tokio::test]
    async fn try_mint_multiple_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let mut asset = DigitalAsset::default();
        let error = asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                2,
            )
            .await
            .unwrap_err();

        assert_custom_error_ix!(1, error, MetadataError::EditionsMustHaveExactlyOneToken);
    }

    #[tokio::test]
    async fn try_mint_multiple_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let mut asset = DigitalAsset::default();
        let error = asset
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 2)
            .await
            .unwrap_err();

        assert_custom_error_ix!(1, error, MetadataError::EditionsMustHaveExactlyOneToken);
    }
}
