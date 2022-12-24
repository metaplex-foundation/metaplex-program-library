#![cfg(feature = "test-bpf")]

pub mod utils;

use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::TransactionError,
};
use utils::*;

mod transfer {

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::{create_escrow_account, DelegateRole, TransferArgs},
        processor::find_escrow_account,
        state::{EscrowAuthority, ProgrammableConfig, TokenStandard},
    };
    use solana_program::{native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey};
    use solana_sdk::transaction::Transaction;
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
    async fn transfer_programmable_wallet_to_wallet() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer requiring the destination to be program owned
        // by Token Metadata program. (Token Owned Escrow scenario.)
        let (rule_set, auth_data) =
            create_test_ruleset(&mut context, payer, "royalty".to_string()).await;

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
        assert_eq!(
            metadata.programmable_config,
            Some(ProgrammableConfig { rule_set })
        );

        let transfer_amount = 1;

        // Our first destination will be an account owned by
        // the mpl-token-auth-rules. This should fail because it's not
        // owned by the Token Metadata program and also not a wallet-to-wallet
        // transfer.
        let destination_owner = rule_set;

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
            args: args.clone(),
        };

        let err = nft.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(
            1,
            err,
            mpl_token_auth_rules::error::RuleSetError::ProgramOwnedCheckFailed
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
    async fn transfer_programmable_program_owned() {
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
        let (rule_set, auth_data) =
            create_test_ruleset(&mut context, payer, "royalty".to_string()).await;

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

        // Our first destination will be an account owned by
        // the mpl-token-auth-rules. This should fail because it's not
        // owned by the Token Metadata program and also not a wallet-to-wallet
        // transfer.
        let destination_owner = rule_set;

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
            args: args.clone(),
        };

        let err = nft.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(
            1,
            err,
            mpl_token_auth_rules::error::RuleSetError::ProgramOwnedCheckFailed
        );

        // Create TOE account and try to transfer to it. This should succeed.
        let (escrow_account, _) =
            find_escrow_account(&toe_nft.mint.pubkey(), &EscrowAuthority::TokenOwner);

        let create_escrow_ix = create_escrow_account(
            mpl_token_metadata::ID,
            escrow_account,
            toe_nft.metadata,
            toe_nft.mint.pubkey(),
            toe_nft.token.unwrap(),
            toe_nft.master_edition.unwrap(),
            context.payer.pubkey(),
            Some(context.payer.pubkey()),
        );

        let tx = Transaction::new_signed_with_payer(
            &[create_escrow_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let authority = &Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let params = TransferParams {
            context: &mut context,
            authority,
            source_owner: &authority.pubkey(),
            destination_owner: escrow_account,
            destination_token: None,
            authorization_rules: Some(rule_set),
            args,
        };

        nft.transfer(params).await.unwrap();

        let destination_token = get_associated_token_address(&escrow_account, &nft.mint.pubkey());

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
            metadata.delegate_state.unwrap().role,
            DelegateRole::Transfer
        );

        let destination_owner = Pubkey::new_unique();
        let destination_token = get_associated_token_address(&destination_owner, &da.mint.pubkey());
        airdrop(&mut context, &destination_owner, LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let args = TransferArgs::V1 {
            authorization_data: None,
            amount: transfer_amount,
        };

        let params = TransferParams {
            context: &mut context,
            authority: &delegate,
            source_owner,
            destination_owner,
            destination_token: None,
            authorization_rules: None,
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
            source_owner,
            destination_owner,
            destination_token: Some(destination_token),
            authorization_rules: None,
            args,
        };

        let err = da.transfer(params).await.unwrap_err();

        assert_custom_error_ix!(0, err, MetadataError::InvalidDelegate);
    }
}
