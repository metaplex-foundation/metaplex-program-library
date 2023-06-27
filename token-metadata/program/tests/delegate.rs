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

    use mpl_token_auth_rules::error::RuleSetError;
    use mpl_token_metadata::{
        error::MetadataError,
        instruction::{DelegateArgs, MetadataDelegateRole},
        pda::{find_metadata_delegate_record_account, find_token_record_account},
        state::{
            Key, Metadata, MetadataDelegateRecord, PrintSupply, TokenDelegateRole, TokenRecord,
            TokenStandard,
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

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());

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
    async fn set_transfer_delegate_programmable_nonfungible_edition() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut master_asset = DigitalAsset::default();
        master_asset
            .create_and_mint_with_supply(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                PrintSupply::Unlimited,
            )
            .await
            .unwrap();

        assert!(master_asset.token.is_some());

        let test_master_edition = MasterEditionV2::new_from_asset(&master_asset);
        let mut test_edition_marker =
            EditionMarker::new_from_asset(&master_asset, &test_master_edition, 1);

        test_edition_marker
            .create_from_asset(&mut context)
            .await
            .unwrap();

        // delegates the asset for transfer

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        test_edition_marker
            .delegate_asset(
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

        let (pda_key, _) = find_token_record_account(
            &test_edition_marker.mint.pubkey(),
            &test_edition_marker.token.pubkey(),
        );

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.key, Key::TokenRecord);
        assert_eq!(token_record.delegate, Some(user_pubkey));
        assert_eq!(
            token_record.delegate_role,
            Some(TokenDelegateRole::Transfer)
        );

        let account = get_account(&mut context, &test_edition_marker.token.pubkey()).await;
        let token_account = Account::unpack(&account.data).unwrap();

        assert!(token_account.is_frozen());
        assert_eq!(token_account.delegate, COption::Some(user_pubkey));
        assert_eq!(token_account.delegated_amount, 1);
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

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());

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
    async fn set_utility_delegate_programmable_nonfungible() {
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

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.key, Key::TokenRecord);
        assert_eq!(token_record.delegate, Some(user_pubkey));
        assert_eq!(token_record.delegate_role, Some(TokenDelegateRole::Utility));

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(token_account.is_frozen());
            assert_eq!(token_account.delegate, COption::Some(user_pubkey));
            assert_eq!(token_account.delegated_amount, 1);

            // Close Authority should be set to the asset's Master Edition key.
            assert_eq!(
                token_account.close_authority,
                COption::Some(asset.edition.unwrap())
            );
        } else {
            panic!("Missing token account");
        }
    }

    #[tokio::test]
    async fn cannot_set_sale_delegate_nonfungible() {
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

        assert_custom_error_ix!(1, error, MetadataError::InvalidDelegateRole);
    }

    #[tokio::test]
    async fn set_standard_delegate_nonfungible() {
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
                DelegateArgs::StandardV1 { amount: 1 },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn cannot_set_utility_delegate_nonfungible() {
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
                DelegateArgs::UtilityV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap_err();

        assert_custom_error_ix!(1, error, MetadataError::InvalidDelegateRole);
    }

    #[tokio::test]
    async fn store_rule_set_revision_on_delegate() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.set_compute_max_units(400_000);
        let mut context = program_test.start_with_context().await;

        // creates the auth rule set

        let payer = context.payer.dirty_clone();
        let (rule_set, auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                Some(rule_set),
                Some(auth_data),
                1,
            )
            .await
            .unwrap();

        assert!(asset.token.is_some());

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());
        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.rule_set_revision, None);

        // delegates the asset for transfer

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                // delegate must be from Token Auth Rules or Rooster
                rule_set,
                DelegateArgs::SaleV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.key, Key::TokenRecord);
        assert_eq!(token_record.delegate, Some(rule_set));
        assert_eq!(token_record.delegate_role, Some(TokenDelegateRole::Sale));
        assert_eq!(token_record.rule_set_revision, Some(0));

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(token_account.is_frozen());
            assert_eq!(token_account.delegate, COption::Some(rule_set));
            assert_eq!(token_account.delegated_amount, 1);
        } else {
            panic!("Missing token account");
        }
    }

    #[tokio::test]
    async fn set_locked_transfer_delegate_programmable_nonfungible() {
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

        asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateArgs::LockedTransferV1 {
                    amount: 1,
                    locked_address: asset.metadata,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.key, Key::TokenRecord);
        assert_eq!(token_record.delegate, Some(user_pubkey));
        assert_eq!(
            token_record.delegate_role,
            Some(TokenDelegateRole::LockedTransfer)
        );
        assert_eq!(token_record.locked_transfer, Some(asset.metadata));

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
    async fn delegate_not_in_allow_list() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.set_compute_max_units(400_000);
        let mut context = program_test.start_with_context().await;

        // creates the auth rule set

        let payer = context.payer.dirty_clone();
        let (rule_set, auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, true).await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                Some(rule_set),
                Some(auth_data),
                1,
            )
            .await
            .unwrap();

        assert!(asset.token.is_some());

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());
        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.rule_set_revision, None);

        // delegates the asset for transfer

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let error = asset
            .delegate(
                &mut context,
                payer,
                // delegate not authorized
                user_pubkey,
                DelegateArgs::TransferV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap_err();

        // asserts

        assert_custom_error_ix!(1, error, RuleSetError::DataIsEmpty);
    }

    #[tokio::test]
    async fn invalid_close_authority_fails() {
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

        let delegate = Keypair::new();
        let delegate_pubkey = delegate.pubkey();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .inject_close_authority(&mut context, &delegate_pubkey)
            .await;

        let err = asset
            .delegate(
                &mut context,
                payer,
                delegate_pubkey,
                DelegateArgs::UtilityV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap_err();

        assert_custom_error_ix!(1, err, MetadataError::InvalidCloseAuthority);
    }
}
