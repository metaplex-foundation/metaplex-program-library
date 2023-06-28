#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program_test::tokio;

use solana_sdk::{signature::Keypair, signer::Signer};
use utils::context::BubblegumTestContext;

use crate::utils::{Airdrop, DirtyClone};

// Test for multiple combinations?
const MAX_DEPTH: usize = 14;
const MAX_BUF_SIZE: usize = 64;

// Minting too many leaves takes quite a long time (in these tests at least).
const DEFAULT_NUM_MINTS: u64 = 10;

#[tokio::test]
async fn verify_collection() {
    let context = BubblegumTestContext::new().await.unwrap();

    let (mut tree, mut leaves) = context
        .default_create_and_mint::<MAX_DEPTH, MAX_BUF_SIZE>(DEFAULT_NUM_MINTS)
        .await
        .unwrap();

    for leaf in leaves.iter_mut() {
        tree.verify_collection(
            leaf,
            &context.payer(),
            context.default_collection.mint.pubkey(),
            context.default_collection.metadata,
            context.default_collection.edition.unwrap(),
        )
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn verify_collection_with_old_delegate() {
    // Uses Collection Authority Record to verify a collection item.
    let mut context = BubblegumTestContext::new().await.unwrap();

    let (mut tree, mut leaves) = context
        .default_create_and_mint::<MAX_DEPTH, MAX_BUF_SIZE>(DEFAULT_NUM_MINTS)
        .await
        .unwrap();

    let payer = context.payer();

    // Set up our old delegate record: collection_authority_record.
    let delegate = Keypair::new();
    delegate
        .airdrop(context.mut_test_context(), LAMPORTS_PER_SOL)
        .await
        .unwrap();

    let record = context
        .set_collection_authority_delegate(payer, delegate.pubkey())
        .await
        .unwrap();

    let collection_asset = context.default_collection;

    // Get the first leaf and try to verify it with the delegate as the authority.
    let leaf = leaves.first_mut().unwrap();

    tree.delegate_verify_collection(
        leaf,
        &delegate,
        collection_asset.mint.pubkey(),
        collection_asset.metadata,
        collection_asset.edition.unwrap(),
        record,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn verify_collection_with_new_delegate() {
    // Uses MetadataDelegate to verify a collection item.

    let mut context = BubblegumTestContext::new().await.unwrap();

    let (mut tree, mut leaves) = context
        .default_create_and_mint::<MAX_DEPTH, MAX_BUF_SIZE>(DEFAULT_NUM_MINTS)
        .await
        .unwrap();

    let payer = context.payer().dirty_clone();

    // Set up our old delegate record: collection_authority_record.
    let delegate = Keypair::new();
    delegate
        .airdrop(context.mut_test_context(), LAMPORTS_PER_SOL)
        .await
        .unwrap();

    let mut collection_asset = context.default_collection.dirty_clone();
    let mut program_context = context.owned_test_context();

    let args = mpl_token_metadata::instruction::DelegateArgs::CollectionV1 {
        authorization_data: None,
    };

    let record = collection_asset
        .delegate(
            &mut program_context,
            payer.dirty_clone(),
            delegate.pubkey(),
            args,
        )
        .await
        .unwrap()
        .unwrap();

    // Get the first leaf and try to verify it with the delegate as the authority.
    let leaf = leaves.first_mut().unwrap();

    tree.delegate_verify_collection(
        leaf,
        &delegate,
        collection_asset.mint.pubkey(),
        collection_asset.metadata,
        collection_asset.edition.unwrap(),
        record,
    )
    .await
    .unwrap();
}
