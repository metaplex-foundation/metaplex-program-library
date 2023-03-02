#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::{builders::BurnBuilder, BurnArgs, DelegateArgs, InstructionBuilder},
    state::{
        Collection, CollectionDetails, Creator, Key, MasterEditionV2 as ProgramMasterEditionV2,
        Metadata as ProgramMetadata, PrintSupply, TokenMetadataAccount, TokenStandard,
    },
};
use num_traits::FromPrimitive;
use solana_program::{program_pack::Pack, pubkey::Pubkey};
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
    use mpl_token_metadata::{instruction::TransferArgs, pda::find_token_record_account};
    use solana_program::system_instruction;

    use super::*;

    #[tokio::test]
    async fn owner_burn() {
        // The owner of the token can burn it.
        let mut context = program_test().start_with_context().await;

        let update_authority = context.payer.dirty_clone();
        let owner = Keypair::new();
        owner.airdrop(&mut context, 1_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        // Transfer to a new owner so the update authority is separate.
        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: 1,
        };

        da.transfer(TransferParams {
            context: &mut context,
            authority: &update_authority,
            source_owner: &update_authority.pubkey(),
            destination_owner: owner.pubkey(),
            destination_token: None, // fn will create the ATA
            payer: &update_authority,
            authorization_rules: None,
            args,
        })
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        da.burn(&mut context, owner, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        da.assert_burned(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn owner_same_as_ua_can_burn() {
        // When the owner is the same as the update authority, the owner can burn.
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        da.burn(&mut context, owner, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        da.assert_burned(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn update_authority_cannot_burn() {
        let mut context = program_test().start_with_context().await;

        let update_authority = context.payer.dirty_clone();
        let owner = Keypair::new();
        owner.airdrop(&mut context, 1_000_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        // Transfer to a new owner so the update authority is separate.
        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: 1,
        };

        da.transfer(TransferParams {
            context: &mut context,
            authority: &update_authority,
            source_owner: &update_authority.pubkey(),
            destination_owner: owner.pubkey(),
            destination_token: None, // fn will create the ATA
            payer: &update_authority,
            authorization_rules: None,
            args,
        })
        .await
        .unwrap();

        // Try to burn with the update authority who is no longer the owner.
        let args = BurnArgs::V1 { amount: 1 };

        let err = da
            .burn(&mut context, update_authority, args, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn utility_delegate_burn() {
        let mut context = program_test().start_with_context().await;

        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        delegate.airdrop(&mut context, 1_000_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        da.delegate(
            &mut context,
            payer,
            delegate.pubkey(),
            DelegateArgs::UtilityV1 {
                amount: 1,
                authorization_data: None,
            },
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        da.burn(&mut context, delegate, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition, token and token record accounts are closed.
        da.assert_burned(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn staking_delegate_cannot_burn() {
        let mut context = program_test().start_with_context().await;

        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        delegate.airdrop(&mut context, 1_000_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        da.delegate(
            &mut context,
            payer,
            delegate.pubkey(),
            DelegateArgs::StakingV1 {
                amount: 1,
                authorization_data: None,
            },
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let err = da
            .burn(&mut context, delegate, args, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn sale_delegate_cannot_burn() {
        let mut context = program_test().start_with_context().await;

        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        delegate.airdrop(&mut context, 1_000_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        da.delegate(
            &mut context,
            payer,
            delegate.pubkey(),
            DelegateArgs::SaleV1 {
                amount: 1,
                authorization_data: None,
            },
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let err = da
            .burn(&mut context, delegate, args, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn locked_transfer_delegate_cannot_burn() {
        let mut context = program_test().start_with_context().await;

        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        delegate.airdrop(&mut context, 1_000_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        da.delegate(
            &mut context,
            payer,
            delegate.pubkey(),
            DelegateArgs::LockedTransferV1 {
                amount: 1,
                locked_address: delegate.pubkey(),
                authorization_data: None,
            },
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let err = da
            .burn(&mut context, delegate, args, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn owner_burn_token_account_must_match_mint() {
        // Try to burn NFT with a token account that does not match the mint.
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let mut other_da = DigitalAsset::new();
        other_da
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(owner.pubkey())
            .metadata(da.metadata)
            .edition(da.edition.unwrap())
            .mint(da.mint.pubkey())
            .token(other_da.token.unwrap());

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &owner],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);
    }

    #[tokio::test]
    async fn delegate_burn_token_account_must_match_mint() {
        // Try to burn NFT with a token account that does not match the mint.
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();
        let delegate = Keypair::new();
        delegate.airdrop(&mut context, 1_000_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        da.delegate(
            &mut context,
            owner.dirty_clone(),
            delegate.pubkey(),
            DelegateArgs::UtilityV1 {
                amount: 1,
                authorization_data: None,
            },
        )
        .await
        .unwrap();

        // We try three cases here:
        // 1. The new token account for the same mint but with the correct token record.
        // This will fail with InvalidAuthorityType since the token record will not match
        // the token account passed in.
        //
        // 2. The new token account for the same mint but with a fake token record.
        // We cannot manually create a valid PDA token record account so we just derive it
        // and pass it in. This makes it owned by the system program and will fail with
        // IncorrectOwner.
        //
        // 3. We pass in a token record account that is owned by Token Metadata but is not
        // a valid PDA. This fails with InvalidAuthorityType because it doesn't match the derivation.

        // 1.
        // Create a token account for a new wallet but the same mint.
        let new_wallet = Keypair::new();
        new_wallet
            .airdrop(&mut context, 1_000_000_000)
            .await
            .unwrap();

        let new_wallet_token = Keypair::new();

        let create_token_ix = system_instruction::create_account(
            &context.payer.pubkey(),
            &new_wallet_token.pubkey(),
            100_000_000,
            165,
            &spl_token::ID,
        );
        let init_token_ix = spl_token::instruction::initialize_account(
            &spl_token::ID,
            &new_wallet_token.pubkey(),
            &da.mint.pubkey(),
            &new_wallet.pubkey(),
        )
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(delegate.pubkey())
            .metadata(da.metadata)
            .edition(da.edition.unwrap())
            .mint(da.mint.pubkey())
            .token(new_wallet_token.pubkey())
            .token_record(da.token_record.unwrap());

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[create_token_ix, init_token_ix, burn_ix.clone()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &delegate, &new_wallet_token],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error_ix!(2, err, MetadataError::InvalidAuthorityType);

        // 2.
        let new_wallet = Keypair::new();
        new_wallet
            .airdrop(&mut context, 1_000_000_000)
            .await
            .unwrap();

        let new_wallet_token = Keypair::new();

        let create_token_ix = system_instruction::create_account(
            &new_wallet.pubkey(),
            &new_wallet_token.pubkey(),
            100_000_000,
            165,
            &spl_token::ID,
        );
        let init_token_ix = spl_token::instruction::initialize_account(
            &spl_token::ID,
            &new_wallet_token.pubkey(),
            &da.mint.pubkey(),
            &new_wallet.pubkey(),
        )
        .unwrap();

        // Valid token record: correctly derived but uninitialized and owned by the system program.
        let (new_wallet_token_record, _) =
            find_token_record_account(&da.mint.pubkey(), &new_wallet_token.pubkey());

        let mut builder = BurnBuilder::new();
        builder
            .authority(delegate.pubkey())
            .metadata(da.metadata)
            .edition(da.edition.unwrap())
            .mint(da.mint.pubkey())
            .token(new_wallet_token.pubkey())
            .token_record(new_wallet_token_record); // Match token and record so we bypass this check.

        let transaction = Transaction::new_signed_with_payer(
            &[create_token_ix, init_token_ix, burn_ix.clone()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &delegate, &new_wallet, &new_wallet_token],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error_ix!(2, err, MetadataError::IncorrectOwner);

        // 3.
        let new_wallet = Keypair::new();
        new_wallet
            .airdrop(&mut context, 1_000_000_000)
            .await
            .unwrap();

        let new_wallet_token = Keypair::new();

        let create_token_ix = system_instruction::create_account(
            &context.payer.pubkey(),
            &new_wallet_token.pubkey(),
            100_000_000,
            165,
            &spl_token::ID,
        );
        let init_token_ix = spl_token::instruction::initialize_account(
            &spl_token::ID,
            &new_wallet_token.pubkey(),
            &da.mint.pubkey(),
            &new_wallet.pubkey(),
        )
        .unwrap();

        // Fake token record: owned by the Token Metadata program but not correctly derived.
        let fake_token_record = Keypair::new();
        let create_fake_token_record_ix = system_instruction::create_account(
            &context.payer.pubkey(),
            &fake_token_record.pubkey(),
            100_000_000,
            80,
            &mpl_token_metadata::ID,
        );

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(delegate.pubkey())
            .metadata(da.metadata)
            .edition(da.edition.unwrap())
            .mint(da.mint.pubkey())
            .token(new_wallet_token.pubkey())
            .token_record(fake_token_record.pubkey());

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[
                create_token_ix,
                init_token_ix,
                create_fake_token_record_ix,
                burn_ix.clone(),
            ],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &delegate,
                &new_wallet_token,
                &fake_token_record,
            ],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error_ix!(3, err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn owner_burn_metadata_must_match_mint() {
        // Try to burn NFT with a metadata account that does not match the mint.
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let mut other_da = DigitalAsset::new();
        other_da
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(owner.pubkey())
            .metadata(other_da.metadata)
            .edition(da.edition.unwrap())
            .mint(da.mint.pubkey())
            .token(da.token.unwrap());

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &owner],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);
    }

    #[tokio::test]
    async fn delegate_burn_metadata_must_match_mint() {
        // Try to burn NFT with a metadata that does not match the mint.
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();
        let delegate = Keypair::new();
        delegate.airdrop(&mut context, 1_000_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let mut other_da = DigitalAsset::new();
        other_da
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        da.delegate(
            &mut context,
            owner.dirty_clone(),
            delegate.pubkey(),
            DelegateArgs::UtilityV1 {
                amount: 1,
                authorization_data: None,
            },
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(delegate.pubkey())
            .metadata(other_da.metadata)
            .edition(da.edition.unwrap())
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .token_record(da.token_record.unwrap());

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix.clone()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &delegate],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error_ix!(0, err, MetadataError::MintMismatch);
    }

    #[tokio::test]
    async fn owner_burn_edition_must_match_mint() {
        // Try to burn NFT with an edition account that does not match the mint.
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let mut other_da = DigitalAsset::new();
        other_da
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(owner.pubkey())
            .metadata(da.metadata)
            .edition(other_da.edition.unwrap())
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .token_record(da.token_record.unwrap());

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &owner],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::DerivedKeyInvalid);
    }

    #[tokio::test]
    async fn delegate_burn_edition_must_match_mint() {
        // Try to burn NFT with an edition account that does not match the mint.
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();
        let delegate = Keypair::new();
        delegate.airdrop(&mut context, 1_000_000_000).await.unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        da.delegate(
            &mut context,
            owner.dirty_clone(),
            delegate.pubkey(),
            DelegateArgs::UtilityV1 {
                amount: 1,
                authorization_data: None,
            },
        )
        .await
        .unwrap();

        let mut other_da = DigitalAsset::new();
        other_da
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(delegate.pubkey())
            .metadata(da.metadata)
            .edition(other_da.edition.unwrap())
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .token_record(da.token_record.unwrap());

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &delegate],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::DerivedKeyInvalid);
    }
}

mod nft {
    use super::*;

    #[tokio::test]
    async fn burn_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

        let mut da = DigitalAsset::new();
        da.create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        da.burn(&mut context, owner, args, None, None)
            .await
            .unwrap();

        // Assert that metadata, edition and token account are closed.
        da.assert_burned(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn burning_decrements_collection_size() {
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

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

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                Some(collection),
                None,
                None, // Collection Item
            )
            .await
            .unwrap();

        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::safe_deserialize(parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 0);
                }
            }
        } else {
            panic!("CollectionDetails is not set!");
        }

        // Verifying increments the size.
        collection_item_nft
            .verify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        // Will look here, this is causing the problem.
        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::safe_deserialize(parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 1);
                }
            }
        } else {
            panic!("CollectionDetails is not set!");
        }

        let mut da: DigitalAsset = collection_item_nft
            .into_digital_asset(&mut context, Some(item_master_edition_account.pubkey))
            .await;

        // Burn the NFT
        da.burn(
            &mut context,
            owner,
            BurnArgs::V1 { amount: 1 },
            None,
            Some(collection_parent_nft.pubkey),
        )
        .await
        .unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::safe_deserialize(parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 0);
                }
            }
        } else {
            panic!("CollectionDetails is not set!");
        }
    }

    #[tokio::test]
    async fn burn_unsized_collection_item() {
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

        // Create a Collection Parent NFT without the CollectionDetails struct
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3_default(&mut context)
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

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                Some(collection),
                None,
                None,
            )
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Verifying collection
        collection_item_nft
            .verify_collection(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let mut da: DigitalAsset = collection_item_nft
            .into_digital_asset(&mut context, Some(item_master_edition_account.pubkey))
            .await;

        // Burn the NFT
        da.burn(
            &mut context,
            owner,
            BurnArgs::V1 { amount: 1 },
            None,
            Some(collection_parent_nft.pubkey),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn burn_unsized_collection_item_with_burned_parent() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT without the CollectionDetails struct
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3_default(&mut context)
            .await
            .unwrap();

        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Create a dummy Collection Parent NFT
        let dummy_collection_parent_nft = Metadata::new();
        dummy_collection_parent_nft
            .create_v3_default(&mut context)
            .await
            .unwrap();

        let dummy_parent_master_edition_account =
            MasterEditionV2::new(&dummy_collection_parent_nft);
        dummy_parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection = Collection {
            key: collection_parent_nft.mint.pubkey(),
            verified: false,
        };

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                Some(collection),
                None,
                None,
            )
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Verifying collection
        collection_item_nft
            .verify_collection(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let collection_metadata = collection_parent_nft.pubkey;

        let mut parent: DigitalAsset = collection_parent_nft
            .into_digital_asset(&mut context, Some(parent_master_edition_account.pubkey))
            .await;

        let mut item: DigitalAsset = collection_item_nft
            .into_digital_asset(&mut context, Some(item_master_edition_account.pubkey))
            .await;

        let owner = context.payer.dirty_clone();

        parent
            .burn(
                &mut context,
                owner.dirty_clone(),
                BurnArgs::V1 { amount: 1 },
                None,
                None,
            )
            .await
            .unwrap();

        // Fails to burn with invalid pubkey as collection
        let err = item
            .burn(
                &mut context,
                owner.dirty_clone(),
                BurnArgs::V1 { amount: 1 },
                None,
                Some(Pubkey::new_unique()),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::NotAMemberOfCollection);

        // Now we use a valid metadata account but one that doesn't match
        // the collection item.
        let err = item
            .burn(
                &mut context,
                owner.dirty_clone(),
                BurnArgs::V1 { amount: 1 },
                None,
                Some(dummy_collection_parent_nft.pubkey),
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::NotAMemberOfCollection);

        // Burn the NFT
        item.burn(
            &mut context,
            owner,
            BurnArgs::V1 { amount: 1 },
            None,
            Some(collection_metadata),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn fail_to_burn_master_edition_with_existing_prints() {
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

        let mut original_nft = DigitalAsset::new();
        original_nft
            .create_and_mint_nonfungible(&mut context, PrintSupply::Limited(10))
            .await
            .unwrap();

        let print_nft = original_nft.print_edition(&mut context, 1).await.unwrap();

        // Metadata, Print Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(print_nft.metadata)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(print_nft.token.unwrap())
            .await
            .unwrap();
        let print_edition_account = context
            .banks_client
            .get_account(print_nft.edition.unwrap())
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(print_edition_account.is_some());

        let err = original_nft
            .burn(&mut context, owner, BurnArgs::V1 { amount: 1 }, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MasterEditionHasPrints);
    }

    #[tokio::test]
    async fn require_md_account_to_burn_collection_nft() {
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

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

        let collection_item_nft = Metadata::new();
        collection_item_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                Some(collection),
                None,
                None, // Collection Item
            )
            .await
            .unwrap();
        let item_master_edition_account = MasterEditionV2::new(&collection_item_nft);
        item_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::safe_deserialize(parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 0);
                }
            }
        } else {
            panic!("CollectionDetails is not set!");
        }

        // Verifying increments the size.
        collection_item_nft
            .verify_sized_collection_item(
                &mut context,
                collection_parent_nft.pubkey,
                &payer,
                collection_parent_nft.mint.pubkey(),
                parent_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();

        let parent_nft_account = get_account(&mut context, &collection_parent_nft.pubkey).await;
        let parent_metadata =
            ProgramMetadata::safe_deserialize(parent_nft_account.data.as_slice()).unwrap();

        if let Some(details) = parent_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    assert_eq!(size, 1);
                }
            }
        } else {
            panic!("CollectionDetails is not set");
        }

        let mut da: DigitalAsset = collection_item_nft
            .into_digital_asset(&mut context, Some(item_master_edition_account.pubkey))
            .await;

        // Burn the NFT w/o passing in collection metadata. This should fail.
        let err = da
            .burn(&mut context, owner, BurnArgs::V1 { amount: 1 }, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MissingCollectionMetadata);
    }

    #[tokio::test]
    async fn only_owner_can_burn() {
        let mut context = program_test().start_with_context().await;

        let test_metadata = Metadata::new();
        test_metadata.create_v2_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&test_metadata);
        master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Metadata, Master Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(test_metadata.pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(test_metadata.token.pubkey())
            .await
            .unwrap();
        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(master_edition_account.is_some());

        let not_owner = Keypair::new();
        airdrop(&mut context, &not_owner.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        let mut item: DigitalAsset = test_metadata
            .into_digital_asset(&mut context, Some(master_edition.pubkey))
            .await;

        // Burn the NFT
        let err = item
            .burn(
                &mut context,
                not_owner,
                BurnArgs::V1 { amount: 1 },
                None,
                None,
            )
            .await
            .unwrap_err();

        // It won't register as a Holder or a Delgate so is invalid authority type.
        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn update_authority_cannot_burn() {
        let mut context = program_test().start_with_context().await;

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let creators = None;
        let seller_fee_basis_points = 10;
        let is_mutable = true;
        let collection = None;
        let uses = None;

        let test_metadata = Metadata::new();
        test_metadata
            .create_v3(
                &mut context,
                name.clone(),
                symbol.clone(),
                uri.clone(),
                creators.clone(),
                seller_fee_basis_points,
                is_mutable,
                collection.clone(),
                uses.clone(),
                None,
            )
            .await
            .unwrap();

        let master_edition = MasterEditionV2::new(&test_metadata);
        master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Metadata, Master Edition and token account exist.
        let md_account = context
            .banks_client
            .get_account(test_metadata.pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(test_metadata.token.pubkey())
            .await
            .unwrap();
        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap();

        assert!(md_account.is_some());
        assert!(token_account.is_some());
        assert!(master_edition_account.is_some());

        // NFT is created with context payer as the update authority so we need to update this first.
        let new_update_authority = Keypair::new();

        test_metadata
            .change_update_authority(&mut context, new_update_authority.pubkey())
            .await
            .unwrap();

        let mut item = test_metadata
            .into_digital_asset(&mut context, Some(master_edition.pubkey))
            .await;

        let err = item
            .burn(
                &mut context,
                new_update_authority,
                BurnArgs::V1 { amount: 1 },
                None,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    pub async fn cannot_burn_with_invalid_parents() {
        // Create two master editions and try burn the second with the first
        // as the parent accounts. This is using the handler wrong and would be
        // confusing for it to succeed even though it could, so it fails.

        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let second_nft = Metadata::new();
        second_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        // We need a valid edition marker for this test.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let second_master_edition = MasterEditionV2::new(&second_nft);
        second_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let owner = context.payer.dirty_clone();

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(owner.pubkey())
            .metadata(second_nft.pubkey)
            .edition(second_master_edition.pubkey)
            .mint(second_nft.mint.pubkey())
            .token(second_nft.token.pubkey())
            .master_edition_mint(original_nft.mint.pubkey())
            .master_edition_token(original_nft.token.pubkey())
            .master_edition(master_edition.pubkey)
            .edition_marker(print_edition.pubkey);

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &owner],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidParentAccounts);
    }
}

mod nft_edition {
    use super::*;

    #[tokio::test]
    async fn burn_nonfungible_edition() {
        let mut context = program_test().start_with_context().await;

        let nft = Metadata::new();
        let nft_master_edition = MasterEditionV2::new(&nft);
        let nft_edition_marker = EditionMarker::new(&nft, &nft_master_edition, 1);

        let payer_key = context.payer.pubkey();

        nft.create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            Some(vec![Creator {
                address: payer_key,
                verified: true,
                share: 100,
            }]),
            10,
            false,
            0,
        )
        .await
        .unwrap();

        nft_master_edition
            .create(&mut context, Some(10))
            .await
            .unwrap();

        nft_edition_marker.create(&mut context).await.unwrap();

        let edition_marker = nft_edition_marker.get_data(&mut context).await;
        let print_edition = get_account(&mut context, &nft_edition_marker.new_edition_pubkey).await;

        assert_eq!(edition_marker.ledger[0], 64);
        assert_eq!(edition_marker.key, Key::EditionMarker);
        assert_eq!(print_edition.data[0], 1);

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(context.payer.pubkey())
            .metadata(nft_edition_marker.new_metadata_pubkey)
            .edition(nft_edition_marker.new_edition_pubkey)
            .mint(nft_edition_marker.mint.pubkey())
            .token(nft_edition_marker.token.pubkey())
            .master_edition_mint(nft.mint.pubkey())
            .master_edition_token(nft.token.pubkey())
            .master_edition(nft_master_edition.pubkey)
            .edition_marker(nft_edition_marker.pubkey);

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // Metadata, and token account are burned.
        let print_md = context
            .banks_client
            .get_account(nft_edition_marker.new_metadata_pubkey)
            .await
            .unwrap();
        let token_account = context
            .banks_client
            .get_account(nft_edition_marker.token.pubkey())
            .await
            .unwrap();
        let print_edition_account = context
            .banks_client
            .get_account(nft_edition_marker.new_edition_pubkey)
            .await
            .unwrap();

        assert!(print_md.is_none());
        assert!(token_account.is_none());
        assert!(print_edition_account.is_none());
    }

    #[tokio::test]
    async fn burn_edition_nft_in_separate_wallet() {
        // Burn a print edition that is in a separate wallet, so owned by a different account
        // than the master edition nft.
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();
        let mut print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Transfer to new owner.
        let new_owner = Keypair::new();
        let new_owner_pubkey = new_owner.pubkey();
        airdrop(&mut context, &new_owner_pubkey, 1_000_000_000)
            .await
            .unwrap();

        context.warp_to_slot(10).unwrap();

        print_edition
            .transfer(&mut context, &new_owner_pubkey)
            .await
            .unwrap();

        let kpbytes = &context.payer;
        let payer = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Old owner should not be able to burn.
        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            print_edition.token.pubkey(),
            original_nft.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        // We've passed in the correct token account associated with the old owner but
        // it has 0 tokens so we get this error.
        assert_custom_error!(err, MetadataError::InsufficientTokenBalance);

        // Old owner should not be able to burn even if we pass in the new token
        // account associated with the new owner.
        let new_owner_token_account =
            get_associated_token_address(&new_owner_pubkey, &print_edition.mint.pubkey());

        let err = burn_edition(
            &mut context,
            print_edition.new_metadata_pubkey,
            &payer,
            print_edition.mint.pubkey(),
            original_nft.mint.pubkey(),
            new_owner_token_account,
            original_nft.token.pubkey(),
            master_edition.pubkey,
            print_edition.new_edition_pubkey,
            print_edition.pubkey,
        )
        .await
        .unwrap_err();

        // We've passed in the correct token account associated with the new owner but
        // the old owner is not the current owner of the account so this should fail with
        // InvalidOwner error.
        assert_custom_error!(err, MetadataError::InvalidOwner);

        let args = BurnArgs::V1 { amount: 1 };

        let mut builder = BurnBuilder::new();
        builder
            .authority(new_owner_pubkey)
            .metadata(print_edition.new_metadata_pubkey)
            .edition(print_edition.new_edition_pubkey)
            .mint(print_edition.mint.pubkey())
            .token(new_owner_token_account)
            .master_edition_mint(original_nft.mint.pubkey())
            .master_edition_token(original_nft.token.pubkey())
            .master_edition(master_edition.pubkey)
            .edition_marker(print_edition.pubkey);

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &new_owner],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        assert!(!print_edition.exists_on_chain(&mut context).await);
    }

    #[tokio::test]
    async fn only_owner_can_burn_edition() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v2_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);

        let not_owner = Keypair::new();
        airdrop(&mut context, &not_owner.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        let mut builder = BurnBuilder::new();
        builder
            .authority(not_owner.pubkey())
            .metadata(print_edition.new_metadata_pubkey)
            .edition(print_edition.new_edition_pubkey)
            .mint(print_edition.mint.pubkey())
            .token(print_edition.token.pubkey())
            .master_edition_mint(original_nft.mint.pubkey())
            .master_edition_token(original_nft.token.pubkey())
            .master_edition(master_edition.pubkey)
            .edition_marker(print_edition.pubkey);

        let default_args = BurnPrintArgs::default(&not_owner);

        let args = BurnPrintArgs {
            metadata: Some(print_edition.new_metadata_pubkey),
            edition: Some(print_edition.new_edition_pubkey),
            mint: Some(print_edition.mint.pubkey()),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(master_edition.pubkey),
            edition_marker: Some(print_edition.pubkey),
            ..default_args
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn update_authority_cannot_burn_edition() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v2_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        // NFT is created with context payer as the update authority so we need to update this before
        // creating the print edition, so it gets a copy of this new update authority.
        let new_update_authority = Keypair::new();

        original_nft
            .change_update_authority(&mut context, new_update_authority.pubkey())
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);

        let default_args = BurnPrintArgs::default(&new_update_authority);

        let args = BurnPrintArgs {
            metadata: Some(print_edition.new_metadata_pubkey),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(master_edition.pubkey),
            edition_marker: Some(print_edition.pubkey),
            ..default_args
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    pub async fn no_master_edition() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v2_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let payer = &context.payer.dirty_clone();

        let args = BurnPrintArgs {
            authority: payer,
            metadata: Some(print_edition.new_metadata_pubkey),
            edition: Some(print_edition.new_edition_pubkey),
            mint: Some(print_edition.mint.pubkey()),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(second_print_edition.pubkey),
            edition_marker: Some(print_edition.pubkey),
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        assert_custom_error!(err, MetadataError::NotAMasterEdition);
    }

    #[tokio::test]
    async fn invalid_master_edition() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let payer = &context.payer.dirty_clone();

        let args = BurnPrintArgs {
            authority: payer,
            metadata: Some(print_edition.new_metadata_pubkey),
            edition: Some(print_edition.new_edition_pubkey),
            mint: Some(print_edition.mint.pubkey()),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(Pubkey::new_unique()),
            edition_marker: Some(print_edition.pubkey),
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        // The random pubkey will be owned by the system program so will have an IncorrectOwner error.
        assert_custom_error!(err, MetadataError::IncorrectOwner);

        // Create a second master edition to try to pass off as the correct one. It's owned by token metadata
        // and has the right len of data, so will pass that check but will fail with InvalidMasterEdition because
        // it's derivation is incorrect.

        let new_nft = Metadata::new();
        new_nft.create_v3_default(&mut context).await.unwrap();

        let incorrect_master_edition = MasterEditionV2::new(&new_nft);
        incorrect_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let args = BurnPrintArgs {
            authority: payer,
            metadata: Some(print_edition.new_metadata_pubkey),
            edition: Some(print_edition.new_edition_pubkey),
            mint: Some(print_edition.mint.pubkey()),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(incorrect_master_edition.pubkey),
            edition_marker: Some(print_edition.pubkey),
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidMasterEdition);
    }

    #[tokio::test]
    pub async fn invalid_print_edition() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let payer = &context.payer.dirty_clone();

        let args = BurnPrintArgs {
            authority: payer,
            metadata: Some(print_edition.new_metadata_pubkey),
            edition: Some(Pubkey::new_unique()),
            mint: Some(print_edition.mint.pubkey()),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(master_edition.pubkey),
            edition_marker: Some(print_edition.pubkey),
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        // The random pubkey will have a data len of zero so is not a Print Edition.
        assert_custom_error!(err, MetadataError::IncorrectOwner);

        // Create a second print edition to try to pass off as the correct one. It's owned by token metadata
        // and has the right data length, so will pass those checks, but will fail with InvalidPrintEdition
        // because the derivation will be incorrect.

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let args = BurnPrintArgs {
            authority: payer,
            metadata: Some(print_edition.new_metadata_pubkey),
            edition: Some(second_print_edition.new_edition_pubkey),
            mint: Some(print_edition.mint.pubkey()),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(master_edition.pubkey),
            edition_marker: Some(print_edition.pubkey),
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidPrintEdition);
    }

    #[tokio::test]
    pub async fn invalid_edition_marker() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let payer = &context.payer.dirty_clone();

        let args = BurnPrintArgs {
            authority: payer,
            metadata: Some(print_edition.new_metadata_pubkey),
            edition: Some(print_edition.new_edition_pubkey),
            mint: Some(print_edition.mint.pubkey()),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(master_edition.pubkey),
            edition_marker: Some(Pubkey::new_unique()),
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        // The error will be IncorrectOwner since the random pubkey we generated is not a PDA owned
        // by the token metadata program.
        assert_custom_error!(err, MetadataError::IncorrectOwner);

        // Create a second print edition to try to pass off as the edition marker. It's owned by token metadata
        // so will pass that check but will fail with IncorrectEditionMarker.

        let second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let args = BurnPrintArgs {
            authority: payer,
            metadata: Some(print_edition.new_metadata_pubkey),
            edition: Some(print_edition.new_edition_pubkey),
            mint: Some(print_edition.mint.pubkey()),
            token: Some(print_edition.token.pubkey()),
            master_edition_mint: Some(original_nft.mint.pubkey()),
            master_edition_token: Some(original_nft.token.pubkey()),
            master_edition: Some(master_edition.pubkey),
            edition_marker: Some(second_print_edition.new_edition_pubkey),
        };

        let err = print_edition.burn(&mut context, args).await.unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidEditionMarker);
    }

    #[tokio::test]
    pub async fn master_supply_is_decremented() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEditionV2 =
            ProgramMasterEditionV2::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 1);
        assert!(master_edition_struct.max_supply == Some(10));

        let mut second_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct =
            ProgramMasterEditionV2::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 2);

        // Transfer second edition to a different owner.
        let user = Keypair::new();
        airdrop(&mut context, &user.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        context.warp_to_slot(10).unwrap();

        second_print_edition
            .transfer(&mut context, &user.pubkey())
            .await
            .unwrap();
        let new_owner_token_account =
            get_associated_token_address(&user.pubkey(), &second_print_edition.mint.pubkey());

        let payer = &context.payer.dirty_clone();

        let burn_print_args = BurnPrintArgs::default(payer);

        // Master edition owner burning.
        print_edition
            .burn(&mut context, burn_print_args)
            .await
            .unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct =
            ProgramMasterEditionV2::safe_deserialize(&master_edition_account.data).unwrap();

        // Master edition owner burning should decrement the supply.
        assert!(master_edition_struct.supply == 1);
        assert!(master_edition_struct.max_supply == Some(10));

        let default_args = BurnPrintArgs::default(&user);

        let burn_print_args = BurnPrintArgs {
            token: Some(new_owner_token_account),
            ..default_args
        };

        // Second owner burning.
        second_print_edition
            .burn(&mut context, burn_print_args)
            .await
            .unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct =
            ProgramMasterEditionV2::safe_deserialize(&master_edition_account.data).unwrap();

        // Second owner burning should decrement the supply.
        assert!(master_edition_struct.supply == 0);
    }

    #[tokio::test]
    pub async fn edition_mask_changed_correctly() {
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        context.warp_to_slot(10).unwrap();

        let print_editions = master_edition
            .mint_editions(&mut context, &original_nft, 10)
            .await
            .unwrap();

        let payer = &context.payer.dirty_clone();

        let edition_marker_account = context
            .banks_client
            .get_account(print_editions[1].pubkey)
            .await
            .unwrap()
            .unwrap();

        // Ledger is the 31 bytes after the key.
        let ledger = &edition_marker_account.data[1..];

        assert!(ledger[0] == 0b0111_1111);
        assert!(ledger[1] == 0b1110_0000);

        let default_args = BurnPrintArgs::default(payer);

        // Burn the second one
        print_editions[1]
            .burn(&mut context, default_args.clone())
            .await
            .unwrap();

        let edition_marker_account = context
            .banks_client
            .get_account(print_editions[1].pubkey)
            .await
            .unwrap()
            .unwrap();

        // Ledger is the 31 bytes after the key.
        let ledger = &edition_marker_account.data[1..];

        // One bit flipped here
        assert!(ledger[0] == 0b0101_1111);
        // None here
        assert!(ledger[1] == 0b1110_0000);

        // Burn the last one
        print_editions[9]
            .burn(&mut context, default_args)
            .await
            .unwrap();

        let edition_marker_account = context
            .banks_client
            .get_account(print_editions[1].pubkey)
            .await
            .unwrap()
            .unwrap();

        // Ledger is the 31 bytes after the key.
        let ledger = &edition_marker_account.data[1..];

        // Stays the same
        assert!(ledger[0] == 0b0101_1111);
        // One bit flipped
        assert!(ledger[1] == 0b1100_0000);
    }

    #[tokio::test]
    pub async fn reprint_burned_edition() {
        // Reprinting a burned edition should work when the owner is the same for
        // the master edition and print edition. Otherwise, it should fail.
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Print a new edition and transfer to a user.
        let mut user_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        user_print_edition.create(&mut context).await.unwrap();

        let user = Keypair::new();
        airdrop(&mut context, &user.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        context.warp_to_slot(10).unwrap();

        user_print_edition
            .transfer(&mut context, &user.pubkey())
            .await
            .unwrap();
        let new_owner_token_account =
            get_associated_token_address(&user.pubkey(), &user_print_edition.mint.pubkey());

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);
        assert!(user_print_edition.exists_on_chain(&mut context).await);

        let payer = &context.payer.dirty_clone();

        let owner_burn_args = BurnPrintArgs::default(payer);

        // Burn owner's edition.
        print_edition
            .burn(&mut context, owner_burn_args)
            .await
            .unwrap();

        let mut user_burn_args = BurnPrintArgs::default(&user);

        user_burn_args = BurnPrintArgs {
            token: Some(new_owner_token_account),
            ..user_burn_args
        };

        // Burn owner's edition.
        user_print_edition
            .burn(&mut context, user_burn_args)
            .await
            .unwrap();

        // Metadata, Print Edition and token account do not exist.
        assert!(!print_edition.exists_on_chain(&mut context).await);
        assert!(!user_print_edition.exists_on_chain(&mut context).await);

        // Reprint owner's burned edition
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);

        // Reprint user's burned edition: this should fail.
        let user_print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        let err = user_print_edition.create(&mut context).await.unwrap_err();

        assert_custom_error!(err, MetadataError::AlreadyInitialized);
    }

    #[tokio::test]
    async fn cannot_modify_wrong_master_edition() {
        let mut context = program_test().start_with_context().await;

        // Someone else's NFT
        let other_nft = Metadata::new();
        other_nft.create_v3_default(&mut context).await.unwrap();

        let other_master_edition = MasterEditionV2::new(&other_nft);
        other_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let new_update_authority = Keypair::new();
        other_nft
            .change_update_authority(&mut context, new_update_authority.pubkey())
            .await
            .unwrap();

        let other_print_edition = EditionMarker::new(&other_nft, &other_master_edition, 1);
        other_print_edition.create(&mut context).await.unwrap();

        let our_nft = Metadata::new();
        our_nft.create_v2_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&our_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&our_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        let payer = &context.payer.dirty_clone();

        let mut owner_burn_args = BurnPrintArgs::default(payer);

        owner_burn_args = BurnPrintArgs {
            master_edition_token: Some(other_nft.token.pubkey()),
            master_edition_mint: Some(other_nft.mint.pubkey()),
            master_edition: Some(other_master_edition.pubkey),
            edition_marker: Some(other_print_edition.pubkey),
            ..owner_burn_args
        };

        // We pass in our edition NFT and someone else's master edition and try to modify their supply.
        let err = print_edition
            .burn(&mut context, owner_burn_args)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::PrintEditionDoesNotMatchMasterEdition);
    }

    #[tokio::test]
    async fn mint_mismatches() {
        let mut context = program_test().start_with_context().await;

        let nft = Metadata::new();
        nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let print_edition = EditionMarker::new(&nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();
        let second_print_edition = EditionMarker::new(&nft, &master_edition, 2);
        second_print_edition.create(&mut context).await.unwrap();

        let payer = &context.payer.dirty_clone();

        let mut owner_burn_args = BurnPrintArgs::default(payer);

        // Wrong print edition mint account.
        owner_burn_args = BurnPrintArgs {
            mint: Some(second_print_edition.mint.pubkey()),
            ..owner_burn_args
        };

        let err = print_edition
            .burn(&mut context, owner_burn_args)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);

        let other_nft = Metadata::new();
        other_nft.create_v3_default(&mut context).await.unwrap();

        let other_master_edition = MasterEditionV2::new(&other_nft);
        other_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let mut owner_burn_args = BurnPrintArgs::default(payer);

        // Wrong master edition mint account.
        owner_burn_args = BurnPrintArgs {
            master_edition_mint: Some(other_nft.mint.pubkey()),
            ..owner_burn_args
        };

        let err = print_edition
            .burn(&mut context, owner_burn_args)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);
    }
}

mod fungible {
    use mpl_token_metadata::instruction::TransferArgs;
    use solana_program::native_token::LAMPORTS_PER_SOL;

    use super::*;

    #[tokio::test]
    async fn owner_burn() {
        let mut context = program_test().start_with_context().await;

        let update_authority = context.payer.dirty_clone();
        let owner = Keypair::new();
        owner.airdrop(&mut context, LAMPORTS_PER_SOL).await.unwrap();

        let initial_amount = 10;
        let burn_amount = 1;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::Fungible,
            None,
            None,
            initial_amount,
        )
        .await
        .unwrap();

        // Transfer to a new owner so the update authority is separate.
        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: 10,
        };

        da.transfer(TransferParams {
            context: &mut context,
            authority: &update_authority,
            source_owner: &update_authority.pubkey(),
            destination_owner: owner.pubkey(),
            destination_token: None, // fn will create the ATA
            payer: &update_authority,
            authorization_rules: None,
            args,
        })
        .await
        .unwrap();

        let args = BurnArgs::V1 {
            amount: burn_amount,
        };

        da.burn(&mut context, owner.dirty_clone(), args, None, None)
            .await
            .unwrap();

        // We only burned one token, so the token account should still exist.
        let token_account = context
            .banks_client
            .get_account(da.token.unwrap())
            .await
            .unwrap()
            .unwrap();

        let token = TokenAccount::unpack(&token_account.data).unwrap();

        assert_eq!(token.amount, initial_amount - burn_amount);

        let burn_remaining = initial_amount - burn_amount;

        let args = BurnArgs::V1 {
            amount: burn_remaining,
        };

        da.burn(&mut context, owner, args, None, None)
            .await
            .unwrap();

        // The token account should be closed now.
        let token_account = context
            .banks_client
            .get_account(da.token.unwrap())
            .await
            .unwrap();

        assert!(token_account.is_none());
    }

    #[tokio::test]
    async fn only_owner_can_burn() {
        let mut context = program_test().start_with_context().await;

        let not_owner = Keypair::new();

        not_owner
            .airdrop(&mut context, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let initial_amount = 10;
        let burn_amount = 1;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::Fungible,
            None,
            None,
            initial_amount,
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 {
            amount: burn_amount,
        };

        let err = da
            .burn(&mut context, not_owner.dirty_clone(), args, None, None)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn owner_token_must_match_mint() {
        let mut context = program_test().start_with_context().await;

        let owner = context.payer.dirty_clone();

        let initial_amount = 10;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::Fungible,
            None,
            None,
            initial_amount,
        )
        .await
        .unwrap();

        let mut other_da = DigitalAsset::new();
        other_da
            .create_and_mint(
                &mut context,
                TokenStandard::Fungible,
                None,
                None,
                initial_amount,
            )
            .await
            .unwrap();

        let args = BurnArgs::V1 {
            amount: initial_amount,
        };

        let mut builder = BurnBuilder::new();
        builder
            .authority(owner.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(other_da.token.unwrap());

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &owner],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MintMismatch);
    }
}
