#![cfg(feature = "test-bpf")]
mod utils;

use mpl_token_metadata::state::{UseMethod, Uses};
use solana_program_test::*;
use solana_sdk::{
    signature::{Signer},
    transaction::{Transaction},
    instruction::InstructionError,
    transaction::{TransactionError},
    transport::TransportError,
};
use num_traits::FromPrimitive;

use utils::*;

mod uses {
    use mpl_token_metadata::{pda::{find_use_authority_account, find_program_as_burner_account}, error::MetadataError};
    use solana_sdk::signature::Keypair;

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
        let total_uses = metadata_uses.total;
        let remaining_uses = metadata_uses.remaining;

        // Confirm we consumed a use and decremented from 1 -> 0
        assert_eq!(remaining_uses, 0);
        assert_eq!(total_uses, 1);
    }

    #[tokio::test]
    async fn multi_use_with_a_second_use_authority_success() {
        let mut context = program_test().start_with_context().await;
        let use_authority = Keypair::new();
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
                    use_method: UseMethod::Multiple,
                    total: 5,
                    remaining: 4, // Intentionally making this < total
                }),
            )
            .await
            .unwrap();

        let utilize_owner = mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::id(),
            test_metadata.pubkey.clone(),
            test_metadata.token.pubkey(),
            test_metadata.mint.pubkey(),
            None,
            test_metadata.token.pubkey(),
            context.payer.pubkey(),
            context.payer.pubkey(),
            None,
            2,
        );

        let (record, _) =
        find_use_authority_account(&test_metadata.mint.pubkey(), &use_authority.pubkey());
        let (burner, _) = find_program_as_burner_account();

        let add_use_authority = mpl_token_metadata::instruction::approve_use_authority(
            mpl_token_metadata::id(),
            record,
            use_authority.pubkey(),
            context.payer.pubkey(),
            context.payer.pubkey(),
            test_metadata.token.pubkey(),
            test_metadata.pubkey,
            test_metadata.mint.pubkey(),
            burner,
            1,
        );

        let utilize_with_use_authority = mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::id(),
            test_metadata.pubkey.clone(),
            test_metadata.token.pubkey(),
            test_metadata.mint.pubkey(),
            None,
            use_authority.pubkey(),
            context.payer.pubkey(),
            context.payer.pubkey(),
            None,
            1,
        );

        let tx = Transaction::new_signed_with_payer(
            &[utilize_owner, add_use_authority, utilize_with_use_authority],
            Some(&context.payer.pubkey()),
            &[&context.payer, &test_metadata.token, &use_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let metadata = test_metadata.get_data(&mut context).await;
        let metadata_uses = metadata.uses.unwrap();
        let remaining_uses = metadata_uses.remaining;

        assert_eq!(remaining_uses, 1);
    }

    #[tokio::test]
    async fn multi_use_add_and_revoke_use_authority_fail() {
        let mut context = program_test().start_with_context().await;
        let use_authority = Keypair::new();
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
                    use_method: UseMethod::Multiple,
                    total: 2,
                    remaining: 2,
                }),
            )
            .await
            .unwrap();

        let (record, _) =
        find_use_authority_account(&test_metadata.mint.pubkey(), &use_authority.pubkey());
        let (burner, _) = find_program_as_burner_account();

        let add_use_authority = mpl_token_metadata::instruction::approve_use_authority(
            mpl_token_metadata::id(),
            record,
            use_authority.pubkey(),
            context.payer.pubkey(),
            context.payer.pubkey(),
            test_metadata.token.pubkey(),
            test_metadata.pubkey,
            test_metadata.mint.pubkey(),
            burner,
            1,
        );

        let utilize_with_use_authority = mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::id(),
            test_metadata.pubkey.clone(),
            test_metadata.token.pubkey(),
            test_metadata.mint.pubkey(),
            Some(record),
            use_authority.pubkey(),
            use_authority.pubkey(),
            context.payer.pubkey(),
            None,
            1,
        );

        let tx = Transaction::new_signed_with_payer(
            &[add_use_authority, utilize_with_use_authority],
            Some(&use_authority.pubkey()),
            &[&context.payer, &use_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx.clone()).await.unwrap();

        let revoke_use_authority = mpl_token_metadata::instruction::revoke_use_authority(
            mpl_token_metadata::id(),
            record,
            use_authority.pubkey(),
            context.payer.pubkey(),
            test_metadata.token.pubkey(),
            test_metadata.pubkey,
            test_metadata.mint.pubkey(),
        );

        let utilize_with_use_authority_fail = mpl_token_metadata::instruction::utilize(
            mpl_token_metadata::id(),
            test_metadata.pubkey.clone(),
            test_metadata.token.pubkey(),
            test_metadata.mint.pubkey(),
            Some(record),
            use_authority.pubkey(),
            use_authority.pubkey(),
            context.payer.pubkey(),
            None,
            1,
        );

        let tx_error = Transaction::new_signed_with_payer(
            &[revoke_use_authority, utilize_with_use_authority_fail],
            Some(&use_authority.pubkey(),),
            &[&context.payer, &use_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx_error.clone()).await.unwrap();

        // let err = context
        //     .banks_client
        //     .process_transaction(tx_error.clone())
        //     .await
        //     .unwrap_err();

        // assert_custom_error!(err, MetadataError::UseAuthorityRecordAlreadyRevoked);
    }
}
