#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::payload::{PayloadType, SeedsVec};
use mpl_token_metadata::{
    error::MetadataError,
    instruction::{DelegateRole, TransferArgs},
    pda::find_delegate_account,
    state::{DelegateRecord, Key, PayloadKey, TokenStandard},
};
use num_traits::FromPrimitive;
// use rooster;
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
            delegate_record: None,
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
            delegate_record: None,
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
            delegate_record: None,
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

        da.delegate(
            &mut context,
            authority,
            delegate.pubkey(),
            DelegateRole::Transfer,
            Some(1),
        )
        .await
        .unwrap();

        let metadata = da.get_metadata(&mut context).await;
        assert_eq!(
            metadata.persistent_delegate.unwrap(),
            DelegateRole::Transfer
        );

        // delegate PDA
        let (delegate_record, _) = find_delegate_account(
            &da.mint.pubkey(),
            DelegateRole::Transfer,
            &authority_pubkey,
            &delegate.pubkey(),
        );

        let pda = get_account(&mut context, &delegate_record).await;
        let delegate_record_data: DelegateRecord = DelegateRecord::from_bytes(&pda.data).unwrap();
        assert_eq!(delegate_record_data.key, Key::Delegate);
        assert_eq!(delegate_record_data.role, DelegateRole::Transfer);

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
            delegate_record: Some(delegate_record),
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

        // Sanity check.
        let fake_delegate = Keypair::new();
        airdrop(&mut context, &fake_delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        // Associated token account already exists so we pass it in,
        // otherwise we will get an "IllegalOwner" errror.

        let params = TransferParams {
            context: &mut context,
            authority: &fake_delegate,
            delegate_record: Some(delegate_record),
            source_owner,
            destination_owner,
            destination_token: Some(destination_token),
            authorization_rules: None,
            payer: &fake_delegate,
            args,
        };

        let err = da.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(0, err, MetadataError::InvalidDelegate);
    }
}

mod auth_rules_transfer {
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

        if let Some(config) = metadata.programmable_config {
            assert_eq!(config.rule_set, Some(rule_set));
        } else {
            panic!("Missing programmable config");
        }

        let transfer_amount = 1;

        // Our first destination will be an account owned by
        // the mpl-token-METADATA. This should fail because it's not
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
            delegate_record: None,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: authority,
            args: args.clone(),
        };

        let err = nft.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(
            1,
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
            delegate_record: None,
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
    }

    #[tokio::test]
    async fn owner_transfer_system_wallet_to_pda() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        // Create NFT for owning the TOE account.
        // Create a NonFungible token using the old handlers.
        let mut toe_nft = DigitalAsset::new();
        toe_nft
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer
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

        // Our first destination will be the an account owned by
        // the mpl-token-metadata. This should fail because it's not
        // in the allowlist and also not a wallet-to-wallet transfer.
        let destination_owner = toe_nft.metadata;

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority,
            delegate_record: None,
            source_owner: &authority.pubkey(),
            destination_owner,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: authority,
            args: args.clone(),
        };

        let err = nft.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(
            1,
            err,
            mpl_token_auth_rules::error::RuleSetError::ProgramOwnedListCheckFailed
        );

        // Now we do a system_wallet_to_pda transfer. This should succeed.

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Update auth data payload with the seeds of the PDA we're transferring
        // to.
        let seeds = SeedsVec {
            seeds: vec![
                String::from("rule_set").as_bytes().to_vec(),
                authority.pubkey().as_ref().to_vec(),
                "Metaplex Royalty Enforcement".as_bytes().to_vec(),
            ],
        };

        auth_data.payload.insert(
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(seeds),
        );

        let args = TransferArgs::V1 {
            authorization_data: Some(auth_data),
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority,
            delegate_record: None,
            source_owner: &authority.pubkey(),
            destination_owner: rule_set,
            destination_token: None,
            authorization_rules: Some(rule_set),
            payer: authority,
            args: args.clone(),
        };

        nft.transfer(params).await.unwrap();

        let source_token_account = spl_token::state::Account::unpack(
            &context
                .banks_client
                .get_account(nft.token.unwrap())
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();

        let destination_token = get_associated_token_address(&rule_set, &nft.mint.pubkey());
        let dest_token_account = spl_token::state::Account::unpack(
            &context
                .banks_client
                .get_account(destination_token)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();

        // Destination now has the token.
        assert_eq!(dest_token_account.amount, 1);
        // Source does not.
        assert_eq!(source_token_account.amount, 0);
    }
}
