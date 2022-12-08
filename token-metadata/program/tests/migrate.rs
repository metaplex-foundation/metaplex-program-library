#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{id, instruction};
use solana_program_test::*;
use solana_sdk::{signature::Signer, transaction::Transaction};
use utils::{MasterEditionV2 as MasterEditionV2Manager, Metadata as MetadataManager, *};

mod migrate {
    use mpl_token_metadata::state::{Metadata, TokenStandard};
    use solana_program::borsh::try_from_slice_unchecked;
    use solana_sdk::signature::Keypair;

    use super::*;
    #[tokio::test]
    async fn success_migrate() {
        let mut context = &mut program_test().start_with_context().await;

        // asset details

        /*
        asset.programmable_config = Some(ProgrammableConfig {
            rule_set: <PUBKEY>,
        });
        */

        // mint a default NFT and set collection

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let metadata_manager = MetadataManager::new();
        let master_edition_manager = MasterEditionV2Manager::new(&metadata_manager);
        metadata_manager.create_v3_default(context).await.unwrap();
        let collection_nft = MetadataManager::create_default_sized_parent(context)
            .await
            .unwrap();

        master_edition_manager
            .create_v3(context, Some(0))
            .await
            .unwrap();

        metadata_manager
            .set_and_verify_sized_collection_item(
                context,
                collection_nft.0.pubkey,
                &update_authority,
                update_authority.pubkey(),
                collection_nft.0.mint.pubkey(),
                collection_nft.1.pubkey,
                None,
            )
            .await
            .unwrap();

        // migrate ix

        let migrate_ix = instruction::migrate(
            /* program id       */ id(),
            /* metadata account */ metadata_manager.pubkey,
            /* master edition   */ master_edition_manager.pubkey,
            /* mint             */ metadata_manager.mint.pubkey(),
            /* token account    */ metadata_manager.token.pubkey(),
            /* update authority */ update_authority.pubkey(),
            /* collection       */ collection_nft.0.pubkey,
            /* authority signer */ None,
            /* additional       */ None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[migrate_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values

        let metadata_account = get_account(&mut context, &metadata_manager.pubkey).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(
            metadata.token_standard,
            Some(TokenStandard::ProgrammableNonFungible)
        );
    }
}
