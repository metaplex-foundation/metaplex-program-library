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

mod delegate {

    use mpl_token_metadata::{
        instruction::{DelegateArgs, DelegateRole},
        pda::find_delegate_account,
        state::{AssetData, TokenStandard, EDITION, PREFIX},
    };

    use super::*;
    #[tokio::test]
    async fn success_delegate_sale() {
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
        let mint_ix = instruction::create(
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
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // delegates the asset for sale

        let user = Keypair::new();
        let user_pubkey = user.pubkey();
        // delegate PDA
        let (delegate, _) = find_delegate_account(
            &mint_pubkey,
            DelegateRole::Sale,
            &user_pubkey,
            &payer_pubkey,
        );

        let delegate_ix = instruction::delegate(
            /* program id            */ id(),
            /* delegate              */ delegate,
            /* user                  */ user_pubkey,
            /* token_owner           */ payer_pubkey,
            /* payer                 */ payer_pubkey,
            /* token                 */ token.pubkey(),
            /* metadata              */ payer_pubkey,
            /* transfer args         */
            DelegateArgs::V1 {
                role: DelegateRole::Sale,
            },
            /* authorization payload */ None,
            /* additional accounts   */ None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
    }
}
