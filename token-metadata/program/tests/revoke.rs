#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::TransactionError,
};
use utils::*;

mod revoke {

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::DelegateRole,
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
    async fn revoke_transfer_delegate_programmable_nonfungible() {
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

        // delegates the asset for sale

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

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

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(
            metadata.delegate_state.unwrap().delegate,
            user_pubkey /* delegate owner */
        );

        // revokes the delegate
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .revoke(&mut context, payer, user_pubkey, DelegateRole::Transfer)
            .await
            .unwrap();

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(metadata.delegate_state, None);

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(token_account.is_frozen());
            assert_eq!(token_account.delegate, COption::None);
        } else {
            panic!("Missing token account");
        }
    }

    #[tokio::test]
    async fn revoke_collection_delegate_programmable_nonfungible() {
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

        // // delegates the asset for transfer
        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // // delegate PDA
        let (delegate_pda, _) = find_delegate_account(
            &asset.mint.pubkey(),
            DelegateRole::Collection,
            &user_pubkey,
            &payer.pubkey(),
        );

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

        // // checks that the delegate exists
        let delegate_account = get_account(&mut context, &delegate_pda).await;
        let delegate: DelegateRecord = DelegateRecord::from_bytes(&delegate_account.data).unwrap();
        assert_eq!(delegate.key, Key::Delegate);
        assert_eq!(delegate.role, DelegateRole::Collection);

        // revokes the delegate
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .revoke(&mut context, payer, user_pubkey, DelegateRole::Collection)
            .await
            .unwrap();

        // checks that the delagate exists (it should not exist)

        assert!(context
            .banks_client
            .get_account(delegate_pda)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn revoke_sale_delegate_programmable_nonfungible() {
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

        // delegates the asset for sale
        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateRole::Sale,
                Some(1),
            )
            .await
            .unwrap();

        // checks that the delagate exists

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(
            metadata.delegate_state.unwrap().delegate,
            user_pubkey /* delegate owner */
        );

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // // delegate PDA
        // let (delegate_pda, _) = find_delegate_account(
        //     &asset.mint.pubkey(),
        //     DelegateRole::Sale,
        //     &user_pubkey,
        //     &payer.pubkey(),
        // );

        // revokes the delegate
        asset
            .revoke(&mut context, payer, user_pubkey, DelegateRole::Sale)
            .await
            .unwrap();

        // assert
        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(metadata.delegate_state, None);

        if let Some(token) = asset.token {
            let account = get_account(&mut context, &token).await;
            let token_account = Account::unpack(&account.data).unwrap();

            assert!(token_account.is_frozen());
            assert_eq!(token_account.delegate, COption::None);
        } else {
            panic!("Missing token account");
        }
    }

    #[tokio::test]
    async fn revoke_sale_delegate_as_transfer_delegate() {
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

        // delegates the asset for sale
        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                user_pubkey,
                DelegateRole::Sale,
                Some(1),
            )
            .await
            .unwrap();

        // checks that the delagate exists

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(
            metadata.delegate_state.unwrap().delegate,
            user_pubkey /* delegate owner */
        );

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // revokes the delegate
        let error = asset
            .revoke(&mut context, payer, user_pubkey, DelegateRole::Transfer)
            .await
            .unwrap_err();

        // assert

        assert_custom_error!(error, MetadataError::InvalidDelegate);
    }
}
