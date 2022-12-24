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
        instruction::{create_escrow_account, DelegateRole},
        processor::find_escrow_account,
        state::{EscrowAuthority, TokenStandard},
    };
    use solana_program::{native_token::LAMPORTS_PER_SOL, program_pack::Pack};
    use solana_sdk::transaction::Transaction;
    use spl_associated_token_account::get_associated_token_address;

    use super::*;

    #[tokio::test]
    async fn transfer_nonfungible() {
        let mut context = program_test().start_with_context().await;

        let mut digital_asset = DigitalAsset::new();
        digital_asset
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let destination = Keypair::new();
        let destination_token =
            get_associated_token_address(&destination.pubkey(), &digital_asset.mint.pubkey());
        airdrop(&mut context, &destination.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        digital_asset
            .transfer(
                &mut context,
                payer,
                destination.pubkey(),
                None,
                None,
                None,
                1,
            )
            .await
            .unwrap();

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
        let transfer_amount = 5;

        let mut digital_asset = DigitalAsset::new();
        digital_asset
            .create_and_mint(
                &mut context,
                TokenStandard::Fungible,
                None,
                None,
                mint_amount,
            )
            .await
            .unwrap();

        let destination = Keypair::new();
        let destination_token =
            get_associated_token_address(&destination.pubkey(), &digital_asset.mint.pubkey());
        airdrop(&mut context, &destination.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        digital_asset
            .transfer(
                &mut context,
                payer,
                destination.pubkey(),
                None,
                None,
                None,
                transfer_amount,
            )
            .await
            .unwrap();

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
    async fn transfer_fungible_asset() {
        let mut context = program_test().start_with_context().await;

        let mint_amount = 100;
        let transfer_amount = 99;

        let mut digital_asset = DigitalAsset::new();
        digital_asset
            .create_and_mint(
                &mut context,
                TokenStandard::FungibleAsset,
                None,
                None,
                mint_amount,
            )
            .await
            .unwrap();

        let destination = Keypair::new();
        let destination_token =
            get_associated_token_address(&destination.pubkey(), &digital_asset.mint.pubkey());
        airdrop(&mut context, &destination.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        digital_asset
            .transfer(
                &mut context,
                payer,
                destination.pubkey(),
                None,
                None,
                None,
                transfer_amount,
            )
            .await
            .unwrap();

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
    async fn transfer_programmable_nft() {
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

        let destination = Keypair::new();
        airdrop(&mut context, &destination.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let err = nft
            .transfer(
                &mut context,
                payer,
                destination.pubkey(),
                None,
                Some(rule_set),
                Some(auth_data.clone()),
                transfer_amount,
            )
            .await
            .unwrap_err();

        // Should fail because the recipient is not owned by Token Metadata
        assert_custom_error_ix!(
            1,
            err,
            mpl_token_auth_rules::error::RuleSetError::ProgramOwnedCheckFailed
        );

        // println!("err: {:?}", err);

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

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        nft.transfer(
            &mut context,
            payer,
            escrow_account,
            None,
            Some(rule_set),
            Some(auth_data),
            transfer_amount,
        )
        .await
        .unwrap();

        let recipient_ata = get_associated_token_address(&escrow_account, &nft.mint.pubkey());

        let token_account = spl_token::state::Account::unpack(
            &context
                .banks_client
                .get_account(recipient_ata)
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

        let mut digital_asset = DigitalAsset::new();
        digital_asset
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let delegate = Keypair::new();
        airdrop(&mut context, &delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        digital_asset
            .delegate(
                &mut context,
                authority,
                delegate.pubkey(),
                DelegateRole::Transfer,
                Some(1),
            )
            .await
            .unwrap();

        let destination = Keypair::new();
        let destination_token =
            get_associated_token_address(&destination.pubkey(), &digital_asset.mint.pubkey());
        airdrop(&mut context, &destination.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        digital_asset
            .transfer(
                &mut context,
                delegate,
                destination.pubkey(),
                None,
                None,
                None,
                1,
            )
            .await
            .unwrap();

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

        // Sanity check.
        let fake_delegate = Keypair::new();
        airdrop(&mut context, &fake_delegate.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        // Associated token account already exists so we pass it in,
        // otherwise we will get an "IllegalOwner" errror.

        let err = digital_asset
            .transfer(
                &mut context,
                fake_delegate,
                destination.pubkey(),
                Some(destination_token), // <-- Associated token account
                None,
                None,
                1,
            )
            .await
            .unwrap_err();

        assert_custom_error_ix!(0, err, MetadataError::InvalidOwner);
    }
}
