#![cfg(feature = "test-bpf")]
pub mod utils;
use utils::*;

use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{instruction::InstructionError, transaction::TransactionError};

mod migrate {
    use super::*;
    use mpl_token_metadata::{
        error::MetadataError,
        instruction::MigrateArgs,
        state::{MigrationType, TokenStandard},
    };
    use solana_program::pubkey::Pubkey;
    use solana_sdk::{signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn success_migrate() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = program_test.start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create a rule set for the pNFTs.
        let (rule_set, _auth_data) =
            create_test_ruleset(&mut context, authority, "royalty".to_string()).await;

        // Create an unsized collection for the pNFT to belong to, since
        // migration requires the item being a verified member of a collection.
        let (collection_nft, collection_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        // Create the NFT item to migrate.
        let (nft, me) = Metadata::create_default_nft(&mut context).await.unwrap();

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        nft.set_and_verify_collection(
            &mut context,
            collection_nft.pubkey,
            &payer,
            payer.pubkey(),
            collection_nft.mint.pubkey(),
            collection_me.pubkey,
            None,
        )
        .await
        .unwrap();

        let md = nft.get_data(&mut context).await;

        // set up our digital asset struct
        let mut asset = nft.into_digital_asset();
        asset.master_edition = Some(me.pubkey);

        let args = MigrateArgs::V1 {
            migration_type: MigrationType::ProgrammableV1,
            rule_set: Some(rule_set),
        };

        assert_eq!(md.token_standard, Some(TokenStandard::NonFungible));
        assert_eq!(md.programmable_config, None);

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .migrate(&mut context, authority, collection_nft.pubkey, args)
            .await
            .unwrap();

        let new_md = asset.get_metadata(&mut context).await;

        assert_eq!(
            new_md.token_standard,
            Some(TokenStandard::ProgrammableNonFungible)
        );

        if let Some(config) = new_md.programmable_config {
            assert_eq!(config.rule_set, Some(rule_set));
        } else {
            panic!("Missing programmable config");
        }
    }

    #[tokio::test]
    async fn migrate_invalid_collection_metadata() {
        let context = &mut program_test().start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let amount = 1;

        let mut asset = DigitalAsset::new();
        asset
            .create_and_mint(context, TokenStandard::NonFungible, None, None, amount)
            .await
            .unwrap();

        let args = MigrateArgs::V1 {
            migration_type: MigrationType::ProgrammableV1,
            rule_set: None,
        };

        let md = asset.get_metadata(context).await;
        assert_eq!(md.token_standard, Some(TokenStandard::NonFungible));

        let err = asset
            .migrate(context, authority, Pubkey::new_unique(), args)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::DataTypeMismatch);
    }

    #[tokio::test]
    async fn migrate_no_collection() {
        let context = &mut program_test().start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let amount = 1;

        // Unsized collection
        let (collection_nft, _collection_me) =
            Metadata::create_default_nft(context).await.unwrap();

        let mut asset = DigitalAsset::new();
        asset
            .create_and_mint(context, TokenStandard::NonFungible, None, None, amount)
            .await
            .unwrap();

        let args = MigrateArgs::V1 {
            migration_type: MigrationType::ProgrammableV1,
            rule_set: None,
        };

        let md = asset.get_metadata(context).await;
        assert_eq!(md.token_standard, Some(TokenStandard::NonFungible));

        let err = asset
            .migrate(context, authority, collection_nft.pubkey, args)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::NotAMemberOfCollection);
    }
}
