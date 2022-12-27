#![cfg(feature = "test-bpf")]
pub mod utils;
use utils::*;

use solana_program_test::*;

mod migrate {
    use super::*;
    use mpl_token_metadata::{
        instruction::MigrateArgs,
        state::{MigrationType, TokenStandard},
    };
    use solana_sdk::{signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn success_migrate() {
        let mut context = &mut program_test().start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Unsized collection
        let (collection_nft, collection_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

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

        let md = nft.get_data(context).await;

        // set up our digital asset struct
        let mut asset = nft.into_digital_asset();
        asset.master_edition = Some(me.pubkey);

        let args = MigrateArgs::V1 {
            migration_type: MigrationType::ProgrammableV1,
        };

        assert_eq!(md.token_standard, Some(TokenStandard::NonFungible));

        asset
            .migrate(&mut context, authority, collection_nft.pubkey, args)
            .await
            .unwrap();

        let new_md = asset.get_metadata(context).await;

        assert_eq!(
            new_md.token_standard,
            Some(TokenStandard::ProgrammableNonFungible)
        );
    }
}
