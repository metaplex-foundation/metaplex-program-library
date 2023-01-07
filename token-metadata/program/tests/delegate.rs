#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use utils::*;

mod delegate {

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::{builders::DelegateBuilder, DelegateArgs, DelegateRole, InstructionBuilder},
        pda::find_delegate_account,
        state::{DelegateRecord, Key, Metadata, TokenStandard},
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
                DelegateRole::Transfer,
                Some(1),
            )
            .await
            .unwrap();

        let (pda_key, _) = find_delegate_account(
            &asset.mint.pubkey(),
            DelegateRole::Transfer,
            &payer_pubkey,
            &user_pubkey,
        );

        let pda = get_account(&mut context, &pda_key).await;
        let delegate_record: DelegateRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(delegate_record.key, Key::Delegate);
        assert_eq!(delegate_record.delegate, user_pubkey);
        assert_eq!(delegate_record.role, DelegateRole::Transfer);

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
                DelegateRole::Collection,
                Some(1),
            )
            .await
            .unwrap();

        // asserts

        // delegate PDA
        let (pda_key, _) = find_delegate_account(
            &asset.mint.pubkey(),
            DelegateRole::Collection,
            &payer_pubkey,
            &user_pubkey,
        );

        let pda = get_account(&mut context, &pda_key).await;
        let delegate_record: DelegateRecord = DelegateRecord::from_bytes(&pda.data).unwrap();
        assert_eq!(delegate_record.key, Key::Delegate);
        assert_eq!(delegate_record.role, DelegateRole::Collection);
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
        let payer_pubkey = context.payer.pubkey();

        // delegate PDA
        let (pda_key, _) = find_delegate_account(
            &asset.mint.pubkey(),
            DelegateRole::Sale,
            &payer_pubkey,
            &user_pubkey,
        );

        let delegate_ix = DelegateBuilder::new()
            .delegate_record(pda_key)
            .delegate(user_pubkey)
            .mint(asset.mint.pubkey())
            .metadata(asset.metadata)
            .master_edition(asset.master_edition.unwrap())
            .approver(payer_pubkey)
            .payer(payer_pubkey)
            .token(asset.token.unwrap())
            .build(DelegateArgs::SaleV1 {
                amount: 1,
                authorization_data: None,
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // asserts

        let pda = get_account(&mut context, &pda_key).await;
        let delegate_record: DelegateRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(delegate_record.key, Key::Delegate);
        assert_eq!(delegate_record.delegate, user_pubkey);
        assert_eq!(delegate_record.role, DelegateRole::Sale);

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
        let payer_pubkey = context.payer.pubkey();

        // delegate PDA
        let (pda_key, _) = find_delegate_account(
            &asset.mint.pubkey(),
            DelegateRole::Sale,
            &payer_pubkey,
            &user_pubkey,
        );

        let delegate_ix = DelegateBuilder::new()
            .delegate_record(pda_key)
            .delegate(user_pubkey)
            .mint(asset.mint.pubkey())
            .metadata(asset.metadata)
            .master_edition(asset.master_edition.unwrap())
            .approver(payer_pubkey)
            .payer(payer_pubkey)
            .token(asset.token.unwrap())
            .build(DelegateArgs::SaleV1 {
                amount: 1,
                authorization_data: None,
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let error = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // asserts

        assert_custom_error!(error, MetadataError::InvalidTokenStandard);
    }
}
