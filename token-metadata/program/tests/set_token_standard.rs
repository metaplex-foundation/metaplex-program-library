#![cfg(feature = "test-bpf")]
pub mod utils;

use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    instruction::set_token_standard,
    state::{Metadata as ProgramMetadata, TokenStandard},
    ID as PROGRAM_ID,
};
use solana_program_test::*;
use solana_sdk::{
    account::AccountSharedData, signature::Keypair, signer::Signer, transaction::Transaction,
};
use utils::*;

#[tokio::test]
async fn successfully_update_nonfungible() {
    let mut context = program_test().start_with_context().await;

    // Create an old version NFT with no token standard set.
    let test_nft = Metadata::new();
    test_nft
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
        )
        .await
        .unwrap();

    let master_edition = MasterEditionV2::new(&test_nft);
    master_edition.create(&mut context, Some(0)).await.unwrap();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    // Check that token standard is not set.
    assert_eq!(metadata.token_standard, None);

    let ix = set_token_standard(
        PROGRAM_ID,
        test_nft.pubkey,
        context.payer.pubkey(),
        test_nft.mint.pubkey(),
        Some(master_edition.pubkey),
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    // Check that token standard has been updated successfully.
    assert_eq!(metadata.token_standard, Some(TokenStandard::NonFungible));
}

#[tokio::test]
async fn successfully_update_nonfungible_edition() {
    let mut context = program_test().start_with_context().await;

    // Create an old version NFT with no token standard set.
    let test_nft = Metadata::new();
    test_nft
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
        )
        .await
        .unwrap();

    let master_edition = MasterEditionV2::new(&test_nft);
    master_edition.create(&mut context, Some(10)).await.unwrap();

    let new_md = Keypair::new();

    let edition = EditionMarker::new(&test_nft, &master_edition, 1);
    edition.create(&mut context).await.unwrap();

    let mut md_account = get_account(&mut context, &edition.new_metadata_pubkey).await;
    let mut metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    println!("{:?}", md_account.data.to_vec()[327]);

    // Modify token standard to be None and then inject account back into ProgramTestContext.
    metadata.token_standard = None;
    metadata.serialize(&mut md_account.data).unwrap();

    let md_account_shared_data: AccountSharedData = md_account.into();
    context.set_account(&new_md.pubkey(), &md_account_shared_data);

    let new_md_account = get_account(&mut context, &new_md.pubkey()).await;
    let new_metadata = ProgramMetadata::deserialize(&mut new_md_account.data.as_slice()).unwrap();

    println!("{:?}", new_md_account.data.to_vec()[327]);

    // Check that token standard is not set.
    assert_eq!(new_metadata.token_standard, None);

    let ix = set_token_standard(
        PROGRAM_ID,
        test_nft.pubkey,
        context.payer.pubkey(),
        test_nft.mint.pubkey(),
        Some(edition.pubkey),
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    // Check that token standard has been updated successfully.
    assert_eq!(
        metadata.token_standard,
        Some(TokenStandard::NonFungibleEdition)
    );
}
