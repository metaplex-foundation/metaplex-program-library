#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::TransactionError,
};
use utils::*;

mod delegate {

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::{DelegateArgs, MetadataDelegateRole},
        pda::{find_metadata_delegate_record_account, find_token_record_account},
        state::{
            Key, Metadata, MetadataDelegateRecord, TokenDelegateRole, TokenRecord, TokenStandard,
        },
    };
    use num_traits::FromPrimitive;
    use solana_program::{
        borsh::try_from_slice_unchecked, program_option::COption, program_pack::Pack,
    };
    use spl_token::state::Account;

    use super::*;

    #[tokio::test]
    async fn set_transfer_delegate_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        assert!(asset.token.is_some());

        // delegates the asset for transfer

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let payer_pubkey = payer.pubkey();

        asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateArgs::TransferV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &payer_pubkey);

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.key, Key::TokenRecord);
        assert_eq!(token_record.delegate, Some(user_pubkey));
        assert_eq!(
            token_record.delegate_role,
            Some(TokenDelegateRole::Transfer)
        );

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(token_account.is_frozen());
            assert_eq!(token_account.delegate, COption::Some(user_pubkey));
            assert_eq!(token_account.delegated_amount, 1);
        } else {
            panic!("Missing token account");
        }
    }

    #[tokio::test]
    async fn set_collection_delegate_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        assert!(asset.token.is_some());

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(metadata.update_authority, context.payer.pubkey());

        // creates a collection delegate

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let payer_pubkey = payer.pubkey();

        asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateArgs::CollectionV1 {
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // asserts

        let (pda_key, _) = find_metadata_delegate_record_account(
            &asset.mint.pubkey(),
            MetadataDelegateRole::Collection,
            &payer_pubkey,
            &user_pubkey,
        );

        let pda = get_account(&mut context, &pda_key).await;
        let delegate_record = MetadataDelegateRecord::from_bytes(&pda.data).unwrap();
        assert_eq!(delegate_record.key, Key::MetadataDelegate);
    }

    #[tokio::test]
    async fn set_sale_delegate_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        assert!(asset.token.is_some());

        // delegates the asset for transfer

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let payer_pubkey = payer.pubkey();

        asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateArgs::SaleV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &payer_pubkey);

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.key, Key::TokenRecord);
        assert_eq!(token_record.delegate, Some(user_pubkey));
        assert_eq!(token_record.delegate_role, Some(TokenDelegateRole::Sale));

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(token_account.is_frozen());
            assert_eq!(token_account.delegate, COption::Some(user_pubkey));
            assert_eq!(token_account.delegated_amount, 1);
        } else {
            panic!("Missing token account");
        }
    }

    #[tokio::test]
    async fn set_sale_delegate_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        assert!(asset.token.is_some());

        // delegates the asset for transfer

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let error = asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateArgs::SaleV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap_err();

        // asserts

        assert_custom_error!(error, MetadataError::InvalidTokenStandard);
    }

    #[tokio::test]
    async fn set_transfer_delegate_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        assert!(asset.token.is_some());

        // delegates the asset for transfer

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateArgs::TransferV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn set_utility_delegate_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        assert!(asset.token.is_some());

        // delegates the asset for transfer

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateArgs::UtilityV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();
    }
}
