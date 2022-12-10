#![cfg(feature = "test-bpf")]
pub mod utils;

use anchor_lang::solana_program::instruction::InstructionError;
use mpl_token_metadata::{
    pda::{find_master_edition_account, find_metadata_account},
    state::{
        MasterEditionV2, TokenMetadataAccount, TokenStandard, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH,
        MAX_URI_LENGTH,
    },
    utils::puffed_out_string,
};
use solana_program::{account_info::AccountInfo, program_option::COption, program_pack::Pack};
use solana_program_test::{tokio, BanksClientError};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::TransactionError;

use spl_associated_token_account::get_associated_token_address;
use spl_token::{self, state::Mint};

use crate::utils::tree::decompress_mint_auth_pda;
use crate::utils::Error::BanksClient;
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
pub async fn context_tree_and_leaves() -> Result<(
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
    // The mint operation implicitly called below also verifies that the on-chain tree
    // root matches the expected value as leaves are added.
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

    // `verify_creator` and `unverify_creator` also validate the on-chain tree root
    // always has the expected value via the inner `TxBuilder::execute` call.

    for leaf in leaves.iter_mut() {
        tree.verify_creator(leaf, &context.default_creators[0])
            .await
            .unwrap();
    }

    for leaf in leaves.iter_mut() {
        tree.unverify_creator(leaf, &context.default_creators[0])
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_delegate_passes() {
    let (_, tree, mut leaves) = context_tree_and_leaves().await.unwrap();
    let new_delegate = Keypair::new();

    // `delegate` also validates whether the on-chain tree root always has the expected
    // value via the inner `TxBuilder::execute` call.

    for leaf in leaves.iter_mut() {
        tree.delegate(leaf, &new_delegate).await.unwrap();
    }
}

#[tokio::test]
async fn test_transfer_passes() {
    let (_, tree, mut leaves) = context_tree_and_leaves().await.unwrap();
    let new_owner = Keypair::new();

    // `transfer` also validates whether the on-chain tree root always has the expected
    // value via the inner `TxBuilder::execute` call.

    for leaf in leaves.iter_mut() {
        tree.transfer(leaf, &new_owner).await.unwrap();
    }
}

#[tokio::test]
async fn test_delegated_transfer_passes() {
    let (mut context, tree, mut leaves) = context_tree_and_leaves().await.unwrap();
    let delegate = Keypair::new();
    let new_owner = Keypair::new();

    context
        .fund_account(delegate.pubkey(), DEFAULT_LAMPORTS_FUND_AMOUNT)
        .await
        .unwrap();

    for leaf in leaves.iter_mut() {
        // We need to explicitly set a new delegate, since by default the owner has both
        // roles right after minting.
        tree.delegate(leaf, &delegate).await.unwrap();

        let mut tx = tree.transfer_tx(leaf, &new_owner).await.unwrap();

        // Set the delegate as payer and signer (by default, it's the owner).
        tx.set_payer(delegate.pubkey()).set_signers(&[&delegate]);

        // Also automatically checks the on-chain tree root matches the expected state.
        tx.execute().await.unwrap();
    }
}

#[tokio::test]
async fn test_burn_passes() {
    let (_, tree, leaves) = context_tree_and_leaves().await.unwrap();

    // `burn` also validates whether the on-chain tree root always has the expected
    // value via the inner `TxBuilder::execute` call.

    for leaf in leaves.iter() {
        tree.burn(&leaf).await.unwrap();
    }
}

#[tokio::test]
async fn test_set_tree_delegate_passes() {
    let (context, tree, _) = context_tree_and_leaves().await.unwrap();
    let new_tree_delegate = Keypair::new();

    // `set_tree_delegate` also validates whether the on-chain tree root always has the expected
    // value via the inner `TxBuilder::execute` call.

    let initial_cfg = tree.read_tree_config().await.unwrap();
    tree.set_tree_delegate(&new_tree_delegate).await.unwrap();
    let mut cfg = tree.read_tree_config().await.unwrap();

    // Configs are not the same.
    assert_ne!(cfg, initial_cfg);
    assert_eq!(cfg.tree_delegate, new_tree_delegate.pubkey());
    // Configs are the same if we change back the delegate (nothing else changed).
    cfg.tree_delegate = context.payer().pubkey();
    assert_eq!(cfg, initial_cfg);
}

#[tokio::test]
async fn test_reedem_and_cancel_passes() {
    let (_, tree, leaves) = context_tree_and_leaves().await.unwrap();

    // `redeem` and `cancel_redeem` also validate the on-chain tree root
    // always has the expected value via the inner `TxBuilder::execute` call.

    for leaf in leaves.iter() {
        tree.redeem(leaf).await.unwrap();

        let v = tree.read_voucher(leaf.nonce).await.unwrap();
        assert_eq!(v, tree.expected_voucher(leaf));
    }

    for leaf in leaves.iter() {
        tree.cancel_redeem(leaf).await.unwrap();
    }
}

#[tokio::test]
async fn test_decompress_passes() {
    let (ctx, tree, mut leaves) = context_tree_and_leaves().await.unwrap();

    for leaf in leaves.iter_mut() {
        tree.verify_creator(leaf, &ctx.default_creators[0])
            .await
            .unwrap();
        tree.redeem(leaf).await.unwrap();
        let voucher = tree.read_voucher(leaf.nonce).await.unwrap();

        // `decompress_v1` also validates whether the on-chain tree root always has
        // the expected value via the inner `TxBuilder::execute` call.
        tree.decompress_v1(&voucher, leaf).await.unwrap();

        let mint_key = voucher.decompress_mint_pda();
        let mint_account = tree.read_account(mint_key).await.unwrap();
        let mint = Mint::unpack(mint_account.data.as_slice()).unwrap();

        let expected_mint = Mint {
            mint_authority: COption::Some(find_master_edition_account(&mint_key).0),
            supply: 1,
            decimals: 0,
            is_initialized: true,
            freeze_authority: COption::Some(find_master_edition_account(&mint_key).0),
        };

        assert_eq!(mint, expected_mint);

        let token_account_key = get_associated_token_address(&leaf.owner.pubkey(), &mint_key);
        let token_account = tree.read_account(token_account_key).await.unwrap();
        let t = spl_token::state::Account::unpack(token_account.data.as_slice()).unwrap();

        let expected_t = spl_token::state::Account {
            mint: mint_key,
            owner: leaf.owner.pubkey(),
            amount: 1,
            state: spl_token::state::AccountState::Initialized,
            delegated_amount: 0,
            delegate: COption::None,
            is_native: COption::None,
            close_authority: COption::None,
        };

        assert_eq!(t, expected_t);

        let metadata_key = find_metadata_account(&mint_key).0;
        let mut meta_account = tree.read_account(metadata_key).await.unwrap();

        let meta: mpl_token_metadata::state::Metadata =
            mpl_token_metadata::state::Metadata::from_account_info(&AccountInfo::from((
                &metadata_key,
                &mut meta_account,
            )))
            .unwrap();

        let mut expected_creators = Vec::new();

        // Can't compare directly as they are different types for some reason.
        for c1 in leaf.metadata.creators.iter() {
            expected_creators.push(mpl_token_metadata::state::Creator {
                address: c1.address,
                verified: c1.verified,
                share: c1.share,
            });
        }

        assert!(expected_creators[0].verified);

        let expected_meta = mpl_token_metadata::state::Metadata {
            key: mpl_token_metadata::state::Key::MetadataV1,
            update_authority: decompress_mint_auth_pda(mint_key),
            mint: mint_key,
            data: mpl_token_metadata::state::Data {
                name: puffed_out_string(&leaf.metadata.name, MAX_NAME_LENGTH),
                symbol: puffed_out_string(&leaf.metadata.symbol, MAX_SYMBOL_LENGTH),
                uri: puffed_out_string(&leaf.metadata.uri, MAX_URI_LENGTH),
                seller_fee_basis_points: leaf.metadata.seller_fee_basis_points,
                creators: Some(expected_creators),
            },
            primary_sale_happened: false,
            is_mutable: false,
            collection: None,
            uses: None,
            collection_details: None,
            // Simply copying this, since the expected value is not straightforward to predict.
            edition_nonce: meta.edition_nonce,
            token_standard: Some(TokenStandard::NonFungible),
        };

        assert_eq!(meta, expected_meta);

        // Test master edition account.
        let me_key = find_master_edition_account(&mint_key).0;
        let mut me_account = tree.read_account(me_key).await.unwrap();
        let me: MasterEditionV2 =
            MasterEditionV2::from_account_info(&AccountInfo::from((&me_key, &mut me_account)))
                .unwrap();

        let expected_me = MasterEditionV2 {
            key: mpl_token_metadata::state::Key::MasterEditionV2,
            supply: 0,
            max_supply: Some(0),
        };
        assert_eq!(me, expected_me);
    }
}

#[tokio::test]
async fn test_create_public_tree_and_mint_passes() {
    // The mint operation implicitly called below also verifies that the on-chain tree
    // root matches the expected value as leaves are added.
    let mut context = BubblegumTestContext::new().await.unwrap();
    let tree = context
        .create_public_tree::<MAX_DEPTH, MAX_BUF_SIZE>()
        .await
        .unwrap();
    let tree_private = context
        .default_create_tree::<MAX_DEPTH, MAX_BUF_SIZE>()
        .await
        .unwrap();
    let payer = context.payer();
    let minter = Keypair::new(); // NON tree authority payer, nor delegate

    context
        .fund_account(minter.pubkey(), 10000000000)
        .await
        .unwrap();
    let cfg = tree.read_tree_config().await.unwrap();

    let name = format!("test{}", 0);
    let symbol = format!("tst{}", 0);
    let mut args = LeafArgs::new(&minter, context.default_metadata_args(name, symbol));

    assert_eq!(cfg.tree_creator, payer.pubkey());
    assert_eq!(cfg.tree_delegate, payer.pubkey());
    assert_eq!(cfg.total_mint_capacity, 1 << MAX_DEPTH);
    assert_eq!(cfg.is_public, true);

    tree.mint_v1_non_owner(&minter, &mut args).await.unwrap();
    let cfg = tree.read_tree_config().await.unwrap();
    assert_eq!(cfg.num_minted, 1);

    if let Err(BanksClient(BanksClientError::TransactionError(e))) =
        tree_private.mint_v1_non_owner(&minter, &mut args).await
    {
        assert_eq!(
            e,
            TransactionError::InstructionError(0, InstructionError::Custom(6016),)
        );
    } else {
        panic!("Should have failed");
    }
}
