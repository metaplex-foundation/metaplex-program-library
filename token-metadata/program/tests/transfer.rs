#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_metadata::{
    id, instruction,
    state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use utils::*;

mod transfer {

    use mpl_token_metadata::{
        instruction::{
            create_escrow_account, create_metadata_accounts_v3, mint, MintArgs, TransferArgs,
        },
        pda::{find_master_edition_account, find_metadata_account},
        processor::find_escrow_account,
        state::{AssetData, EscrowAuthority, ProgrammableConfig, TokenStandard},
    };
    use solana_program::{
        native_token::LAMPORTS_PER_SOL, program_pack::Pack, system_instruction::create_account,
    };
    use spl_associated_token_account::{
        get_associated_token_address, instruction::create_associated_token_account,
    };
    use spl_token::instruction::{initialize_mint, mint_to};

    use super::*;

    #[tokio::test]
    async fn transfer_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // Create a NonFungible token using the old handlers.
        let nft = Metadata::new();
        nft.create_v3_default(&mut context).await.unwrap();

        let master_edition = MasterEditionV2::new(&nft);
        master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let recipient = Keypair::new();
        airdrop(&mut context, &recipient.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let recipient_ata = get_associated_token_address(&recipient.pubkey(), &nft.mint.pubkey());

        let ata_ix = create_associated_token_account(
            &context.payer.pubkey(),
            &recipient.pubkey(),
            &nft.mint.pubkey(),
            &spl_token::id(),
        );

        let transfer_ix = instruction::transfer(
            id(),
            nft.token.pubkey(),
            nft.pubkey,
            nft.mint.pubkey(),
            None,
            context.payer.pubkey(),
            recipient_ata,
            recipient.pubkey(),
            TransferArgs::V1 {
                authorization_data: None,
                amount: 1,
            },
            None,
            None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ata_ix, transfer_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

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
    async fn transfer_fungible() {
        // Transfer a fungible asset with a metadata account,
        // created outside of the Token Metadata program.
        let mut context = program_test().start_with_context().await;

        let mint = Keypair::new();
        let ata = get_associated_token_address(&context.payer.pubkey(), &mint.pubkey());
        let (metadata, _) = find_metadata_account(&mint.pubkey());

        let mint_layout: u64 = 82;
        let token_amount = 10;
        let transfer_amount = 5;

        let payer = context.payer.pubkey();

        let min_rent = context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(mint_layout as usize);

        // Create mint account
        let create_mint_account_ix = create_account(
            &payer,
            &mint.pubkey(),
            min_rent,
            mint_layout,
            &spl_token::ID,
        );

        // Initalize mint ix
        let init_mint_ix =
            initialize_mint(&spl_token::ID, &mint.pubkey(), &payer, Some(&payer), 0).unwrap();

        let create_assoc_account_ix =
            create_associated_token_account(&payer, &payer, &mint.pubkey(), &spl_token::ID);

        let mint_to_ix = mint_to(
            &spl_token::ID,
            &mint.pubkey(),
            &ata,
            &payer,
            &[],
            token_amount,
        )
        .unwrap();

        let create_metadata_account_ix = create_metadata_accounts_v3(
            id(),
            metadata,
            mint.pubkey(),
            payer,
            payer,
            payer,
            "name".to_string(),
            "symbol".to_string(),
            "uri".to_string(),
            None,
            0,
            true,
            true,
            None,
            None,
            None,
        );

        let instructions = vec![
            create_mint_account_ix,
            init_mint_ix,
            create_assoc_account_ix,
            mint_to_ix,
            create_metadata_account_ix,
        ];

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&context.payer.pubkey()),
            &[&context.payer, &mint],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let recipient = Keypair::new();
        airdrop(&mut context, &recipient.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let recipient_ata = get_associated_token_address(&recipient.pubkey(), &mint.pubkey());

        let ata_ix = create_associated_token_account(
            &context.payer.pubkey(),
            &recipient.pubkey(),
            &mint.pubkey(),
            &spl_token::id(),
        );

        let transfer_ix = instruction::transfer(
            id(),
            ata,
            metadata,
            mint.pubkey(),
            None,
            context.payer.pubkey(),
            recipient_ata,
            recipient.pubkey(),
            TransferArgs::V1 {
                authorization_data: None,
                amount: transfer_amount,
            },
            None,
            None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ata_ix, transfer_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

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

        assert_eq!(token_account.amount, transfer_amount);
    }

    #[tokio::test]
    async fn transfer_fungible_asset() {
        let mut context = program_test().start_with_context().await;

        // Create a Fungible token using the the new generic `mint` handler.
        let name = puffed_out_string("Fungible", MAX_NAME_LENGTH);
        let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
        let uri = puffed_out_string("uri", MAX_URI_LENGTH);

        let mut asset = AssetData::new(
            TokenStandard::FungibleAsset,
            name.clone(),
            symbol.clone(),
            uri.clone(),
        );
        asset.seller_fee_basis_points = 500;

        let mint = Keypair::new();
        let ata = get_associated_token_address(&context.payer.pubkey(), &mint.pubkey());
        let (metadata, _) = find_metadata_account(&mint.pubkey());

        let payer = context.payer.pubkey();

        let token_amount = 10;
        let transfer_amount = 5;

        let create_ix = instruction::create(
            metadata,
            None,
            mint.pubkey(),
            payer,
            payer,
            payer,
            true,
            true,
            asset,
            Some(0),
            Some(1000),
        );

        let create_assoc_account_ix =
            create_associated_token_account(&payer, &payer, &mint.pubkey(), &spl_token::ID);

        let mint_ix = mint_to(
            &spl_token::ID,
            &mint.pubkey(),
            &ata,
            &payer,
            &[],
            token_amount,
        )
        .unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, create_assoc_account_ix, mint_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &mint],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let recipient = Keypair::new();
        airdrop(&mut context, &recipient.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let recipient_ata = get_associated_token_address(&recipient.pubkey(), &mint.pubkey());

        let ata_ix = create_associated_token_account(
            &context.payer.pubkey(),
            &recipient.pubkey(),
            &mint.pubkey(),
            &spl_token::id(),
        );

        let transfer_ix = instruction::transfer(
            id(),
            ata,
            metadata,
            mint.pubkey(),
            None,
            context.payer.pubkey(),
            recipient_ata,
            recipient.pubkey(),
            TransferArgs::V1 {
                authorization_data: None,
                amount: transfer_amount,
            },
            None,
            None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ata_ix, transfer_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

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

        assert_eq!(token_account.amount, transfer_amount);
    }

    #[tokio::test]
    async fn transfer_programmable_nft() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        // Create NFT for owning the TOE account.
        // Create a NonFungible token using the old handlers.
        let nft = Metadata::new();
        nft.create_v3_default(&mut context).await.unwrap();

        let nft_master_edition = MasterEditionV2::new(&nft);
        nft_master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        // Create NFT for transfer tests.
        let name = puffed_out_string("Fungible", MAX_NAME_LENGTH);
        let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
        let uri = puffed_out_string("uri", MAX_URI_LENGTH);

        let mut asset = AssetData::new(
            TokenStandard::ProgrammableNonFungible,
            name.clone(),
            symbol.clone(),
            uri.clone(),
        );
        asset.seller_fee_basis_points = 500;

        // Create rule-set for the transfer
        let (rule_set, auth_data) = create_royalty_ruleset(&mut context).await;
        asset.programmable_config = Some(ProgrammableConfig { rule_set });

        let mint_key = Keypair::new();
        let ata = get_associated_token_address(&context.payer.pubkey(), &mint_key.pubkey());
        let (metadata, _) = find_metadata_account(&mint_key.pubkey());
        let (master_edition, _) = find_master_edition_account(&mint_key.pubkey());

        let payer = context.payer.pubkey();

        let token_amount = 1;

        let create_ix = instruction::create(
            metadata,
            Some(master_edition),
            mint_key.pubkey(),
            payer,
            payer,
            payer,
            true,
            true,
            asset.clone(),
            Some(0),
            Some(0),
        );

        let mint_ix = mint(
            ata,
            metadata,
            mint_key.pubkey(),
            payer,
            payer,
            Some(master_edition),
            None,
            MintArgs::V1 {
                amount: token_amount,
            },
        );

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, mint_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &mint_key],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let recipient = Keypair::new();
        airdrop(&mut context, &recipient.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let recipient_ata = get_associated_token_address(&recipient.pubkey(), &mint_key.pubkey());

        let ata_ix = create_associated_token_account(
            &context.payer.pubkey(),
            &recipient.pubkey(),
            &mint_key.pubkey(),
            &spl_token::id(),
        );

        let transfer_ix = instruction::transfer(
            id(),
            ata,
            metadata,
            mint_key.pubkey(),
            Some(master_edition),
            context.payer.pubkey(),
            recipient_ata,
            recipient.pubkey(),
            TransferArgs::V1 {
                authorization_data: Some(auth_data.clone()),
                amount: 1,
            },
            Some(rule_set),
            None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ata_ix, transfer_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // Should fail because the recipient is not owned by Token Metadata
        assert_custom_error_ix!(
            1,
            err,
            mpl_token_auth_rules::error::RuleSetError::ProgramOwnedCheckFailed
        );

        // Create TOE account and try to transfer to it. This should succeed.
        let (escrow_account, _) =
            find_escrow_account(&nft.mint.pubkey(), &EscrowAuthority::TokenOwner);

        let create_escrow_ix = create_escrow_account(
            id(),
            escrow_account,
            nft.pubkey,
            nft.mint.pubkey(),
            nft.token.pubkey(),
            nft_master_edition.pubkey,
            payer,
            Some(payer),
        );

        let recipient_ata = get_associated_token_address(&escrow_account, &mint_key.pubkey());

        let ata_ix = create_associated_token_account(
            &context.payer.pubkey(),
            &escrow_account,
            &mint_key.pubkey(),
            &spl_token::id(),
        );

        let transfer_ix = instruction::transfer(
            id(),
            ata,
            metadata,
            mint_key.pubkey(),
            Some(master_edition),
            context.payer.pubkey(),
            recipient_ata,
            escrow_account,
            TransferArgs::V1 {
                authorization_data: Some(auth_data),
                amount: 1,
            },
            Some(rule_set),
            None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[create_escrow_ix, ata_ix, transfer_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

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
}
