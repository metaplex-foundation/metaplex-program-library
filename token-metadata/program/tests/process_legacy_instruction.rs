#![cfg(feature = "test-bpf")]
pub mod utils;

use num_traits::FromPrimitive;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use utils::*;

mod process_legacy_instruction {

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::{sign_metadata, DelegateArgs},
        state::{Metadata, TokenStandard},
    };
    use solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack};
    use spl_token::state::Account;

    use super::*;

    #[tokio::test]
    async fn programmable_nft_in_legacy_processor() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create(&mut context, TokenStandard::ProgrammableNonFungible, None)
            .await
            .unwrap();

        // mints one token

        let payer_pubkey = context.payer.pubkey();
        let (token, _) = Pubkey::find_program_address(
            &[
                &payer_pubkey.to_bytes(),
                &spl_token::ID.to_bytes(),
                &asset.mint.pubkey().to_bytes(),
            ],
            &spl_associated_token_account::ID,
        );
        asset.token = Some(token);

        asset.mint(&mut context, None, None, 1).await.unwrap();

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(
            metadata.token_standard,
            Some(TokenStandard::ProgrammableNonFungible)
        );

        // tries to use a "legacy" instruction with a pNFT

        // we won't need to use this keypair
        let creator = Keypair::new();

        let sign_ix = sign_metadata(mpl_token_metadata::ID, asset.metadata, creator.pubkey());
        let sign_tx = Transaction::new_signed_with_payer(
            &[sign_ix],
            Some(&context.payer.pubkey()),
            &[&creator, &context.payer],
            context.last_blockhash,
        );

        let error = context
            .banks_client
            .process_transaction(sign_tx)
            .await
            .unwrap_err();

        assert_custom_error!(error, MetadataError::InstructionNotSupported);
    }

    #[tokio::test]
    async fn thaw_programmable_nft() {
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

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(
            metadata.token_standard,
            Some(TokenStandard::ProgrammableNonFungible)
        );

        let account = get_account(&mut context, &asset.token.unwrap()).await;
        let token_account = Account::unpack(&account.data).unwrap();

        assert!(token_account.is_frozen());
        assert_eq!(token_account.amount, 1);
        assert_eq!(token_account.mint, asset.mint.pubkey());
        assert_eq!(token_account.owner, context.payer.pubkey());

        // creates a transfer delegate so we have a SPL Token delegate in place

        let delegate = Keypair::new();
        let delegate_pubkey = delegate.pubkey();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                delegate_pubkey,
                DelegateArgs::TransferV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // tries to use a "legacy" thaw instruction with a pNFT

        let thaw_ix = mpl_token_metadata::instruction::thaw_delegated_account(
            mpl_token_metadata::ID,
            delegate_pubkey,
            asset.token.unwrap(),
            asset.edition.unwrap(),
            asset.mint.pubkey(),
        );
        let thaw_tx = Transaction::new_signed_with_payer(
            &[thaw_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &delegate],
            context.last_blockhash,
        );

        // fails because it is a pNFT master edition
        let error = context
            .banks_client
            .process_transaction(thaw_tx)
            .await
            .unwrap_err();

        assert_custom_error!(error, MetadataError::InvalidTokenStandard);

        // makes sure the token still frozen

        let account = get_account(&mut context, &asset.token.unwrap()).await;
        let token_account = Account::unpack(&account.data).unwrap();
        assert!(token_account.is_frozen());

        // tries to freeze (this would normally fail at the SPL Token level, but we
        // should get our custom error first)

        let freeze_ix = mpl_token_metadata::instruction::freeze_delegated_account(
            mpl_token_metadata::ID,
            delegate_pubkey,
            asset.token.unwrap(),
            asset.edition.unwrap(),
            asset.mint.pubkey(),
        );

        let freeze_tx = Transaction::new_signed_with_payer(
            &[freeze_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &delegate],
            context.last_blockhash,
        );

        // fails because it is a pNFT master edition
        let error = context
            .banks_client
            .process_transaction(freeze_tx)
            .await
            .unwrap_err();

        assert_custom_error!(error, MetadataError::InvalidTokenStandard);
    }
}
