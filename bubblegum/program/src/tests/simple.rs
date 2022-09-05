use solana_program::pubkey::Pubkey;
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;

use crate::state::metaplex_adapter::{Creator, MetadataArgs, TokenProgramVersion};
use crate::tests::clone_keypair;

use super::{program_test, Error, LeafArgs, Result, Tree};

async fn fund_account(ctx: &mut ProgramTestContext, address: Pubkey) -> Result<()> {
    let payer = &ctx.payer;

    // Create a transaction to send some funds to the `new_owner` account, which is used
    // as a payer in one of the operations below. Having the payer be an account with no
    // funds causes the Banks server to hang. Will find a better way to implement this
    // op.
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &payer.pubkey(),
            &address,
            1_000_000,
        )],
        Some(&payer.pubkey()),
        &[payer],
        ctx.last_blockhash,
    );

    ctx.banks_client
        .process_transaction(tx)
        .await
        .map_err(Error::BanksClient)
}

// Test for multiple combinations?
const MAX_DEPTH: usize = 20;
const MAX_BUF_SIZE: usize = 64;

// TODO: test signer conditions on mint_authority and other stuff that's manually checked
// and not by anchor (what else is there?)
#[tokio::test]
async fn test_simple() {
    let mut context = program_test().start_with_context().await;

    let creator_kp = [
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
    ];

    for k in creator_kp.iter() {
        fund_account(&mut context, k.pubkey()).await.unwrap();
    }

    // Cloning to avoid borrow checker annoyance w.r.t. context references.
    let payer = &clone_keypair(&context.payer);

    let mut tree =
        Tree::<MAX_DEPTH, MAX_BUF_SIZE>::with_creator(payer, context.banks_client.clone());

    tree.alloc(payer).await.unwrap();

    tree.create(payer).await.unwrap();

    tree.set_default_mint_request(1024 * 1024).await.unwrap();

    // println!("*** tree config {:?}", tree.read_tree_config().await);
    //
    // println!(
    //     "*** mint_auth_req {:?}",
    //     tree.read_mint_authority_request(&tree.authority()).await
    // );

    tree.approve_mint_request(tree.mint_authority_request(&tree.authority()), 1)
        .await
        .unwrap();

    // println!(
    //     "*** mint_auth_req {:?}",
    //     tree.read_mint_authority_request(&tree.authority()).await
    // );

    let message = MetadataArgs {
        name: "test".to_owned(),
        symbol: "tst".to_owned(),
        uri: "www.solana.pos".to_owned(),
        seller_fee_basis_points: 0,
        primary_sale_happened: false,
        is_mutable: false,
        edition_nonce: None,
        token_standard: None,
        token_program_version: TokenProgramVersion::Original,
        collection: None,
        uses: None,
        creators: vec![
            Creator {
                address: creator_kp[0].pubkey(),
                verified: false,
                share: 20,
            },
            Creator {
                address: creator_kp[1].pubkey(),
                verified: false,
                share: 20,
            },
            Creator {
                address: creator_kp[2].pubkey(),
                verified: false,
                share: 20,
            },
            Creator {
                address: creator_kp[3].pubkey(),
                verified: false,
                share: 40,
            },
        ],
    };

    let mut args = LeafArgs::new(payer, message.clone());

    tree.mint_v1(tree.authority(), &args).await.unwrap();

    tree.verify_creator(&args, &creator_kp[0]).await.unwrap();
    // Reflect change.
    args.metadata.creators[0].verified = true;

    tree.unverify_creator(&args, &creator_kp[0]).await.unwrap();
    args.metadata.creators[0].verified = false;

    let mut args2 = LeafArgs::new(payer, message.clone());
    args2.metadata.name = "test2".to_owned();

    // Will fail bc we only approved one mint request (for best testing we can
    // verify the error is the expected one, and not just any error).
    assert!(tree.mint_v1(tree.authority(), &args2).await.is_err());

    // Test delegate.
    {
        let new_nft_delegate = Keypair::new();

        let mut tx = tree
            .delegate_tx(&args, new_nft_delegate.pubkey())
            .await
            .unwrap();

        // We just instantiated the tx builder, but haven't attempted to execute
        // the transaction yet.

        // The tx above will succeed by default, but let's make it fail by providing
        // invalid signers.
        assert!(tx
            .set_signers(&[&new_nft_delegate])
            .execute()
            .await
            .is_err());

        // Finally it will work.
        tx.set_signers(&[payer]).execute().await.unwrap();
        // Reflect changes.
        args.delegate = new_nft_delegate;
    }

    // Test transfer
    {
        let new_owner = Keypair::new();

        // Funding the new owner account because it will be the payer
        // of the transfer transaction. Show the owner be the payer actually?
        fund_account(&mut context, new_owner.pubkey())
            .await
            .unwrap();

        let mut tx = tree.transfer_tx(&args, new_owner.pubkey()).await.unwrap();

        // Changing one of the accounts to something random to see the
        // tx fail.
        tx.accounts.owner = Keypair::new().pubkey();
        assert!(tx.execute().await.is_err());

        // Changing it back to the proper value, so it should succeed.
        tx.accounts.owner = payer.pubkey();
        tx.execute().await.unwrap();

        // Both owner and delegate change post transfer.
        args.owner = clone_keypair(&new_owner);
        args.delegate = new_owner;
    }

    // Test burn.
    tree.burn(&args).await.unwrap();

    // Test setting a new tree delegate.
    let new_tree_delegate = Keypair::new();
    tree.set_tree_delegate(&new_tree_delegate).await.unwrap();
}
