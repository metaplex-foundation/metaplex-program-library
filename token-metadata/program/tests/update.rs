#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    id, instruction,
    state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use solana_program_test::*;
use solana_sdk::{signature::Signer, transaction::Transaction};
use utils::{DigitalAsset, *};

mod update {

    use mpl_token_metadata::{
        instruction::{AuthorityType, UpdateArgs},
        state::TokenStandard,
    };
    use solana_sdk::signature::Keypair;

    use super::*;
    #[tokio::test]
    async fn success_update() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let new_update_authority = Keypair::new();

        let mut digital_asset = DigitalAsset::new();
        digital_asset
            .create(context, TokenStandard::NonFungible, None)
            .await
            .unwrap();

        let metadata = digital_asset.get_metadata(context).await;
        assert_eq!(
            metadata.data.name,
            puffed_out_string(DEFAULT_NAME, MAX_NAME_LENGTH)
        );
        assert_eq!(
            metadata.data.symbol,
            puffed_out_string(DEFAULT_SYMBOL, MAX_SYMBOL_LENGTH)
        );
        assert_eq!(
            metadata.data.uri,
            puffed_out_string(DEFAULT_URI, MAX_URI_LENGTH)
        );
        assert_eq!(metadata.update_authority, update_authority.pubkey());

        let new_name = puffed_out_string("New Name", MAX_NAME_LENGTH);
        let new_symbol = puffed_out_string("NEW", MAX_SYMBOL_LENGTH);
        let new_uri = puffed_out_string("https://new.digital.asset.org", MAX_URI_LENGTH);

        // Change a few values and update the metadata.
        let mut asset_data = digital_asset.get_asset_data(context).await;
        asset_data.name = new_name.clone();
        asset_data.symbol = new_symbol.clone();
        asset_data.uri = new_uri.clone();
        asset_data.is_mutable = false;
        asset_data.primary_sale_happened = true;
        asset_data.update_authority = new_update_authority.pubkey();

        let authority = AuthorityType::UpdateAuthority(update_authority.pubkey());
        let args = UpdateArgs::V1 {
            authorization_data: None,
            asset_data: Some(asset_data.clone()),
        };

        let update_ix = instruction::update(
            /* program id       */ id(),
            /* metadata account */ digital_asset.metadata,
            /* mint account     */ digital_asset.mint.pubkey(),
            /* master edition   */ digital_asset.master_edition,
            /* authority        */ authority,
            /* auth rules       */ None,
            /* update args      */ args,
            /* additional       */ None,
        );

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&update_authority.pubkey()),
            &[&update_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values
        let metadata = digital_asset.get_metadata(context).await;

        asset_data.update_authority = update_authority.pubkey();

        assert_eq!(metadata.data.name, new_name);
        assert_eq!(metadata.data.symbol, new_symbol);
        assert_eq!(metadata.data.uri, new_uri);

        digital_asset.compare_asset_data(context, &asset_data).await;
    }
}
