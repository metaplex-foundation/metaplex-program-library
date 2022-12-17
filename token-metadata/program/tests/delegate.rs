#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::instruction;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use utils::*;

mod delegate {

    use mpl_token_metadata::{
        instruction::{DelegateArgs, DelegateRole},
        pda::find_delegate_account,
        state::{DelegateRecord, Key, Metadata, TokenStandard},
    };
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
        let (delegate, _) = find_delegate_account(
            &asset.mint.pubkey(),
            DelegateRole::Transfer,
            &user_pubkey,
            &payer_pubkey,
        );

        let delegate_ix = instruction::delegate(
            /* delegate              */ delegate,
            /* delegate owner        */ user_pubkey,
            /* mint                  */ asset.mint.pubkey(),
            /* metadata              */ asset.metadata,
            /* master_edition        */ asset.master_edition,
            /* authority             */ payer_pubkey,
            /* payer                 */ payer_pubkey,
            /* token                 */ asset.token,
            /* authorization payload */ None,
            /* additional accounts   */ None,
            /* delegate args         */ DelegateArgs::TransferV1 { amount: 1 },
        );

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // asserts

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(
            metadata.delegate,
            Some(user_pubkey) /* delegate owner */
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
                1,
            )
            .await
            .unwrap();

        assert!(asset.token.is_some());

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(metadata.update_authority, context.payer.pubkey());

        // delegates the asset for transfer

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer_pubkey = context.payer.pubkey();

        // delegate PDA
        let (delegate, _) = find_delegate_account(
            &asset.mint.pubkey(),
            DelegateRole::Collection,
            &user_pubkey,
            &payer_pubkey,
        );

        let delegate_ix = instruction::delegate(
            /* delegate              */ delegate,
            /* delegate owner        */ user_pubkey,
            /* mint                  */ asset.mint.pubkey(),
            /* metadata              */ asset.metadata,
            /* master_edition        */ asset.master_edition,
            /* authority             */ payer_pubkey,
            /* payer                 */ payer_pubkey,
            /* token                 */ None,
            /* authorization payload */ None,
            /* additional accounts   */ None,
            /* delegate args         */ DelegateArgs::CollectionV1,
        );

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // asserts

        let delegate_account = get_account(&mut context, &delegate).await;
        let delegate: DelegateRecord = DelegateRecord::from_bytes(&delegate_account.data).unwrap();
        assert_eq!(delegate.key, Key::Delegate);
        assert_eq!(delegate.role, DelegateRole::Collection);
    }
}
