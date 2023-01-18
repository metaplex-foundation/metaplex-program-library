#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use utils::*;

mod lock {

    use mpl_token_metadata::{
        pda::find_token_record_account,
        state::{TokenRecord, TokenStandard, TokenState},
    };
    use solana_program::{
        borsh::try_from_slice_unchecked, native_token::LAMPORTS_PER_SOL, program_option::COption,
        program_pack::Pack,
    };
    use solana_sdk::signature::{Keypair, Signer};
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

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &context.payer.pubkey());

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.state, TokenState::Unlocked);

        // locks

        let approver = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .lock(&mut context, approver, Some(pda_key), payer)
            .await
            .unwrap();

        // asserts

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.state, TokenState::Locked);
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

        let token_account = get_account(&mut context, &asset.token.unwrap()).await;
        let token = Account::unpack(&token_account.data).unwrap();
        // should be frozen
        assert!(token.is_frozen());
    }

    #[tokio::test]
    async fn non_fungible_program_delegate_lock() {
        // Tests a delegate transferring from a system wallet to a PDA and vice versa.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        let mut context = program_test.start_with_context().await;

        // creates an NFT

        let mut nft = DigitalAsset::new();
        nft.create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        // locks the NFT in a "staking" (rooster) program

        let payer = context.payer.dirty_clone();
        airdrop(&mut context, &payer.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let rooster_manager = RoosterManager::init(&mut context, payer).await.unwrap();

        let token_owner = context.payer.dirty_clone();
        let token = nft.token.unwrap();

        rooster_manager
            .lock(
                &mut context,
                &token_owner,
                token,
                nft.mint.pubkey(),
                nft.metadata,
                nft.master_edition.unwrap(),
            )
            .await
            .unwrap();

        // asserts

        let account = get_account(&mut context, &token).await;
        let token_account = Account::unpack(&account.data).unwrap();

        assert!(token_account.is_frozen());
        assert_eq!(
            token_account.delegate,
            COption::Some(rooster_manager.pda()),
        );
        assert_eq!(token_account.delegated_amount, 1);
    }
}
