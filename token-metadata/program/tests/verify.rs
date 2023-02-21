#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::{builders::BurnBuilder, BurnArgs, DelegateArgs, InstructionBuilder, VerifyArgs},
    state::{
        Collection, CollectionDetails, Creator, Key, MasterEditionV2 as ProgramMasterEditionV2,
        Metadata as ProgramMetadata, PrintSupply, TokenMetadataAccount, TokenStandard,
    },
};
use num_traits::FromPrimitive;
use solana_program::{native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;
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

            da.verify(
                &mut context,
                creator,
                args,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();

            da.assert_creators_matches_on_chain(&mut context, &verified_creators)
                .await;
        }
    }
}
