#![cfg(feature = "test-bpf")]
mod utils;

use mpl_token_metadata::state::{UseMethod, Uses};
use solana_program_test::*;
use solana_sdk::{
    signature::{Signer},
    transaction::{Transaction},
};
use utils::*;

mod uses {
    use super::*;
    #[tokio::test]
    async fn single_use_success() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        test_metadata
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(Uses {
                    use_method: UseMethod::Single,
                    total: 1,
                    remaining: 1,
                }),
            )
            .await
            .unwrap();

        let ix = mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::id(),
            test_metadata.pubkey.clone(),
            test_metadata.token.pubkey(),
            test_metadata.mint.pubkey(),
            None,
            test_metadata.token.pubkey(),
            context.payer.pubkey(),
            context.payer.pubkey(),
            None,
            1,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &test_metadata.token],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
    
        let metadata = test_metadata.get_data(&mut context).await;
        let metadata_uses = metadata.uses.unwrap();
        let remaining_uses = metadata_uses.remaining;

        assert_eq!(remaining_uses, 0);
    }
}
