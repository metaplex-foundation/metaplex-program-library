#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    id, instruction,
    state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use utils::*;

mod mint {

    use mpl_token_metadata::state::{AssetData, TokenStandard, EDITION, PREFIX};
    use solana_program::program_pack::Pack;
    use spl_token::state::Account;

    use super::*;
    #[tokio::test]
    async fn mint_programmable_nonfungible() {
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

        // build the mint transaction

        let payer_pubkey = context.payer.pubkey();
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

        let create_ix = instruction::create(
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
            &[create_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &mint],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // mints one token

        let (token, _) = Pubkey::find_program_address(
            &[
                &payer_pubkey.to_bytes(),
                &spl_token::id().to_bytes(),
                &mint_pubkey.to_bytes(),
            ],
            &spl_associated_token_account::id(),
        );

        let mint_ix = instruction::mint(
            /* token account       */ token,
            /* metadata account    */ metadata,
            /* mint account        */ mint.pubkey(),
            /* payer               */ payer_pubkey,
            /* authority           */ payer_pubkey,
            /* master edition      */ Some(master_edition),
            /* authorization rules */ None,
            /* amount              */ 1,
        );

        let tx = Transaction::new_signed_with_payer(
            &[mint_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account(&mut context, &token).await;
        let token_account = Account::unpack(&account.data).unwrap();
        assert!(token_account.is_frozen());
        assert_eq!(token_account.amount, 1);
        assert_eq!(token_account.mint, mint.pubkey());
        assert_eq!(token_account.owner, payer_pubkey);
    }
}
