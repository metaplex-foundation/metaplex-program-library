#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::error::MetadataError;
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{instruction::InstructionError, pubkey::Pubkey, transaction::TransactionError};
use test_case::test_case;
use utils::*;

#[test_case(spl_token::id(); "token")]
#[test_case(spl_token_2022::id(); "token-2022")]
#[tokio::test]
async fn create_master_edition_without_freeze_auth_fails(token_program_id: Pubkey) {
    let mut context = program_test().start_with_context().await;

    let mut nft = Metadata::new();
    nft.token_program_id = token_program_id;
    // Create a NFT mint with Freeze Authority set to None.
    nft.create_v3_no_freeze_auth(&mut context).await.unwrap();

    // Creating a Master Edition should fail as create_master_edition requires a Freeze Authority
    // to be set.
    let master_edition = MasterEditionV2::new(&nft);
    let err = master_edition
        .create_v3(&mut context, Some(0))
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::NoFreezeAuthoritySet);
}
