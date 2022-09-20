pub mod utils;

use solana_program_test::tokio;
use solana_sdk::signature::{Keypair, Signer};

use utils::{
    context::{BubblegumTestContext, DEFAULT_LAMPORTS_FUND_AMOUNT},
    tree::Tree,
    LeafArgs, Result,
};

// Test for multiple combinations?
const MAX_DEPTH: usize = 14;
const MAX_BUF_SIZE: usize = 64;

// Minting too many leaves takes quite a long time (in these tests at least).
const DEFAULT_NUM_MINTS: u64 = 10;

// TODO: test signer conditions on mint_authority and other stuff that's manually checked
// and not by anchor (what else is there?)

// TODO: will add some exta checks to the tests below (i.e. read accounts and
// assert on values therein).

// Creates a `BubblegumTestContext`, a `Tree` with default arguments, and also mints an NFT
// with the default `LeafArgs`.
async fn context_tree_and_leaves() -> Result<(
    BubblegumTestContext,
    Tree<MAX_DEPTH, MAX_BUF_SIZE>,
    Vec<LeafArgs>,
)> {
    let context = BubblegumTestContext::new().await?;

    let (tree, leaves) = context
        .default_create_and_mint::<MAX_DEPTH, MAX_BUF_SIZE>(DEFAULT_NUM_MINTS)
        .await?;

    Ok((context, tree, leaves))
}

#[tokio::test]
async fn test_create_tree_and_mint_passes() {
    let (context, tree, _) = context_tree_and_leaves().await.unwrap();

    let payer = context.payer();

    let cfg = tree.read_tree_config().await.unwrap();
    assert_eq!(cfg.tree_creator, payer.pubkey());
    assert_eq!(cfg.tree_delegate, payer.pubkey());
    assert_eq!(cfg.total_mint_capacity, 1 << MAX_DEPTH);
    assert_eq!(cfg.num_minted, DEFAULT_NUM_MINTS);
}

#[tokio::test]
async fn test_creator_verify_and_unverify_passes() {
    let (context, tree, mut leaves) = context_tree_and_leaves().await.unwrap();
    let leaf = leaves.last_mut().unwrap();

    tree.verify_creator(leaf, &context.default_creators[0])
        .await
        .unwrap();

    tree.unverify_creator(leaf, &context.default_creators[0])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_delegate_passes() {
    let (_, tree, mut leaves) = context_tree_and_leaves().await.unwrap();
    let leaf = leaves.last_mut().unwrap();

    let new_delegate = Keypair::new();

    tree.delegate(leaf, &new_delegate).await.unwrap();
}

#[tokio::test]
async fn test_transfer_passes() {
    let (mut context, tree, mut leaves) = context_tree_and_leaves().await.unwrap();
    let leaf = leaves.last_mut().unwrap();

    let new_owner = Keypair::new();

    context
        .fund_account(new_owner.pubkey(), DEFAULT_LAMPORTS_FUND_AMOUNT)
        .await
        .unwrap();

    tree.transfer(leaf, &new_owner).await.unwrap();
}

#[tokio::test]
async fn test_burn_passes() {
    let (_, tree, leaves) = context_tree_and_leaves().await.unwrap();
    let leaf = leaves.last().unwrap();

    tree.burn(&leaf).await.unwrap();
}

#[tokio::test]
async fn test_set_tree_delegate_passes() {
    let (_, tree, _) = context_tree_and_leaves().await.unwrap();
    let new_tree_delegate = Keypair::new();

    tree.set_tree_delegate(&new_tree_delegate).await.unwrap();
}
