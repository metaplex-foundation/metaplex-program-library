#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::state::Metadata as ProgramMetadata;
use num_traits::FromPrimitive;
use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use utils::*;

mod escrow {
    use mpl_token_metadata::pda::find_escrow_account;

    use super::*;

    #[tokio::test]
    async fn create_escrow_account_success() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        test_metadata
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        test_master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();

        let escrow_address = find_escrow_account(&test_metadata.mint.pubkey());

        let ix = mpl_token_metadata::instruction::create_escrow_account(
            mpl_token_metadata::id(),
            escrow_address.0,
            test_metadata.pubkey,
            test_metadata.mint.pubkey(),
            test_master_edition.pubkey,
            context.payer.pubkey(),
            solana_program::system_program::id(),
            solana_program::sysvar::rent::id(),
            None,
        );
        println!("{:?} {:?}", &context.payer, &test_metadata.token);

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let metadata = test_metadata.get_data(&mut context).await;
        let escrow_account = get_account(&mut context, &escrow_address.0).await;
        let escrow: mpl_token_metadata::state::TokenOwnedEscrow =
            try_from_slice_unchecked(&escrow_account.data).unwrap();
        print!("{:?}", escrow);
        assert!(escrow.tokens.is_empty());
        assert!(escrow.tokens.is_empty());
        assert!(escrow.model == None);
    }
}
