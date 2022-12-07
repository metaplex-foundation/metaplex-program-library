#![cfg(feature = "test-bpf")]
pub mod utils;

use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::TransactionError,
};
use utils::*;

// We need to check that all four migration cases work as expected:
// * Sized -> Sized
// * Sized -> Unsized
// * Unsized -> Sized
// * Unsized -> Unsized
mod collection_migrations {
    use mpl_token_metadata::error::MetadataError;

    use super::*;

    #[tokio::test]
    async fn sized_to_sized() {
        let mut context = program_test().start_with_context().await;

        // Sized collection, size = 0
        let (collection_a_nft, collection_a_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Sized collection, size = 0
        let (collection_b_nft, collection_b_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_two, _me_item_two) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_three, _me_item_three) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the NFTs to Collection A
        nft_item_one
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Check that size is correct.
        assert_collection_size(&mut context, &collection_a_nft, 3).await;

        // Try to move a NFT to Collection B without unverifying.
        let err = nft_item_one
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MustUnverify);

        // Unverify the NFT and move it to Collection B, then check size.
        nft_item_one
            .unverify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // We need several slots between unverifying and running set_and_verify_collection.
        context.warp_to_slot(2).unwrap();

        nft_item_one
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        assert_collection_size(&mut context, &collection_a_nft, 2).await;
        assert_collection_size(&mut context, &collection_b_nft, 1).await;

        // Unverify the other two and migrate them over.
        nft_item_two
            .unverify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .unverify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        assert_collection_size(&mut context, &collection_a_nft, 0).await;
        assert_collection_size(&mut context, &collection_b_nft, 3).await;
    }

    #[tokio::test]
    async fn sized_to_unsized() {
        let mut context = program_test().start_with_context().await;

        // Sized collection, size = 0
        let (collection_a_nft, collection_a_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Unsized collection
        let (collection_b_nft, collection_b_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_two, _me_item_two) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_three, _me_item_three) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the NFTs to Collection A
        nft_item_one
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Check that size is correct.
        assert_collection_size(&mut context, &collection_a_nft, 3).await;

        // Try to move a NFT to Collection B without unverifying.
        let err = nft_item_one
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MustUnverify);

        // Unverify the NFT and move it to Collection B, then check size.
        nft_item_one
            .unverify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // We need several slots between unverifying and running set_and_verify_collection.
        context.warp_to_slot(2).unwrap();

        nft_item_one
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Collection A should have its size decremented, but Collection b is unsized.
        assert_collection_size(&mut context, &collection_a_nft, 2).await;

        // Unverify the other two and migrate them over.
        nft_item_two
            .unverify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .unverify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Colleciton A should be empty now.
        assert_collection_size(&mut context, &collection_a_nft, 0).await;
    }

    #[tokio::test]
    async fn unsized_to_sized() {
        let mut context = program_test().start_with_context().await;

        // Unsized collection
        let (collection_a_nft, collection_a_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        // Sized collection, size = 0
        let (collection_b_nft, collection_b_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_two, _me_item_two) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_three, _me_item_three) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the NFTs to Collection A
        nft_item_one
            .set_and_verify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .set_and_verify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .set_and_verify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Try to move a NFT to Collection B without unverifying.
        let err = nft_item_one
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MustUnverify);

        // Unverify the NFT and move it to Collection B, then check size.
        nft_item_one
            .unverify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // We need several slots between unverifying and running set_and_verify_collection.
        context.warp_to_slot(2).unwrap();

        nft_item_one
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Collection B should have its size incremented, but Collection A is unsized.
        assert_collection_size(&mut context, &collection_b_nft, 1).await;

        // Unverify the other two and migrate them over.
        nft_item_two
            .unverify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .unverify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Colleciton B should have all three NFTs now.
        assert_collection_size(&mut context, &collection_b_nft, 3).await;
    }

    #[tokio::test]
    async fn unsized_to_unsized() {
        let mut context = program_test().start_with_context().await;

        // Unsized collection
        let (collection_a_nft, collection_a_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        // Unsized collection
        let (collection_b_nft, collection_b_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_two, _me_item_two) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_three, _me_item_three) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the NFTs to Collection A
        nft_item_one
            .set_and_verify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .set_and_verify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .set_and_verify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Try to move a NFT to Collection B without unverifying.
        // We still need to unverify, because this handler cannot know if the current collection
        // is sized or not as it does not receive the account for collection A.
        let err = nft_item_one
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::MustUnverify);

        // Unverify the NFTs and move them to Collection B.
        nft_item_one
            .unverify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .unverify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .unverify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // We need several slots between unverifying and running set_and_verify_collection.
        context.warp_to_slot(2).unwrap();

        nft_item_one
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_two
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        nft_item_three
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Both collections are unsized so no size checks.
    }

    #[tokio::test]
    pub async fn migrate_to_self() {
        let mut context = program_test().start_with_context().await;

        // Sized collection, size = 0
        let (collection_a_nft, collection_a_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Unsized collection
        let (collection_b_nft, collection_b_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_two, _me_item_two) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the first NFT to Collection A
        nft_item_one
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Check size
        assert_collection_size(&mut context, &collection_a_nft, 1).await;

        // Move the collection to itself. This should succeed even though it's already verified because it's not
        // actually migrating.
        nft_item_one
            .set_and_verify_sized_collection_item(
                &mut context,
                collection_a_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Add the second NFT to Collection B
        nft_item_two
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();

        // Move the collection to itself. This should succeed even though it's already verified because it's not
        // actually migrating.
        nft_item_two
            .set_and_verify_collection(
                &mut context,
                collection_b_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_b_nft.mint.pubkey(),
                collection_b_me.pubkey,
                None,
            )
            .await
            .unwrap();
    }
}
