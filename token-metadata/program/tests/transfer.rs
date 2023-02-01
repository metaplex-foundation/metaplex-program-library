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

        let params = TransferFromParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: authority,
            args,
        };

        da.transfer_from(params).await.unwrap();

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

        let params = TransferFromParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: authority,
            args,
        };

        da.transfer_from(params).await.unwrap();

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

        let params = TransferFromParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: authority,
            args,
        };

        da.transfer_from(params).await.unwrap();

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

        let params = TransferFromParams {
            context: &mut context,
            authority: &delegate,
            source_owner,
            destination_owner,
            destination_token: None,
            authorization_rules: None,
            payer: &payer,
            args: args.clone(),
        };

        da.transfer_from(params).await.unwrap();

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

        let params = TransferFromParams {
            context: &mut context,
            authority: &fake_delegate,
            source_owner,
            destination_owner,
            destination_token: Some(destination_token),
            authorization_rules: None,
            payer: &fake_delegate,
            args,
        };

        let err = da.transfer_from(params).await.unwrap_err();

        // Owner does not match.
        assert_custom_error_ix!(1, err, MetadataError::InvalidAuthorityType);
    }
}

mod auth_rules_transfer {
    use mpl_token_auth_rules::payload::Payload;
    use mpl_token_metadata::{
        instruction::DelegateArgs,
        state::{ProgrammableConfig, TokenDelegateRole}, error::MetadataError,
    };

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
        let (rule_set, auth_data) = create_default_metaplex_rule_set(&mut context, payer).await;

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

        let params = TransferFromParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: authority,
            args: args.clone(),
        };

        let err = nft.transfer_from(params).await.unwrap_err();

        assert_custom_error_ix!(
            2,
            err,
            mpl_token_auth_rules::error::RuleSetError::ProgramOwnedListCheckFailed
        );

        // Our second destination will be a wallet-to-wallet transfer so should
        // circumvent the program owned check and should succeed.
        let destination_owner = Pubkey::new_unique();

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let params = TransferFromParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: authority,
            args,
        };

        nft.transfer_from(params).await.unwrap();

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
        let (rule_set, mut auth_data) = create_default_metaplex_rule_set(&mut context, payer).await;

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

        let params = TransferFromParams {
            context: &mut context,
            authority: &authority,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer_from(params).await.unwrap();

        let destination_token =
            get_associated_token_address(&rooster_manager.pda(), &nft.mint.pubkey());
        let dest_token_account = spl_token::state::Account::unpack(
            get_account(&mut context, &destination_token)
                .await
                .data
                .as_slice(),
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(dest_token_account.amount, 1);

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

        // Now we withdraw from Rooster to test the pda-to-system-wallet transfer.
        rooster_manager
            .withdraw(
                &mut context,
                &authority,
                authority.pubkey(),
                nft.mint.pubkey(),
                nft.metadata,
                nft.master_edition.unwrap(),
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

        assert_eq!(authority_ata_account.amount, 1);
    }

    #[tokio::test]
    async fn transfer_delegate() {
        // Tests a delegate transferring from a system wallet to a PDA and vice versa.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // Create rule-set for the transfer; this has the Rooster program in the allowlist.
        let (rule_set, mut auth_data) = create_default_metaplex_rule_set(&mut context, payer).await;

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

        let params = TransferFromParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer_from(params).await.unwrap();

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
                nft.master_edition.unwrap(),
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

        let params = TransferToParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &rooster_manager.pda(),
            source_token: &rooster_ata,
            destination_owner: authority.pubkey(),
            destination_token: Some(nft.token.unwrap()),
            authorization_rules: Some(rule_set),
            payer: &delegate,
            args: args.clone(),
        };

        nft.transfer_to(params).await.unwrap();

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
    async fn sale_delegate() {
        // Tests a delegate transferring from a system wallet to a PDA and vice versa.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // Create rule-set for the transfer; this has the Rooster program in the allowlist.
        let (rule_set, mut auth_data) = create_default_metaplex_rule_set(&mut context, payer).await;

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

        let params = TransferFromParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer_from(params).await.unwrap();

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
                nft.master_edition.unwrap(),
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

        let params = TransferToParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &rooster_manager.pda(),
            source_token: &rooster_ata,
            destination_owner: authority.pubkey(),
            destination_token: Some(nft.token.unwrap()),
            authorization_rules: Some(rule_set),
            payer: &delegate,
            args: args.clone(),
        };

        nft.transfer_to(params).await.unwrap();

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

        let params = TransferFromParams {
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
        nft.transfer_from(params).await.unwrap();
    }

    #[tokio::test]
    async fn locked_transfer_delegate() {
        // tests a LoackedTransfer delegate transferring from a system wallet to an invalid address and
        // from a system wallet to the the locked PDA address
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        program_test.add_program("rooster", rooster::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = context.payer.dirty_clone();

        // create rule set for the transfer; this has the Rooster program in the allowlist
        let (rule_set, mut auth_data) = create_default_metaplex_rule_set(&mut context, payer).await;

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

        let params = TransferFromParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: nft.metadata,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        let error = nft.transfer_from(params).await.unwrap_err();

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

        let params = TransferFromParams {
            context: &mut context,
            authority: &delegate,
            source_owner: &authority.pubkey(),
            destination_owner: rooster_manager.pda(),
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: &authority,
            args: args.clone(),
        };

        nft.transfer_from(params).await.unwrap();

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
    }
}
