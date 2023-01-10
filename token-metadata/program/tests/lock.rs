#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use utils::*;

mod lock {

    use mpl_token_metadata::state::{AssetState, Metadata, TokenStandard};
    use solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack};
    use solana_sdk::signature::Keypair;
    use spl_token::state::Account;

    use super::*;

    #[tokio::test]
    async fn lock_programmable_nonfungible() {
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

        // asserts

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(metadata.asset_state, Some(AssetState::Unlocked));

        // locks

        let approver = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .lock(&mut context, approver, None, payer)
            .await
            .unwrap();

        // asserts

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();
        assert_eq!(metadata.asset_state, Some(AssetState::Locked));
    }

    #[tokio::test]
    async fn lock_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        // asserts

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(metadata.asset_state, Some(AssetState::Unlocked));

        let token_account = get_account(&mut context, &asset.token.unwrap()).await;
        let token = Account::unpack(&token_account.data).unwrap();
        // should not be frozen
        assert!(!token.is_frozen());

        // lock the token

        let approver = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .lock(&mut context, approver, None, payer)
            .await
            .unwrap();

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(metadata.asset_state, Some(AssetState::Locked));

        let token_account = get_account(&mut context, &asset.token.unwrap()).await;
        let token = Account::unpack(&token_account.data).unwrap();
        // should be frozen
        assert!(token.is_frozen());
    }
}
