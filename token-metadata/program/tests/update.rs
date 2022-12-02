#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    id, instruction,
    state::{Key, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use solana_program_test::*;
use solana_sdk::{signature::Signer, transaction::Transaction};
use utils::Metadata as MetadataManager;
use utils::*;

mod update {

    use mpl_token_metadata::instruction::AuthorityType;
    use mpl_token_metadata::state::{AssetData, Metadata, TokenStandard, PREFIX};
    use solana_program::borsh::try_from_slice_unchecked;

    use super::*;
    #[tokio::test]
    async fn success_update() {
        let mut context = program_test().start_with_context().await;

        // asset details

        /*
        asset.programmable_config = Some(ProgrammableConfig {
            rule_set: <PUBKEY>,
        });
        */

        // mint a default NFT

        let nft = MetadataManager::new();
        nft.create_v3_default(&mut context).await.unwrap();

        // Build the update txn

        let name = puffed_out_string("Programmable NFT", MAX_NAME_LENGTH);
        let symbol = puffed_out_string("PRG", MAX_SYMBOL_LENGTH);
        let uri = puffed_out_string("uri", MAX_URI_LENGTH);

        let mut new_asset = AssetData::new(name.clone(), symbol.clone(), uri.clone());
        new_asset.seller_fee_basis_points = 500;

        let payer_pubkey = context.payer.pubkey();
        let new_update_authority = None;
        let authority = AuthorityType::UpdateAuthority(payer_pubkey);
        let mint_ix = instruction::update(
            /* program id       */ id(),
            /* metadata account */ nft.pubkey,
            /* mint account     */ nft.mint.pubkey(),
            /* master edition   */ None,
            /* mint authority   */ new_update_authority,
            /* payer            */ authority,
            /* update authority */ None,
            /* asset data       */ Some(new_asset),
            /* authority signer */ None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[mint_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values

        let metadata_account = get_account(&mut context, &nft.pubkey).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        assert_eq!(metadata.data.name, name);
        assert_eq!(metadata.data.symbol, symbol);
        assert_eq!(metadata.data.uri, uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 500);
        assert_eq!(metadata.data.creators, None);

        assert!(!metadata.primary_sale_happened);
        assert!(!metadata.is_mutable);
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);

        // assert_eq!(
        //     metadata.token_standard,
        //     Some(TokenStandard::ProgrammableNonFungible)
        // );
        assert_eq!(metadata.collection, None);
        assert_eq!(metadata.uses, None);
    }
}
