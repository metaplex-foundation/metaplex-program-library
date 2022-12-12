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
        instruction::DelegateRole,
        pda::find_delegate_account,
        state::{DelegateRecord, Key, Metadata, TokenStandard},
    };
    use solana_program::borsh::try_from_slice_unchecked;
    use std::borrow::Borrow;

    use super::*;
    #[tokio::test]
    async fn set_sale_delegate() {
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

        // delegates the asset for sale

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        let payer_pubkey = context.payer.pubkey();

        // delegate PDA
        let (delegate, _) = find_delegate_account(
            &asset.mint.pubkey(),
            DelegateRole::Sale,
            &user_pubkey,
            &payer_pubkey,
        );

        let delegate_ix = instruction::delegate(
            /* delegate              */ delegate,
            /* delegate owner        */ user_pubkey,
            /* owner                 */ payer_pubkey,
            /* payer                 */ payer_pubkey,
            /* token                 */ asset.token.unwrap(),
            /* metadata              */ asset.metadata,
            /* mint                  */ asset.mint.pubkey(),
            /* delegate role         */ DelegateRole::Sale,
            /* authorization payload */ None,
            /* additional accounts   */ None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let delegate_account = get_account(&mut context, &delegate).await;
        let delegate = Delegate::from_bytes(delegate_account.data.borrow()).unwrap();
        assert_eq!(delegate.key, Key::Delegate);
        assert_eq!(delegate.role, DelegateRole::Sale);

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(
            metadata.delegate,
            Some(user_pubkey) /* delegate owner */
        );
    }
}
