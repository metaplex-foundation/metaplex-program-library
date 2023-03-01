#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::VerifyArgs,
    state::{Creator, TokenStandard},
};
use num_traits::FromPrimitive;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};
use utils::*;

mod verify_creator {
    use super::*;

    #[tokio::test]
    async fn metadata_wrong_owner() {
        let mut context = program_test().start_with_context().await;
        let mut da = DigitalAsset::new();

        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let unverified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        // Create, mint, verify creator, and check creators matches on-chain.
        create_mint_verify_check(
            &mut context,
            &mut da,
            creator.dirty_clone(),
            &unverified_creators,
        )
        .await;

        let verified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: true,
        }]);

        // Unverify creator.
        let args = VerifyArgs::CreatorV1;
        let metadata_wrong_owner = Keypair::new().pubkey();
        let err = da
            .unverify(
                &mut context,
                creator,
                args,
                Some(metadata_wrong_owner),
                None,
                None,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::IncorrectOwner);

        da.assert_creators_matches_on_chain(&mut context, &verified_creators)
            .await;
    }

    #[tokio::test]
    async fn update_authority_cannot_unverify_creator() {
        let mut context = program_test().start_with_context().await;
        let mut da = DigitalAsset::new();

        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let unverified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        // Create, mint, verify creator, and check creators matches on-chain.
        create_mint_verify_check(
            &mut context,
            &mut da,
            creator.dirty_clone(),
            &unverified_creators,
        )
        .await;

        let verified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: true,
        }]);

        // Unverify creator.
        let payer = context.payer.dirty_clone();
        let args = VerifyArgs::CreatorV1;
        let err = da
            .unverify(&mut context, payer, args, None, None, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CreatorNotFound);

        da.assert_creators_matches_on_chain(&mut context, &verified_creators)
            .await;
    }

    #[tokio::test]
    async fn no_creators_found() {
        let mut context = program_test().start_with_context().await;

        // Create and mint item.
        let mut da = DigitalAsset::new();
        da.create_and_mint_with_creators(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
            None,
        )
        .await
        .unwrap();

        da.assert_creators_matches_on_chain(&mut context, &None)
            .await;

        // Unverify creator.
        let payer = context.payer.dirty_clone();
        let args = VerifyArgs::CreatorV1;
        let err = da
            .unverify(&mut context, payer, args, None, None, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::NoCreatorsPresentOnMetadata);

        da.assert_creators_matches_on_chain(&mut context, &None)
            .await;
    }

    #[tokio::test]
    async fn pass() {
        let mut context = program_test().start_with_context().await;
        let mut da = DigitalAsset::new();

        let creator = Keypair::new();
        airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let unverified_creators = Some(vec![Creator {
            address: creator.pubkey(),
            share: 100,
            verified: false,
        }]);

        // Create, mint, verify creator, and check creators matches on-chain.
        create_mint_verify_check(
            &mut context,
            &mut da,
            creator.dirty_clone(),
            &unverified_creators,
        )
        .await;

        // Unverify creator.
        let args = VerifyArgs::CreatorV1;

        da.unverify(&mut context, creator, args, None, None, None, None)
            .await
            .unwrap();

        da.assert_creators_matches_on_chain(&mut context, &unverified_creators)
            .await;
    }
}

async fn create_mint_verify_check(
    context: &mut ProgramTestContext,
    da: &mut DigitalAsset,
    creator: Keypair,
    unverified_creators: &Option<Vec<Creator>>,
) {
    // Create and mint item.
    da.create_and_mint_with_creators(
        context,
        TokenStandard::ProgrammableNonFungible,
        None,
        None,
        1,
        unverified_creators.clone(),
    )
    .await
    .unwrap();

    da.assert_creators_matches_on_chain(context, unverified_creators)
        .await;

    // Verify.
    let args = VerifyArgs::CreatorV1;

    let verified_creators = Some(
        unverified_creators
            .clone()
            .unwrap()
            .into_iter()
            .map(|mut c| {
                if c.address == creator.pubkey() {
                    c.verified = true
                }
                c
            })
            .collect::<Vec<Creator>>(),
    );

    da.verify(
        context,
        creator.dirty_clone(),
        args,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    da.assert_creators_matches_on_chain(context, &verified_creators)
        .await;
}
