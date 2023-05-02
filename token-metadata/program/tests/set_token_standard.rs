#![cfg(feature = "test-bpf")]
pub mod utils;

use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    error::MetadataError,
    instruction::set_token_standard,
    state::{Metadata as ProgramMetadata, TokenStandard},
    ID as PROGRAM_ID,
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    account::AccountSharedData,
    instruction::InstructionError,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use utils::*;

#[tokio::test]
async fn successfully_update_nonfungible() {
    let mut context = program_test().start_with_context().await;

    // Create an old version NFT with no token standard set.
    let test_nft = Metadata::new();
    test_nft.create_v3_default(&mut context).await.unwrap();

    let master_edition = MasterEditionV2::new(&test_nft);
    master_edition
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    assert_eq!(metadata.token_standard, Some(TokenStandard::NonFungible));

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
    test_nft.create_v3_default(&mut context).await.unwrap();

    let master_edition = MasterEditionV2::new(&test_nft);
    master_edition
        .create_v3(&mut context, Some(10))
        .await
        .unwrap();

    let edition = EditionMarker::new(&test_nft, &master_edition, 1);
    edition.create(&mut context).await.unwrap();

    let mut md_account = get_account(&mut context, &edition.new_metadata_pubkey).await;
    let mut metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    // Modify token standard to be None and then inject account back into ProgramTestContext.
    metadata.token_standard = None;
    let mut data = metadata.try_to_vec().unwrap();
    let filler = vec![0u8; 679 - data.len()];
    data.extend_from_slice(&filler[..]);
    md_account.data = data;

    let md_account_shared_data: AccountSharedData = md_account.into();
    context.set_account(&edition.new_metadata_pubkey, &md_account_shared_data);

    let new_md_account = get_account(&mut context, &edition.new_metadata_pubkey).await;
    let new_metadata = ProgramMetadata::deserialize(&mut new_md_account.data.as_slice()).unwrap();

    assert_eq!(new_metadata.token_standard, None);

    let ix = set_token_standard(
        PROGRAM_ID,
        edition.new_metadata_pubkey,
        context.payer.pubkey(),
        edition.mint.pubkey(),
        Some(edition.new_edition_pubkey),
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let md_account = get_account(&mut context, &edition.new_metadata_pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    // Check that token standard has been updated successfully.
    assert_eq!(
        metadata.token_standard,
        Some(TokenStandard::NonFungibleEdition)
    );
}

#[tokio::test]
async fn successfully_update_fungible_asset() {
    let mut context = program_test().start_with_context().await;

    let test_nft = Metadata::new();
    test_nft.create_v3_default(&mut context).await.unwrap();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));

    let ix = set_token_standard(
        PROGRAM_ID,
        test_nft.pubkey,
        context.payer.pubkey(),
        test_nft.mint.pubkey(),
        None,
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
    assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));
}

#[tokio::test]
async fn successfully_update_fungible() {
    let mut context = program_test().start_with_context().await;

    let test_nft = Metadata::new();
    test_nft.create_v3_default(&mut context).await.unwrap();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));

    let ix = set_token_standard(
        PROGRAM_ID,
        test_nft.pubkey,
        context.payer.pubkey(),
        test_nft.mint.pubkey(),
        None,
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

    assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));
}

#[tokio::test]
async fn updating_without_authority_fails() {
    let mut context = program_test().start_with_context().await;

    // Create an old version NFT with no token standard set.
    let test_nft = Metadata::new();
    test_nft.create_v3_default(&mut context).await.unwrap();

    let master_edition = MasterEditionV2::new(&test_nft);
    master_edition
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let fake_authority = Keypair::new();

    let ix = set_token_standard(
        PROGRAM_ID,
        test_nft.pubkey,
        fake_authority.pubkey(),
        test_nft.mint.pubkey(),
        Some(master_edition.pubkey),
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &fake_authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);
}

#[tokio::test]
async fn mint_matches_metadata() {
    let mut context = program_test().start_with_context().await;

    // Create an old version NFT with no token standard set.
    let test_nft = Metadata::new();
    test_nft.create_v3_default(&mut context).await.unwrap();

    let master_edition = MasterEditionV2::new(&test_nft);
    master_edition
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let mut md_account = get_account(&mut context, &test_nft.pubkey).await;
    let mut metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    assert_eq!(metadata.token_standard, Some(TokenStandard::NonFungible));

    let invalid_mint = Keypair::new();

    // Modify metadata to have an invalid mint.
    metadata.mint = invalid_mint.pubkey();

    let mut data = metadata.try_to_vec().unwrap();
    let filler = vec![0u8; 679 - data.len()];
    data.extend_from_slice(&filler[..]);
    md_account.data = data;

    let md_account_shared_data: AccountSharedData = md_account.into();
    context.set_account(&test_nft.pubkey, &md_account_shared_data);

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
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::MintMismatch);
}

#[tokio::test]
async fn incorrect_edition_fails() {
    let mut context = program_test().start_with_context().await;

    // Create an old version NFT with no token standard set.
    let test_nft = Metadata::new();
    test_nft.create_v3_default(&mut context).await.unwrap();

    // Other NFT so we can create a valid, but incorrect edition account.
    // This one won't match the mint passed in.
    let test_nft2 = Metadata::new();
    test_nft2.create_v3_default(&mut context).await.unwrap();

    let wrong_master_edition = MasterEditionV2::new(&test_nft2);
    wrong_master_edition
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));

    let ix = set_token_standard(
        PROGRAM_ID,
        test_nft.pubkey,
        context.payer.pubkey(),
        test_nft.mint.pubkey(),
        Some(wrong_master_edition.pubkey),
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::DerivedKeyInvalid);
}

#[tokio::test]
async fn invalid_edition_fails() {
    let mut context = program_test().start_with_context().await;

    // Create an old version NFT with no token standard set.
    let test_nft = Metadata::new();
    test_nft.create_v3_default(&mut context).await.unwrap();

    let invalid_edition = Keypair::new().pubkey();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));

    let ix = set_token_standard(
        PROGRAM_ID,
        test_nft.pubkey,
        context.payer.pubkey(),
        test_nft.mint.pubkey(),
        Some(invalid_edition),
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::IncorrectOwner);
}

#[tokio::test]
async fn updating_nonfungible_without_edition_fails() {
    let mut context = program_test().start_with_context().await;

    // Create an old version NFT with no token standard set.
    let test_nft = Metadata::new();
    test_nft.create_v3_default(&mut context).await.unwrap();

    let master_edition = MasterEditionV2::new(&test_nft);
    master_edition
        .create_v3(&mut context, Some(0))
        .await
        .unwrap();

    let md_account = get_account(&mut context, &test_nft.pubkey).await;
    let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();

    assert_eq!(metadata.token_standard, Some(TokenStandard::NonFungible));

    let ix = set_token_standard(
        PROGRAM_ID,
        test_nft.pubkey,
        context.payer.pubkey(),
        test_nft.mint.pubkey(),
        None,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::MissingEditionAccount);
}
