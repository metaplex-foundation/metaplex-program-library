#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    state::{Key, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
    ID,
};
use num_traits::FromPrimitive;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use utils::*;

mod create {

    use std::str::FromStr;

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::{builders::CreateBuilder, CreateArgs, InstructionBuilder},
        state::{
            AssetData, Metadata, PrintSupply, ProgrammableConfig, TokenStandard, EDITION, PREFIX,
        },
    };
    use solana_program::borsh::try_from_slice_unchecked;

    use super::*;

    #[tokio::test]
    async fn create_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset details

        let name = puffed_out_string("Programmable NFT", MAX_NAME_LENGTH);
        let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
        let uri = puffed_out_string("uri", MAX_URI_LENGTH);

        let mut asset = AssetData::new(
            TokenStandard::ProgrammableNonFungible,
            name.clone(),
            symbol.clone(),
            uri.clone(),
        );
        asset.seller_fee_basis_points = 500;
        asset.rule_set =
            Some(Pubkey::from_str("Cex6GAMtCwD9E17VsEK4rQTbmcVtSdHxWcxhwdwXkuAN").unwrap());

        // build the mint transaction

        let payer_pubkey = context.payer.pubkey();
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();

        let program_id = ID;
        // metadata PDA address
        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (metadata, _) = Pubkey::find_program_address(metadata_seeds, &ID);
        // master edition PDA address
        let master_edition_seeds = &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_pubkey.as_ref(),
            EDITION.as_bytes(),
        ];
        let (master_edition, _) = Pubkey::find_program_address(master_edition_seeds, &ID);

        let create_ix = CreateBuilder::new()
            .metadata(metadata)
            .master_edition(master_edition)
            .mint(mint.pubkey())
            .authority(payer_pubkey)
            .payer(payer_pubkey)
            .update_authority(payer_pubkey)
            .initialize_mint(true)
            .update_authority_as_signer(true)
            .build(CreateArgs::V1 {
                asset_data: asset,
                decimals: Some(0),
                print_supply: Some(PrintSupply::Zero),
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &mint],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values

        let metadata_account = get_account(&mut context, &metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(metadata.data.name, name);
        assert_eq!(metadata.data.symbol, symbol);
        assert_eq!(metadata.data.uri, uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 500);
        assert_eq!(metadata.data.creators, None);

        assert!(!metadata.primary_sale_happened);
        assert!(metadata.is_mutable);
        assert_eq!(metadata.mint, mint_pubkey);
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);

        assert_eq!(
            metadata.token_standard,
            Some(TokenStandard::ProgrammableNonFungible)
        );
        assert_eq!(metadata.uses, None);
        assert_eq!(metadata.collection, None);
        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(rule_set),
        }) = metadata.programmable_config
        {
            assert_eq!(
                rule_set,
                Pubkey::from_str("Cex6GAMtCwD9E17VsEK4rQTbmcVtSdHxWcxhwdwXkuAN").unwrap()
            );
        } else {
            panic!("Missing programmable config");
        }
    }

    #[tokio::test]
    async fn create_programmable_nonfungible_with_existing_mint() {
        let mut context = program_test().start_with_context().await;

        // asset details

        let name = puffed_out_string("Programmable NFT", MAX_NAME_LENGTH);
        let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
        let uri = puffed_out_string("uri", MAX_URI_LENGTH);

        let mut asset = AssetData::new(
            TokenStandard::ProgrammableNonFungible,
            name.clone(),
            symbol.clone(),
            uri.clone(),
        );
        asset.seller_fee_basis_points = 500;
        /*
        asset.programmable_config = Some(ProgrammableConfig {
            rule_set: Pubkey::from_str("Cex6GAMtCwD9E17VsEK4rQTbmcVtSdHxWcxhwdwXkuAN")?,
        });
        */

        // creates the mint

        let payer_pubkey = context.payer.pubkey();
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();

        create_mint(&mut context, &mint, &payer_pubkey, Some(&payer_pubkey), 0)
            .await
            .unwrap();

        // build the create metadata transaction

        let program_id = ID;
        // metadata PDA address
        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (metadata, _) = Pubkey::find_program_address(metadata_seeds, &ID);
        // master edition PDA address
        let master_edition_seeds = &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_pubkey.as_ref(),
            EDITION.as_bytes(),
        ];
        let (master_edition, _) = Pubkey::find_program_address(master_edition_seeds, &ID);

        let create_ix = CreateBuilder::new()
            .metadata(metadata)
            .master_edition(master_edition)
            .mint(mint.pubkey())
            .authority(payer_pubkey)
            .payer(payer_pubkey)
            .update_authority(payer_pubkey)
            .initialize_mint(false)
            .update_authority_as_signer(true)
            .build(CreateArgs::V1 {
                asset_data: asset,
                decimals: Some(0),
                print_supply: Some(PrintSupply::Zero),
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values

        let metadata_account = get_account(&mut context, &metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(metadata.data.name, name);
        assert_eq!(metadata.data.symbol, symbol);
        assert_eq!(metadata.data.uri, uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 500);
        assert_eq!(metadata.data.creators, None);

        assert!(!metadata.primary_sale_happened);
        assert!(metadata.is_mutable);
        assert_eq!(metadata.mint, mint_pubkey);
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);

        assert_eq!(
            metadata.token_standard,
            Some(TokenStandard::ProgrammableNonFungible)
        );
        assert_eq!(metadata.uses, None);
        assert_eq!(metadata.collection, None);
        assert_eq!(
            metadata.programmable_config,
            Some(ProgrammableConfig::V1 { rule_set: None })
        );
    }

    #[tokio::test]
    async fn create_programmable_nonfungible_with_invalid_mint() {
        let mut context = program_test().start_with_context().await;

        // asset details

        let name = puffed_out_string("Programmable NFT", MAX_NAME_LENGTH);
        let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
        let uri = puffed_out_string("uri", MAX_URI_LENGTH);

        let mut asset = AssetData::new(
            TokenStandard::ProgrammableNonFungible,
            name.clone(),
            symbol.clone(),
            uri.clone(),
        );
        asset.seller_fee_basis_points = 500;
        /*
        asset.programmable_config = Some(ProgrammableConfig {
            rule_set: Pubkey::from_str("Cex6GAMtCwD9E17VsEK4rQTbmcVtSdHxWcxhwdwXkuAN")?,
        });
        */

        // creates the mint

        let payer_pubkey = context.payer.pubkey();
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();

        // invalid number of digits for non-fungible assets
        create_mint(&mut context, &mint, &payer_pubkey, Some(&payer_pubkey), 2)
            .await
            .unwrap();

        // build the create metadata transaction

        let program_id = ID;
        // metadata PDA address
        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (metadata, _) = Pubkey::find_program_address(metadata_seeds, &ID);
        // master edition PDA address
        let master_edition_seeds = &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_pubkey.as_ref(),
            EDITION.as_bytes(),
        ];
        let (master_edition, _) = Pubkey::find_program_address(master_edition_seeds, &ID);

        let create_ix = CreateBuilder::new()
            .metadata(metadata)
            .master_edition(master_edition)
            .mint(mint.pubkey())
            .authority(payer_pubkey)
            .payer(payer_pubkey)
            .update_authority(payer_pubkey)
            .initialize_mint(false)
            .update_authority_as_signer(true)
            .build(CreateArgs::V1 {
                asset_data: asset,
                decimals: Some(0),
                print_supply: Some(PrintSupply::Zero),
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let error = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();
        assert_custom_error!(error, MetadataError::InvalidMintForTokenStandard);
    }

    #[tokio::test]
    async fn create_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create(&mut context, TokenStandard::NonFungible, None)
            .await
            .unwrap();

        // checks the created metadata values

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(
            metadata.data.name,
            puffed_out_string(DEFAULT_NAME, MAX_NAME_LENGTH)
        );
        assert_eq!(
            metadata.data.symbol,
            puffed_out_string(DEFAULT_SYMBOL, MAX_SYMBOL_LENGTH)
        );
        assert_eq!(
            metadata.data.uri,
            puffed_out_string(DEFAULT_URI, MAX_URI_LENGTH)
        );
        assert!(metadata.data.creators.is_some());

        assert!(!metadata.primary_sale_happened);
        assert!(metadata.is_mutable);
        assert_eq!(metadata.mint, asset.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);

        assert_eq!(metadata.token_standard, Some(TokenStandard::NonFungible));
        assert_eq!(metadata.uses, None);
        assert_eq!(metadata.collection, None);
        assert_eq!(metadata.programmable_config, None);
        assert!(asset.edition.is_some());
    }

    #[tokio::test]
    async fn create_fungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create(&mut context, TokenStandard::Fungible, None)
            .await
            .unwrap();

        // checks the created metadata values

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(
            metadata.data.name,
            puffed_out_string(DEFAULT_NAME, MAX_NAME_LENGTH)
        );
        assert_eq!(
            metadata.data.symbol,
            puffed_out_string(DEFAULT_SYMBOL, MAX_SYMBOL_LENGTH)
        );
        assert_eq!(
            metadata.data.uri,
            puffed_out_string(DEFAULT_URI, MAX_URI_LENGTH)
        );
        assert!(metadata.data.creators.is_some());

        assert!(!metadata.primary_sale_happened);
        assert!(metadata.is_mutable);
        assert_eq!(metadata.mint, asset.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);

        assert_eq!(metadata.token_standard, Some(TokenStandard::Fungible));
        assert_eq!(metadata.uses, None);
        assert_eq!(metadata.collection, None);
        assert_eq!(metadata.programmable_config, None);
        assert!(asset.edition.is_none());
    }

    #[tokio::test]
    async fn create_fungible_asset() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create(&mut context, TokenStandard::FungibleAsset, None)
            .await
            .unwrap();

        // checks the created metadata values

        let metadata_account = get_account(&mut context, &asset.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(
            metadata.data.name,
            puffed_out_string(DEFAULT_NAME, MAX_NAME_LENGTH)
        );
        assert_eq!(
            metadata.data.symbol,
            puffed_out_string(DEFAULT_SYMBOL, MAX_SYMBOL_LENGTH)
        );
        assert_eq!(
            metadata.data.uri,
            puffed_out_string(DEFAULT_URI, MAX_URI_LENGTH)
        );
        assert!(metadata.data.creators.is_some());

        assert!(!metadata.primary_sale_happened);
        assert!(metadata.is_mutable);
        assert_eq!(metadata.mint, asset.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);

        assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));
        assert_eq!(metadata.uses, None);
        assert_eq!(metadata.collection, None);
        assert_eq!(metadata.programmable_config, None);
        assert!(asset.edition.is_none());
    }
}
