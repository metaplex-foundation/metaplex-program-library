#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    id, instruction,
    state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
// use num_traits::FromPrimitive;
// use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use utils::*;

mod transfer {

    use mpl_token_metadata::{
        instruction::{create_metadata_accounts_v3, TransferArgs},
        pda::find_metadata_account,
        state::{AssetData, TokenStandard},
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
            context.payer.pubkey(),
            recipient_ata,
            recipient.pubkey(),
            TransferArgs::V1 {
                authorization_payload: None,
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
        // let transfer_amount = 5;

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
            context.payer.pubkey(),
            recipient_ata,
            recipient.pubkey(),
            TransferArgs::V1 {
                authorization_payload: None,
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

        let create_ix = instruction::create_metadata(
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
            context.payer.pubkey(),
            recipient_ata,
            recipient.pubkey(),
            TransferArgs::V1 {
                authorization_payload: None,
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

    // #[tokio::test]
    // async fn success_transfer() {
    //     let mut context = program_test().start_with_context().await;

    //     // asset details

    //     let name = puffed_out_string("Programmable NFT", MAX_NAME_LENGTH);
    //     let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
    //     let uri = puffed_out_string("uri", MAX_URI_LENGTH);

    //     let mut asset = AssetData::new(name.clone(), symbol.clone(), uri.clone());
    //     asset.token_standard = Some(TokenStandard::ProgrammableNonFungible);
    //     asset.seller_fee_basis_points = 500;
    //     /*
    //     asset.programmable_config = Some(ProgrammableConfig {
    //         rule_set: <PUBKEY>,
    //     });
    //     */

    //     // build the mint transaction

    //     let token = Keypair::new();
    //     let mint = Keypair::new();

    //     let mint_pubkey = mint.pubkey();
    //     let program_id = id();
    //     // metadata PDA address
    //     let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
    //     let (metadata, _) = Pubkey::find_program_address(metadata_seeds, &id());
    //     // master edition PDA address
    //     let master_edition_seeds = &[
    //         PREFIX.as_bytes(),
    //         program_id.as_ref(),
    //         mint_pubkey.as_ref(),
    //         EDITION.as_bytes(),
    //     ];
    //     let (master_edition, _) = Pubkey::find_program_address(master_edition_seeds, &id());

    //     let payer_pubkey = context.payer.pubkey();
    //     let mint_ix = instruction::mint(
    //         /* program id       */ id(),
    //         /* token account    */ token.pubkey(),
    //         /* metadata account */ metadata,
    //         /* master edition   */ Some(master_edition),
    //         /* mint account     */ mint.pubkey(),
    //         /* mint authority   */ payer_pubkey,
    //         /* payer            */ payer_pubkey,
    //         /* update authority */ payer_pubkey,
    //         /* asset data       */ asset,
    //         /* initialize mint  */ true,
    //         /* authority signer */ true,
    //     );

    //     let tx = Transaction::new_signed_with_payer(
    //         &[mint_ix],
    //         Some(&context.payer.pubkey()),
    //         &[&context.payer, &mint],
    //         context.last_blockhash,
    //     );

    //     context.banks_client.process_transaction(tx).await.unwrap();

    //     // tries to transfer via spl-token (should fail)

    //     let destination = Keypair::new();
    //     let destination_seeds = &[
    //         PREFIX.as_bytes(),
    //         spl_token::ID.as_ref(),
    //         mint_pubkey.as_ref(),
    //     ];
    //     let (destination_ata, _) = Pubkey::find_program_address(destination_seeds, &id());

    //     let transfer_ix = spl_token::instruction::transfer(
    //         &spl_token::id(),
    //         &token.pubkey(),
    //         &destination_ata,
    //         &payer_pubkey,
    //         &[],
    //         1,
    //     )
    //     .unwrap();
    //     let transfer_tx = Transaction::new_signed_with_payer(
    //         &[transfer_ix],
    //         Some(&payer_pubkey),
    //         &[&context.payer],
    //         context.last_blockhash,
    //     );
    //     let err = context
    //         .banks_client
    //         .process_transaction(transfer_tx)
    //         .await
    //         .unwrap_err();
    //     // it shoudl fail since the account should be frozen
    //     assert_custom_error!(err, spl_token::error::TokenError::AccountFrozen);

    //     // transfer the asset via Token Metadata

    //     let transfer_ix = instruction::transfer(
    //         /* program id            */ id(),
    //         /* token account         */ token.pubkey(),
    //         /* metadata account      */ metadata,
    //         /* mint account          */ mint.pubkey(),
    //         /* destination           */ destination.pubkey(),
    //         /* destination ata       */ destination_ata,
    //         /* owner                 */ payer_pubkey,
    //         /* transfer args         */
    //         TransferArgs::V1 {
    //             authorization_payload: None,
    //         },
    //         /* authorization payload */ None,
    //         /* additional accounts   */ None,
    //     );

    //     let tx = Transaction::new_signed_with_payer(
    //         &[transfer_ix],
    //         Some(&context.payer.pubkey()),
    //         &[&context.payer],
    //         context.last_blockhash,
    //     );

    //     context.banks_client.process_transaction(tx).await.unwrap();
    // }
}
