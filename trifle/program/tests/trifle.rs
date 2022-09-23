#![cfg(feature = "test-bpf")]

pub mod utils;

use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use utils::*;

mod trifle {
    use super::*;

    #[tokio::test]
    async fn create_trifle_account() {
        let mut context = program_test().start_with_context().await;

        // create a transaction that creates two mints and one token account for the payer for each mint.
        // let mint_1_keypair = Keypair::new();
        // let create_mint_1_account_ix = system_instruction::create_account(
        //     &payer.pubkey(),
        //     &mint_1_keypair.pubkey(),
        //     1_000_000_000,
        //     Mint::LEN as u64,
        //     &TOKEN_PROGRAM_ID,
        // );

        // let (metadata_account_address, _) =
        //     mpl_token_metadata::pda::find_metadata_account(&mint_1_keypair.pubkey());

        // let create_metadata_account_ix =
        //     mpl_token_metadata::instruction::create_metadata_accounts_v3(
        //         TOKEN_METADATA_PROGRAM_ID,
        //         metadata_account_address,
        //         mint_1_keypair.pubkey(),
        //         payer.pubkey(),
        //         payer.pubkey(),
        //         payer.pubkey(),
        //         "test".to_string(),
        //         "test".to_string(),
        //         "test".to_string(),
        //         None,
        //         100,
        //         false,
        //         false,
        //         None,
        //         None,
        //         None,
        //     );

        // // create a metadata account
        // // create a master edition account

        // let tx = Transaction::new_signed_with_payer(
        //     &[create_mint_1_account_ix, create_metadata_account_ix],
        //     Some(&payer.pubkey()),
        //     &[&payer, &mint_1_keypair],
        //     recent_blockhash,
        // );

        // banks_client.process_transaction(tx).await.unwrap();

        // create an escrow constraint model
        // create trifle account
    }
}
