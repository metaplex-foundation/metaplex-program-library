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
    use mpl_token_metadata::{
        error::MetadataError,
        pda::find_collection_authority_account,
        state::{Metadata as TmMetadata, TokenMetadataAccount},
    };
    use solana_sdk::transaction::Transaction;

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

    #[tokio::test]
    pub async fn burned_collection_parent() {
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

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the items to Collection A
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

        // Burn the collection A parent.
        burn(
            &mut context,
            collection_a_nft.pubkey,
            &kp,
            collection_a_nft.mint.pubkey(),
            collection_a_nft.token.pubkey(),
            collection_a_me.pubkey,
            None,
        )
        .await
        .unwrap();

        // This should succeed.
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

        // Now we can migrate it over to collection b.
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

        let account = get_account(&mut context, &nft_item_one.pubkey).await;
        let md = TmMetadata::safe_deserialize(&account.data).unwrap();

        let collection = md.collection.unwrap();

        assert_eq!(collection.key, collection_b_nft.mint.pubkey());
        assert!(collection.verified);
    }

    #[tokio::test]
    pub async fn burned_collection_parent_sized_unverify() {
        let mut context = program_test().start_with_context().await;

        // Unsized collection
        let (collection_a_nft, collection_a_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Unsized collection
        let (collection_b_nft, collection_b_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_two, _me_item_two) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the items to Collection A
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

        // Burn the collection A parent.
        burn(
            &mut context,
            collection_a_nft.pubkey,
            &kp,
            collection_a_nft.mint.pubkey(),
            collection_a_nft.token.pubkey(),
            collection_a_me.pubkey,
            None,
        )
        .await
        .unwrap();

        // This should succeed.
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

        // Now we can migrate it over to collection b.
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

        let account = get_account(&mut context, &nft_item_one.pubkey).await;
        let md = TmMetadata::safe_deserialize(&account.data).unwrap();

        let collection = md.collection.unwrap();

        assert_eq!(collection.key, collection_b_nft.mint.pubkey());
        assert!(collection.verified);
    }

    #[tokio::test]
    pub async fn burned_collection_parent_sized_collection() {
        let mut context = program_test().start_with_context().await;

        // Sized collection
        let (collection_a_nft, collection_a_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        // Sized collection
        let (collection_b_nft, collection_b_me) =
            Metadata::create_default_sized_parent(&mut context)
                .await
                .unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_two, _me_item_two) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the items to Collection A
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

        // Burn the collection A parent.
        burn(
            &mut context,
            collection_a_nft.pubkey,
            &kp,
            collection_a_nft.mint.pubkey(),
            collection_a_nft.token.pubkey(),
            collection_a_me.pubkey,
            None,
        )
        .await
        .unwrap();

        // This should succeed.
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

        // Now we can migrate it over to collection b.
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

        let account = get_account(&mut context, &nft_item_one.pubkey).await;
        let md = TmMetadata::safe_deserialize(&account.data).unwrap();

        let collection = md.collection.unwrap();

        assert_eq!(collection.key, collection_b_nft.mint.pubkey());
        assert!(collection.verified);
    }

    #[tokio::test]
    pub async fn burned_collection_parent_wrong_authority_fails() {
        let mut context = program_test().start_with_context().await;

        let incorrect_authority = Keypair::new();

        // Unsized collection
        let (collection_a_nft, collection_a_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();
        let (nft_item_two, _me_item_two) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the items to Collection A
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

        // Burn the collection A parent.
        burn(
            &mut context,
            collection_a_nft.pubkey,
            &kp,
            collection_a_nft.mint.pubkey(),
            collection_a_nft.token.pubkey(),
            collection_a_me.pubkey,
            None,
        )
        .await
        .unwrap();

        let err = nft_item_one
            .unverify_collection(
                &mut context,
                collection_a_nft.pubkey,
                &incorrect_authority,
                collection_a_nft.mint.pubkey(),
                collection_a_me.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);
    }

    #[tokio::test]
    pub async fn burned_collection_parent_delegate_fails() {
        let mut context = program_test().start_with_context().await;

        // Unsized collection
        let (collection_nft, collection_me) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let (nft_item_one, _me_item_one) =
            Metadata::create_default_nft(&mut context).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Add the item to Collection
        nft_item_one
            .set_and_verify_collection(
                &mut context,
                collection_nft.pubkey,
                &kp,
                kp.pubkey(),
                collection_nft.mint.pubkey(),
                collection_me.pubkey,
                None,
            )
            .await
            .unwrap();

        let delegate_keypair = Keypair::new();
        let update_authority = context.payer.pubkey();

        let (record, _) = find_collection_authority_account(
            &collection_nft.mint.pubkey(),
            &delegate_keypair.pubkey(),
        );

        let ix1 = mpl_token_metadata::instruction::approve_collection_authority(
            mpl_token_metadata::ID,
            record,
            delegate_keypair.pubkey(),
            update_authority,
            context.payer.pubkey(),
            collection_nft.pubkey,
            collection_nft.mint.pubkey(),
        );

        let tx1 = Transaction::new_signed_with_payer(
            &[ix1],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx1).await.unwrap();

        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();

        // Burn the collection A parent.
        burn(
            &mut context,
            collection_nft.pubkey,
            &kp,
            collection_nft.mint.pubkey(),
            collection_nft.token.pubkey(),
            collection_me.pubkey,
            None,
        )
        .await
        .unwrap();

        // Collection delegate is valid but this should fail because
        // the collection parent is burned.
        let err = nft_item_one
            .unverify_collection(
                &mut context,
                collection_nft.pubkey,
                &delegate_keypair,
                collection_nft.mint.pubkey(),
                collection_me.pubkey,
                None,
            )
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::UpdateAuthorityIncorrect);
    }
}
