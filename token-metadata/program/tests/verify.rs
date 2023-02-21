#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::VerifyArgs,
    state::{Collection, Creator, TokenStandard},
};
use num_traits::FromPrimitive;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};

use utils::*;

mod pnft {
    use super::*;

    mod verify_creator {
        use super::*;

        #[tokio::test]
        async fn metadata_wrong_owner() {
            let mut context = program_test().start_with_context().await;

            let update_authority = context.payer.dirty_clone();
            let creator = Keypair::new();
            airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
                .await
                .unwrap();

            let creators = Some(vec![Creator {
                address: creator.pubkey(),
                share: 100,
                verified: false,
            }]);

            let mut da = DigitalAsset::new();
            da.create_and_mint_with_creators(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                creators.clone(),
            )
            .await
            .unwrap();

            da.assert_creators_matches_on_chain(&mut context, &creators)
                .await;

            let args = VerifyArgs::CreatorV1;
            let wrong_owner_metadata = Keypair::new().pubkey();

            let err = da
                .verify(
                    &mut context,
                    update_authority,
                    args,
                    Some(wrong_owner_metadata),
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .unwrap_err();

            assert_custom_error!(err, MetadataError::IncorrectOwner);

            da.assert_creators_matches_on_chain(&mut context, &creators)
                .await;
        }

        #[tokio::test]
        async fn update_authority_cannot_verify_creator() {
            let mut context = program_test().start_with_context().await;

            let update_authority = context.payer.dirty_clone();
            let creator = Keypair::new();
            airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
                .await
                .unwrap();

            let creators = Some(vec![Creator {
                address: creator.pubkey(),
                share: 100,
                verified: false,
            }]);

            let mut da = DigitalAsset::new();
            da.create_and_mint_with_creators(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                creators.clone(),
            )
            .await
            .unwrap();

            da.assert_creators_matches_on_chain(&mut context, &creators)
                .await;

            let args = VerifyArgs::CreatorV1;
            let err = da
                .verify(
                    &mut context,
                    update_authority,
                    args,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .unwrap_err();

            assert_custom_error!(err, MetadataError::CreatorNotFound);

            da.assert_creators_matches_on_chain(&mut context, &creators)
                .await;
        }

        #[tokio::test]
        async fn no_creators_found() {
            let mut context = program_test().start_with_context().await;

            let update_authority = context.payer.dirty_clone();
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

            let args = VerifyArgs::CreatorV1;
            let err = da
                .verify(
                    &mut context,
                    update_authority,
                    args,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .unwrap_err();

            assert_custom_error!(err, MetadataError::NoCreatorsPresentOnMetadata);

            da.assert_creators_matches_on_chain(&mut context, &None)
                .await;
        }

        #[tokio::test]
        async fn pass() {
            let mut context = program_test().start_with_context().await;

            let creator = Keypair::new();
            airdrop(&mut context, &creator.pubkey(), LAMPORTS_PER_SOL)
                .await
                .unwrap();

            let creators = Some(vec![Creator {
                address: creator.pubkey(),
                share: 100,
                verified: false,
            }]);

            let mut da = DigitalAsset::new();
            da.create_and_mint_with_creators(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                creators.clone(),
            )
            .await
            .unwrap();

            da.assert_creators_matches_on_chain(&mut context, &creators)
                .await;

            let args = VerifyArgs::CreatorV1;

            let verified_creators = Some(vec![Creator {
                address: creator.pubkey(),
                share: 100,
                verified: true,
            }]);

            da.verify(&mut context, creator, args, None, None, None, None, None)
                .await
                .unwrap();

            da.assert_creators_matches_on_chain(&mut context, &verified_creators)
                .await;
        }
    }

    mod verify_collection {
        use super::*;

        #[tokio::test]
        async fn collection_update_authority_pass() {
            let mut context = program_test().start_with_context().await;

            // Create a Collection Parent NFT with the CollectionDetails struct populated
            let collection_parent_nft = Metadata::new();
            collection_parent_nft
                .create_v3(
                    &mut context,
                    "Test".to_string(),
                    "TST".to_string(),
                    "uri".to_string(),
                    None,
                    10,
                    false,
                    None,
                    None,
                    DEFAULT_COLLECTION_DETAILS, // Collection Parent
                )
                .await
                .unwrap();

            let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
            parent_master_edition_account
                .create_v3(&mut context, Some(0))
                .await
                .unwrap();

            let collection = Collection {
                key: collection_parent_nft.mint.pubkey(),
                verified: false,
            };

            let mut da = DigitalAsset::new();
            da.create_and_mint_with_collection(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                Some(collection),
            )
            .await
            .unwrap();

            let args = VerifyArgs::CollectionV1;

            let payer = context.payer.dirty_clone();
            da.verify(
                &mut context,
                payer,
                args,
                None,
                None,
                Some(collection_parent_nft.mint.pubkey()),
                Some(collection_parent_nft.pubkey),
                Some(parent_master_edition_account.pubkey),
            )
            .await
            .unwrap();
        }
    }
}