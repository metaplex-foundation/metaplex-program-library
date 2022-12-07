#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    id, instruction,
    state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
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

mod transfer {

    use mpl_token_metadata::{
        instruction::TransferArgs,
        state::{AssetData, TokenStandard, EDITION, PREFIX},
    };

    use super::*;
    #[tokio::test]
    async fn success_transfer() {
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
            rule_set: <PUBKEY>,
        });
        */

        // build the mint transaction

        let token = Keypair::new();
        let mint = Keypair::new();

        let mint_pubkey = mint.pubkey();
        let program_id = id();
        // metadata PDA address
        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (metadata, _) = Pubkey::find_program_address(metadata_seeds, &id());
        // master edition PDA address
        let master_edition_seeds = &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_pubkey.as_ref(),
            EDITION.as_bytes(),
        ];
        let (master_edition, _) = Pubkey::find_program_address(master_edition_seeds, &id());

        let payer_pubkey = context.payer.pubkey();
        let mint_ix = instruction::create_metadata(
            /* metadata account */ metadata,
            /* master edition   */ Some(master_edition),
            /* mint account     */ mint.pubkey(),
            /* mint authority   */ payer_pubkey,
            /* payer            */ payer_pubkey,
            /* update authority */ payer_pubkey,
            /* initialize mint  */ true,
            /* authority signer */ true,
            /* asset data       */ asset,
            /* decimals         */ Some(0),
            /* max supply       */ Some(0),
        );

        let tx = Transaction::new_signed_with_payer(
            &[mint_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &mint],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // tries to transfer via spl-token (should fail)

        let destination = Keypair::new();
        let destination_seeds = &[
            PREFIX.as_bytes(),
            spl_token::ID.as_ref(),
            mint_pubkey.as_ref(),
        ];
        let (destination_ata, _) = Pubkey::find_program_address(destination_seeds, &id());

        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &token.pubkey(),
            &destination_ata,
            &payer_pubkey,
            &[],
            1,
        )
        .unwrap();
        let transfer_tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&payer_pubkey),
            &[&context.payer],
            context.last_blockhash,
        );
        let err = context
            .banks_client
            .process_transaction(transfer_tx)
            .await
            .unwrap_err();
        // it shoudl fail since the account should be frozen
        assert_custom_error!(err, spl_token::error::TokenError::AccountFrozen);

        // transfer the asset via Token Metadata

        let transfer_ix = instruction::transfer(
            /* program id            */ id(),
            /* token account         */ token.pubkey(),
            /* metadata account      */ metadata,
            /* mint account          */ mint.pubkey(),
            /* destination           */ destination.pubkey(),
            /* destination ata       */ destination_ata,
            /* owner                 */ payer_pubkey,
            /* transfer args         */
            TransferArgs::V1 {
                authorization_payload: None,
            },
            /* authorization payload */ None,
            /* additional accounts   */ None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
    }
}
