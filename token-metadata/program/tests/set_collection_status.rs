#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    instruction::set_collection_status, state::Metadata as ProgramMetadata, ID as PROGRAM_ID,
};
use solana_program_test::*;
use utils::*;

mod set_collection_status {

    use borsh::BorshDeserialize;
    use mpl_token_metadata::{instruction::CollectionStatus, state::ItemDetails};
    use solana_sdk::{signer::Signer, transaction::Transaction};

    use super::*;

    #[tokio::test]
    async fn successfully_update_status() {
        let mut context = program_test().start_with_context().await;

        // Create a Collection Parent NFT with the ItemDetails struct populated
        let collection_parent_nft = Metadata::new();
        collection_parent_nft
            .create_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
                true, // is collection parent
            )
            .await
            .unwrap();
        let parent_master_edition_account = MasterEditionV2::new(&collection_parent_nft);
        parent_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let current_status = CollectionStatus::None;
        let new_status = CollectionStatus::Announced;

        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_status =
            if let ItemDetails::CollectionInfo { status, size: _ } = metadata.item_details {
                status
            } else {
                panic!("Expected ItemDetails::CollectionInfo");
            };

        assert_eq!(retrieved_status, current_status);

        let ix = set_collection_status(
            PROGRAM_ID,
            collection_parent_nft.pubkey,
            context.payer.pubkey(),
            new_status,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let md_account = context
            .banks_client
            .get_account(collection_parent_nft.pubkey)
            .await
            .unwrap()
            .unwrap();

        let metadata = ProgramMetadata::deserialize(&mut md_account.data.as_slice()).unwrap();
        let retrieved_status =
            if let ItemDetails::CollectionInfo { status, size: _ } = metadata.item_details {
                status
            } else {
                panic!("Expected ItemDetails::CollectionInfo");
            };

        assert_eq!(retrieved_status, new_status);
    }
}
