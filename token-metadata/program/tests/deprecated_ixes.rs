#![cfg(feature = "test-bpf")]
pub mod utils;

use num_traits::FromPrimitive;
#[allow(deprecated)]
use old_token_metadata::{error::MetadataError, instruction::create_metadata_accounts_v2, ID};
use solana_program_test::*;

use solana_sdk::{
    instruction::InstructionError,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use utils::*;

#[tokio::test]
async fn deserialize_removed_instruction() {
    let mut context = program_test().start_with_context().await;

    let payer = context.payer.pubkey();

    let test_metadata = Metadata::new();
    let name = "Test".to_string();
    let symbol = "TST".to_string();
    let uri = "uri".to_string();

    create_mint(&mut context, &test_metadata.mint, &payer, Some(&payer), 0)
        .await
        .unwrap();

    create_token_account(
        &mut context,
        &test_metadata.token,
        &test_metadata.mint.pubkey(),
        &payer,
    )
    .await
    .unwrap();

    mint_tokens(
        &mut context,
        &test_metadata.mint.pubkey(),
        &test_metadata.token.pubkey(),
        1,
        &payer,
        None,
    )
    .await
    .unwrap();

    #[allow(deprecated)]
    let tx = Transaction::new_signed_with_payer(
        &[create_metadata_accounts_v2(
            ID,
            test_metadata.pubkey,
            test_metadata.mint.pubkey(),
            payer,
            payer,
            payer,
            name,
            symbol,
            uri,
            None,
            100,
            false,
            true,
            None,
            None,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_custom_error!(err, MetadataError::Removed);
}
