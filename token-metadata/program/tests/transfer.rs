#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::payload::{PayloadType, SeedsVec};
use mpl_token_metadata::{
    instruction::TransferArgs,
    state::{PayloadKey, TokenStandard},
};
use num_traits::FromPrimitive;
use rooster::instruction::DelegateArgs as RoosterDelegateArgs;
use solana_program::{native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::TransactionError,
};
use spl_associated_token_account::get_associated_token_address;
use utils::*;

mod standard_transfer {

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::{DelegateArgs, TransferArgs},
        state::TokenStandard,
    };
    use solana_program::{
        native_token::LAMPORTS_PER_SOL, program_option::COption, program_pack::Pack, pubkey::Pubkey,
    };
    use spl_associated_token_account::get_associated_token_address;

    use super::*;

    #[tokio::test]
    async fn transfer_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let mut da = DigitalAsset::new();
        da.create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let destination_owner = Keypair::new().pubkey();
        let destination_token = get_associated_token_address(&destination_owner, &da.mint.pubkey());
        airdrop(&mut context, &destination_owner, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: 1,
        };

        let params = TransferParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: authority,
            args,
        };

        da.transfer(params).await.unwrap();

        let token_account = spl_token::state::Account::unpack(
            &context
                .banks_client
                .get_account(destination_token)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();

        assert_eq!(token_account.amount, 1);
    }

    #[tokio::test]
    async fn transfer_fungible() {
        let mut context = program_test().start_with_context().await;

        let mint_amount = 10;
        let amount = 5;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::Fungible,
            None,
            None,
            mint_amount,
        )
        .await
        .unwrap();

        let destination_owner = Keypair::new().pubkey();
        let destination_token = get_associated_token_address(&destination_owner, &da.mint.pubkey());
        airdrop(&mut context, &destination_owner, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: authority,
            args,
        };

        da.transfer(params).await.unwrap();

        let token_account = spl_token::state::Account::unpack(
            &context
                .banks_client
                .get_account(destination_token)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();

        assert_eq!(token_account.amount, amount);
    }

    #[tokio::test]
    async fn transfer_fungible_asset() {
        let mut context = program_test().start_with_context().await;

        let mint_amount = 100;
        let transfer_amount = 99;

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            &mut context,
            TokenStandard::FungibleAsset,
            None,
            None,
            mint_amount,
        )
        .await
        .unwrap();

        let destination_owner = Pubkey::new_unique();
        let destination_token = get_associated_token_address(&destination_owner, &da.mint.pubkey());
        airdrop(&mut context, &destination_owner, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: authority,
            args,
        };

        da.transfer(params).await.unwrap();

        let token_account = spl_token::state::Account::unpack(
            &context
                .banks_client
                .get_account(destination_token)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();

        assert_eq!(token_account.amount, transfer_amount);
    }

    #[tokio::test]
    async fn transfer_with_delegate() {
        let mut context = program_test().start_with_context().await;

        let transfer_amount = 1;

        let mut da = DigitalAsset::new();
        da.create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let authority_pubkey = authority.pubkey();
        let source_owner = &Keypair::from_bytes(&context.payer.to_bytes())
            .unwrap()
            .pubkey();

        let delegate_args = DelegateArgs::StandardV1 {
            amount: transfer_amount,
        };

        da.delegate(&mut context, authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        let delegate_role = da
            .get_token_delegate_role(&mut context, &da.token.unwrap())
            .await;
        // Because this is a pass-through SPL token delegate there will be no role
        // set but the record will still exist.
        assert_eq!(delegate_role, None);

        // SPL delegate will exist.
        let authority_ata = get_associated_token_address(&authority_pubkey, &da.mint.pubkey());
        let authority_token_account = get_account(&mut context, &authority_ata).await;
        let authority_token: spl_token::state::Account =
            spl_token::state::Account::unpack(&authority_token_account.data).unwrap();

        assert_eq!(authority_token.delegate, COption::Some(delegate.pubkey()));

        let destination_owner = Pubkey::new_unique();
        let destination_token = get_associated_token_address(&destination_owner, &da.mint.pubkey());
        airdrop(&mut context, &destination_owner, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: transfer_amount,
        };

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner,
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: &payer,
            args: args.clone(),
        };

        da.transfer(params).await.unwrap();

        let token_account = spl_token::state::Account::unpack(
            &context
                .banks_client
                .get_account(destination_token)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();

        assert_eq!(token_account.amount, transfer_amount);
    }

    #[tokio::test]
    async fn fake_delegate_fails() {
        let mut context = program_test().start_with_context().await;

        let transfer_amount = 1;

        let mut da = DigitalAsset::new();
        da.create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let authority_pubkey = authority.pubkey();
        let source_owner = &Keypair::from_bytes(&context.payer.to_bytes())
            .unwrap()
            .pubkey();

        let delegate_args = DelegateArgs::StandardV1 {
            amount: transfer_amount,
        };

        da.delegate(&mut context, authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        let delegate_role = da
            .get_token_delegate_role(&mut context, &da.token.unwrap())
            .await;
        // Because this is a pass-through SPL token delegate there will be no role
        // set but the record will still exist.
        assert_eq!(delegate_role, None);

        // SPL delegate will exist.
        let authority_ata = get_associated_token_address(&authority_pubkey, &da.mint.pubkey());
        let authority_token_account = get_account(&mut context, &authority_ata).await;
        let authority_token: spl_token::state::Account =
            spl_token::state::Account::unpack(&authority_token_account.data).unwrap();

        assert_eq!(authority_token.delegate, COption::Some(delegate.pubkey()));

        let destination_owner = Pubkey::new_unique();
        let destination_token = get_associated_token_address(&destination_owner, &da.mint.pubkey());
        airdrop(&mut context, &destination_owner, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: transfer_amount,
        };

        let fake_delegate = Keypair::new();
        airdrop(&mut context, &fake_delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        // Associated token account already exists so we pass it in,
        // otherwise we will get an "IllegalOwner" errror.

        let params = TransferParams {
            context: &mut context,
            authority: &fake_delegate,
            source_owner,
            destination_owner,
            destination_token: Some(destination_token),
            authorization_rules: None,
            payer: &fake_delegate,
            args,
        };

        let err = da.transfer(params).await.unwrap_err();

        // Owner does not match.
        assert_custom_error_ix!(1, err, MetadataError::InvalidAuthorityType);
    }
}

mod auth_rules_transfer {
    use mpl_token_auth_rules::payload::Payload;
    use mpl_token_metadata::{
        error::MetadataError,
        instruction::DelegateArgs,
        pda::find_token_record_account,
        state::{ProgrammableConfig, TokenDelegateRole, TokenRecord},
    };
    use solana_program::borsh::try_from_slice_unchecked;
    use solana_sdk::transaction::Transaction;
    use spl_associated_token_account::instruction::create_associated_token_account;
    use spl_token::instruction::approve;

    use super::*;

    #[tokio::test]
    async fn wallet_to_wallet() {
        // Wallet to wallet should skip royalties rules, for now.

        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer requiring the destination to be program owned
        // by Token Metadata program. (Token Owned Escrow scenario.)
        let (rule_set, auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // Create NFT for transfer tests.
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let source_token = nft.token.unwrap();

        let metadata = nft.get_metadata(&mut context).await;
        assert_eq!(
            metadata.token_standard,
            Some(TokenStandard::ProgrammableNonFungible)
        );

        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(rule_set),
        }) = metadata.programmable_config
        {
            assert_eq!(rule_set, rule_set);
        } else {
            panic!("Missing programmable config");
        }

        let transfer_amount = 1;

        // Our first destination will be an account owned by
        // the mpl-token-metadata. This should fail because it's not
        // in the program allowlist and also not a wallet-to-wallet
        // transfer.
        let destination_owner = nft.metadata;

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: authority,
            args: args.clone(),
        };

        let err = nft.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(
            2,
            err,
            mpl_token_auth_rules::error::RuleSetError::ProgramOwnedListCheckFailed
        );

        // Our second destination will be a wallet-to-wallet transfer so should
        // circumvent the program owned check and should succeed.
        let destination_owner = Pubkey::new_unique();

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let params = TransferParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: authority,
            args,
        };

        nft.transfer(params).await.unwrap();

        let destination_token =
            get_associated_token_address(&destination_owner, &nft.mint.pubkey());

        let token_account = spl_token::state::Account::unpack(
            &context
                .banks_client
                .get_account(destination_token)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();

        assert_eq!(token_account.amount, transfer_amount);

        nft.assert_token_record_closed(&mut context, &source_token)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn owner_transfer() {
        // Tests an owner transferring from a system wallet to a PDA and vice versa.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // Create rule-set for the transfer; this has the Rooster program in the allowlist.
        let (rule_set, mut auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // Create NFT for transfer tests.
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let transfer_amount = 1;

        let authority = context.payer.dirty_clone();
        let rooster_manager = RoosterManager::init(&mut context, authority).await.unwrap();

        let authority = context.payer.dirty_clone();

        // Update auth data payload with the seeds of the PDA we're
        // transferring to.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        let source_token_record = nft.token_record.unwrap();

        let params = TransferParams {
            context: &mut context,
            authority: &authority,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer(params).await.unwrap();

        // Nft.token is updated by transfer to be the new token account where the asset currently
        let dest_token_account = spl_token::state::Account::unpack(
            get_account(&mut context, &nft.token.unwrap())
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        let source_token_record = context
            .banks_client
            .get_account(source_token_record)
            .await
            .unwrap();

        // Destination now has the token, and source accounts are closed.
        assert_eq!(dest_token_account.amount, 1);
        assert!(source_token_record.is_none());

        // Update auth data payload with the seeds of the PDA we're
        // transferring from.
        let mut payload = Payload::new();
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };
        payload.insert(
            PayloadKey::SourceSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        // Set the source to the current value.
        let source_token_record = nft.token_record.unwrap();

        // Now we withdraw from Rooster to test the pda-to-system-wallet transfer.
        rooster_manager
            .withdraw(
                &mut context,
                &authority,
                authority.pubkey(),
                nft.mint.pubkey(),
                nft.metadata,
                nft.edition.unwrap(),
                rule_set,
                payload,
            )
            .await
            .unwrap();

        let authority_ata = get_associated_token_address(&authority.pubkey(), &nft.mint.pubkey());
        let authority_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &authority_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        let source_token_record = context
            .banks_client
            .get_account(source_token_record)
            .await
            .unwrap();

        // Destination account for the withdraw now has the token.
        assert_eq!(authority_ata_account.amount, 1);

        // Rooster token record account closed.
        assert!(source_token_record.is_none());
    }

    #[tokio::test]
    async fn transfer_delegate() {
        // Tests a delegate transferring from a system wallet to a PDA and vice versa.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        program_test.set_compute_max_units(400_000);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // Create rule-set for the transfer; this has the Rooster program in the allowlist.
        let (rule_set, mut auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // Create NFT for transfer tests.
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let original_token = nft.token.unwrap();

        let transfer_amount = 1;

        // Create a transfer delegate
        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let delegate_args = DelegateArgs::TransferV1 {
            amount: transfer_amount,
            authorization_data: None,
        };

        nft.delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        let delegate_role = nft
            .get_token_delegate_role(&mut context, &nft.token.unwrap())
            .await;

        assert_eq!(delegate_role, Some(TokenDelegateRole::Transfer));

        // Set up the PDA account.
        let authority = context.payer.dirty_clone();
        let rooster_manager = RoosterManager::init(&mut context, authority).await.unwrap();

        let authority = context.payer.dirty_clone();

        // Update auth data payload with the seeds of the PDA we're
        // transferring to.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer(params).await.unwrap();

        let rooster_ata = get_associated_token_address(&rooster_manager.pda(), &nft.mint.pubkey());
        let rooster_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &rooster_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(rooster_ata_account.amount, 1);

        let rooster_delegate_args = RoosterDelegateArgs {
            amount: 1,
            bump: rooster_manager.bump(),
            authority: authority.pubkey(),
        };

        // Create new delegate using Rooster
        rooster_manager
            .delegate(
                &mut context,
                &delegate,
                nft.mint.pubkey(),
                nft.metadata,
                nft.edition.unwrap(),
                Some(rule_set),
                rooster_delegate_args,
            )
            .await
            .unwrap();

        // Update auth data payload with the seeds of the PDA we're
        // transferring from.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };
        auth_data.payload.insert(
            PayloadKey::SourceSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &rooster_manager.pda(),
            destination_owner: authority.pubkey(),
            destination_token: Some(original_token),
            authorization_rules: Some(rule_set),
            payer: &delegate,
            args: args.clone(),
        };

        nft.transfer(params).await.unwrap();

        let authority_ata = get_associated_token_address(&authority.pubkey(), &nft.mint.pubkey());
        let authority_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &authority_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(authority_ata_account.amount, 1);
    }

    #[tokio::test]
    async fn transfer_delegate_wrong_metadata() {
        // Tests a delegate transferring from a system wallet to a PDA and vice versa.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        program_test.set_compute_max_units(400_000);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // Create rule-set for the transfer; this has the Rooster program in the allowlist.
        let (rule_set, mut auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // Create NFT for transfer tests.
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let mut nft_naughty = DigitalAsset::new();
        nft_naughty
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        let transfer_amount = 1;

        // Create a transfer delegate
        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let delegate_args = DelegateArgs::TransferV1 {
            amount: transfer_amount,
            authorization_data: None,
        };

        nft.delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        let delegate_role = nft
            .get_token_delegate_role(&mut context, &nft.token.unwrap())
            .await;

        assert_eq!(delegate_role, Some(TokenDelegateRole::Transfer));

        // Set up the PDA account.
        let authority = context.payer.dirty_clone();
        let rooster_manager = RoosterManager::init(&mut context, authority).await.unwrap();

        let authority = context.payer.dirty_clone();

        // Update auth data payload with the seeds of the PDA we're
        // transferring to.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };
        nft.metadata = nft_naughty.metadata;
        let err = nft.transfer(params).await.unwrap_err();
        assert_custom_error_ix!(2, err, MetadataError::MintMismatch);
    }

    #[tokio::test]
    async fn sale_delegate() {
        // Tests a delegate transferring from a system wallet to a PDA and vice versa.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        program_test.set_compute_max_units(400_000);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // Create rule-set for the transfer; this has the Rooster program in the allowlist.
        let (rule_set, mut auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // Create NFT for transfer tests.
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let original_token = nft.token.unwrap();
        let transfer_amount = 1;

        // Create a sale delegate
        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let delegate_args = DelegateArgs::SaleV1 {
            amount: transfer_amount,
            authorization_data: Some(auth_data.clone()),
        };
        nft.delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        let delegate_role = nft
            .get_token_delegate_role(&mut context, &nft.token.unwrap())
            .await;

        assert_eq!(delegate_role, Some(TokenDelegateRole::Sale));

        // Set up the PDA account.
        let authority = context.payer.dirty_clone();
        let rooster_manager = RoosterManager::init(&mut context, authority).await.unwrap();

        let authority = context.payer.dirty_clone();

        // Update auth data payload with the seeds of the PDA we're
        // transferring to.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer(params).await.unwrap();

        let rooster_ata = get_associated_token_address(&rooster_manager.pda(), &nft.mint.pubkey());
        let rooster_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &rooster_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(rooster_ata_account.amount, 1);

        let rooster_delegate_args = RoosterDelegateArgs {
            amount: 1,
            bump: rooster_manager.bump(),
            authority: authority.pubkey(),
        };

        // Create new delegate using Rooster
        rooster_manager
            .delegate(
                &mut context,
                &delegate,
                nft.mint.pubkey(),
                nft.metadata,
                nft.edition.unwrap(),
                Some(rule_set),
                rooster_delegate_args,
            )
            .await
            .unwrap();

        // Update auth data payload with the seeds of the PDA we're
        // transferring from.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };
        auth_data.payload.insert(
            PayloadKey::SourceSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &rooster_manager.pda(),
            destination_owner: authority.pubkey(),
            destination_token: Some(original_token),
            authorization_rules: Some(rule_set),
            payer: &delegate,
            args: args.clone(),
        };

        nft.transfer(params).await.unwrap();

        let authority_ata = get_associated_token_address(&authority.pubkey(), &nft.mint.pubkey());
        let authority_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &authority_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(authority_ata_account.amount, 1);

        let rooster_manager_ata =
            get_associated_token_address(&rooster_manager.pda(), &nft.mint.pubkey());
        let rooster_manager_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &rooster_manager_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Source should not have a delegate.
        assert!(rooster_manager_ata_account.delegate.is_none());
    }

    #[tokio::test]
    async fn transfer_nft_with_utility_delegate_clears_close_authority() {
        // UtilityDelegates require setting the token account CloseAuthority to allow
        // the delegate to close the account. This test ensures that the CloseAuthority
        // is cleared after the transfer along with the rest of the delegate data.

        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        program_test.set_compute_max_units(400_000);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // Create rule-set for the transfer; this has the Rooster program in the allowlist.
        let (rule_set, mut auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // Create NFT for transfer tests.
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let transfer_amount = 1;

        // Create a utility delegate
        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let delegate_args = DelegateArgs::UtilityV1 {
            amount: transfer_amount,
            authorization_data: Some(auth_data.clone()),
        };
        nft.delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        let delegate_role = nft
            .get_token_delegate_role(&mut context, &nft.token.unwrap())
            .await;

        assert_eq!(delegate_role, Some(TokenDelegateRole::Utility));

        // Set up the PDA account.
        let authority = context.payer.dirty_clone();
        let rooster_manager = RoosterManager::init(&mut context, authority).await.unwrap();

        let authority = context.payer.dirty_clone();

        // Update auth data payload with the seeds of the PDA we're
        // transferring to.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        // We do an owner transfer because Utility Delegates can't transfer.
        let params = TransferParams {
            context: &mut context,
            authority: &authority,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer(params).await.unwrap();

        let rooster_ata = get_associated_token_address(&rooster_manager.pda(), &nft.mint.pubkey());
        let rooster_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &rooster_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(rooster_ata_account.amount, 1);

        // Check that the CloseAuthority is cleared.
        let authority_ata = get_associated_token_address(&authority.pubkey(), &nft.mint.pubkey());
        let source_token = spl_token::state::Account::unpack(
            get_account(&mut context, &authority_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();
        assert!(source_token.close_authority.is_none());
    }

    #[tokio::test]
    async fn no_auth_rules_skips_validation() {
        // Tests a pNFT with a rule_set of None skipping validation and still being
        // transferred correctly.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        // Create NFT for transfer tests.
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let transfer_amount = 1;

        // Our destination will be an account owned by the mpl-token-metadata
        // program. This will fail normally because it's not
        // in the program allowlist and also not a wallet-to-wallet
        // transfer. However, with no rule set present it should succeed because
        // there are no rules to validate.
        let destination_owner = nft.metadata;

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: authority,
            args: args.clone(),
        };

        // Transfer should succeed because no rule set is present on the NFT.
        nft.transfer(params).await.unwrap();
    }

    #[tokio::test]
    async fn locked_transfer_delegate() {
        // tests a LockedTransfer delegate transferring from a system wallet to an invalid address and
        // from a system wallet to the the locked PDA address
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        program_test.set_compute_max_units(400_000);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // create rule set for the transfer; this has the Rooster program in the allowlist
        let (rule_set, mut auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // create NFT for transfer tests
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let transfer_amount = 1;
        // Set up the PDA account.
        let authority = context.payer.dirty_clone();
        let rooster_manager = RoosterManager::init(&mut context, authority).await.unwrap();

        // Create a locked transfer delegate
        let payer = context.payer.dirty_clone();
        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let delegate_args = DelegateArgs::LockedTransferV1 {
            amount: transfer_amount,
            locked_address: rooster_manager.pda(),
            authorization_data: None,
        };

        nft.delegate(&mut context, payer, delegate.pubkey(), delegate_args)
            .await
            .unwrap();

        // asserts (before transfer)

        let pda = get_account(&mut context, &nft.token_record.unwrap()).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.rule_set_revision, Some(0));

        let delegate_role = nft
            .get_token_delegate_role(&mut context, &nft.token.unwrap())
            .await;

        assert_eq!(delegate_role, Some(TokenDelegateRole::LockedTransfer));

        // tries to make an invalid transfer: the destination address does not match
        // the address at the delegate creation

        let authority = context.payer.dirty_clone();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: nft.metadata,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        let error = nft.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(2, error, MetadataError::InvalidLockedTransferAddress);

        // makes the correct transfer

        let authority = context.payer.dirty_clone();

        // update auth data payload with the seeds of the PDA we're
        // transferring to.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer(params).await.unwrap();

        let rooster_ata = get_associated_token_address(&rooster_manager.pda(), &nft.mint.pubkey());
        let rooster_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &rooster_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(rooster_ata_account.amount, 1);

        // asserts (after transfer)

        let pda = get_account(&mut context, &nft.token_record.unwrap()).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.rule_set_revision, None);

        let destination_token =
            get_associated_token_address(&rooster_manager.pda(), &nft.mint.pubkey());

        let (destination_token_record, _bump) =
            find_token_record_account(&nft.mint.pubkey(), &destination_token);
        let pda = get_account(&mut context, &destination_token_record).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.rule_set_revision, None);
    }

    #[tokio::test]
    async fn escrowless_delegate_transfer() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        let source_owner = context.payer.dirty_clone().pubkey();
        let destination_owner = Pubkey::new_unique();
        airdrop(&mut context, &destination_owner, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        // create rule set for the transfer; this has the Rooster program in the allowlist
        let (rule_set, mut auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // create NFT for transfer tests
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let transfer_amount = 1;

        let authority = context.payer.dirty_clone();
        let rooster_manager = RoosterManager::init(&mut context, authority).await.unwrap();

        // Create a Sale Delegate for the NFT assigned to the Rooster PDA.
        let payer = context.payer.dirty_clone();
        airdrop(&mut context, &rooster_manager.pda(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let delegate_args = DelegateArgs::SaleV1 {
            amount: transfer_amount,
            authorization_data: None,
        };

        nft.delegate(&mut context, payer, rooster_manager.pda(), delegate_args)
            .await
            .unwrap();

        // makes the transfer

        let authority = context.payer.dirty_clone();

        // update auth data payload with the seeds of the authority PDA
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rooster").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::AuthoritySeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        rooster_manager
            .delegate_transfer(
                &mut context,
                &authority,
                source_owner,
                destination_owner,
                nft.mint.pubkey(),
                rule_set,
                auth_data.payload,
            )
            .await
            .unwrap();

        let source_ata = get_associated_token_address(&source_owner, &nft.mint.pubkey());
        let source_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &source_ata).await.data.as_slice(),
        )
        .unwrap();

        let destination_ata = get_associated_token_address(&destination_owner, &nft.mint.pubkey());
        let destination_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &destination_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(destination_ata_account.amount, 1);
        assert_eq!(source_ata_account.amount, 0);
    }

    #[tokio::test]
    async fn destination_token_matches_destination_owner() {
        // We ensure that the destination owner is linked to the destination token account
        // so that people cannot get around auth rules by passing in an owner that is in an allowlist
        // but doesn't actually correspond to the token account.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // create rule set for the transfer; this has the Rooster program in the allowlist
        let (rule_set, mut auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // create NFT for transfer tests
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        // We need a PDA from a program not in the allowlist to be the destination
        // owner.
        let actual_owner = nft.mint.pubkey();
        let destination_ata = get_associated_token_address(&actual_owner, &nft.mint.pubkey());

        let payer = context.payer.dirty_clone();

        // Create the ATA for the destination owner so it already exists for the transfer.
        let ix = create_associated_token_account(
            &payer.dirty_clone().pubkey(),
            &actual_owner,
            &nft.mint.pubkey(),
            &spl_token::ID,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let transfer_amount = 1;

        // update auth data payload with the seeds of the fake owner PDA
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rule_set").as_bytes().to_vec(),
                payer.pubkey().as_ref().to_vec(),
                nft.mint.pubkey().as_ref().to_vec(),
                String::from("Metaplex Royalty Enforcement")
                    .as_bytes()
                    .to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: transfer_amount,
        };

        let authority = context.payer.dirty_clone();

        // We transfer to the ATA of the actual owner,
        // but pass in a Token Metadata PDA as the destination owner as that program
        // is in the allowlist.
        let params = TransferParams {
            context: &mut context,
            authority: &authority,
            source_owner: &authority.pubkey(),
            destination_owner: rule_set,
            destination_token: Some(destination_ata),
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        let err = nft.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(1, err, MetadataError::InvalidOwner);
    }

    #[tokio::test]
    async fn invalid_close_authority_fails() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // create rule set for the transfer; this has the Rooster program in the allowlist
        let (rule_set, auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        assert!(asset.token.is_some());

        let delegate = Keypair::new();
        let delegate_pubkey = delegate.pubkey();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                delegate_pubkey,
                DelegateArgs::UtilityV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // To simulate the state where the close authority is set to the delegate instead of
        // the asset's master edition account, we need to inject modified token account state.
        asset
            .inject_close_authority(&mut context, &delegate_pubkey)
            .await;

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data.clone()),
            amount: 1,
        };

        let destination_owner = Pubkey::new_unique();
        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let params = TransferParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: authority,
            args,
        };

        let err = asset.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(2, err, MetadataError::InvalidCloseAuthority);
    }

    #[tokio::test]
    async fn clear_delegate_after_holder_transfer() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        let source_owner = context.payer.dirty_clone().pubkey();
        let destination_owner = Pubkey::new_unique();
        airdrop(&mut context, &destination_owner, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        // create rule set for the transfer; this has the Rooster program in the allowlist
        let (rule_set, auth_data) =
            create_default_metaplex_rule_set(&mut context, payer, false).await;

        // create NFT for transfer tests
        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
            Some(auth_data.clone()),
            1,
        )
        .await
        .unwrap();

        let authority = context.payer.dirty_clone();
        let rooster_manager = RoosterManager::init(&mut context, authority).await.unwrap();

        // Create a Transfer Delegate for the NFT assigned to the Rooster PDA.
        let payer = context.payer.dirty_clone();
        airdrop(&mut context, &rooster_manager.pda(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let delegate_args = DelegateArgs::TransferV1 {
            amount: 1,
            authorization_data: None,
        };

        nft.delegate(&mut context, payer, rooster_manager.pda(), delegate_args)
            .await
            .unwrap();

        // makes the transfer

        let payer = context.payer.dirty_clone();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: 1,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &payer,
            source_owner: &payer.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: &payer,
            args,
        };

        nft.transfer(params).await.unwrap();

        let destination_ata = get_associated_token_address(&destination_owner, &nft.mint.pubkey());
        let destination_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &destination_ata)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(destination_ata_account.amount, 1);

        let source_ata = get_associated_token_address(&source_owner, &nft.mint.pubkey());
        let source_ata_account = spl_token::state::Account::unpack(
            get_account(&mut context, &source_ata).await.data.as_slice(),
        )
        .unwrap();

        // Source delegate should be cleared.
        assert!(source_ata_account.delegate.is_none());
    }

    #[tokio::test]
    async fn delegate_on_destination_transfer_fails() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        // Create NFT for transfer tests.

        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        // Creates a destination token account and approves the delegate on it.

        let destination_owner = Keypair::new();
        let delegate = Pubkey::new_unique();
        let payer = context.payer.dirty_clone();

        let destination_token =
            get_associated_token_address(&destination_owner.pubkey(), &nft.mint.pubkey());

        let instructions = vec![
            create_associated_token_account(
                &payer.pubkey(),
                &destination_owner.pubkey(),
                &nft.mint.pubkey(),
                &spl_token::id(),
            ),
            approve(
                &spl_token::id(),
                &destination_token,
                &delegate,
                &destination_owner.pubkey(),
                &[],
                1,
            )
            .unwrap(),
        ];

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer, &destination_owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // makes the transfer (fails because the destination token account has a delegate)

        let payer = context.payer.dirty_clone();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: 1,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &payer,
            source_owner: &payer.pubkey(),
            destination_owner: destination_owner.pubkey(),
            destination_token: Some(destination_token),
            authorization_rules: None,
            payer: &payer,
            args,
        };

        let error = nft.transfer(params).await.unwrap_err();
        // error indicating that there is an existing delegate on the destination token account
        assert_custom_error_ix!(1, error, MetadataError::DelegateAlreadyExists);
    }
}
