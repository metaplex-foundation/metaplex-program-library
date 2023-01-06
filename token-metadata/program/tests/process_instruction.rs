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

mod process_instruction {

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::sign_metadata,
        state::{Metadata, TokenStandard},
    };
    use solana_program::borsh::try_from_slice_unchecked;

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
                &spl_token::id().to_bytes(),
                &asset.mint.pubkey().to_bytes(),
            ],
            &spl_associated_token_account::id(),
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

        let sign_ix = sign_metadata(mpl_token_metadata::id(), asset.metadata, creator.pubkey());
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
}
