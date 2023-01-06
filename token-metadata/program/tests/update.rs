#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    instruction::{builders::UpdateBuilder, InstructionBuilder},
    state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use solana_program_test::*;
use solana_sdk::{signature::Signer, transaction::Transaction};
use utils::{DigitalAsset, *};

mod update {

    use mpl_token_metadata::{
        instruction::{AuthorityType, RuleSetToggle, UpdateArgs},
        state::{Data, ProgrammableState, TokenStandard, ProgrammableConfig},
    };
    use solana_sdk::signature::Keypair;

    use super::*;
    #[tokio::test]
    async fn success_update() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        da.create(context, TokenStandard::NonFungible, None)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
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
        let data = Data {
            name: new_name.clone(),
            symbol: new_symbol.clone(),
            uri: new_uri.clone(),
            creators: None,
            seller_fee_basis_points: 0,
        };

        let mut update_args = UpdateArgs::default();
        let UpdateArgs::V1 {
            data: current_data, ..
        } = &mut update_args;
        *current_data = Some(data);

        let UpdateArgs::V1 {
            authority_type: current_authority_type,
            ..
        } = &mut update_args;
        *current_authority_type = AuthorityType::Metadata;

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey());

        if let Some(edition) = da.master_edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&update_authority.pubkey()),
            &[&update_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values
        let metadata = da.get_metadata(context).await;

        assert_eq!(metadata.data.name, new_name);
        assert_eq!(metadata.data.symbol, new_symbol);
        assert_eq!(metadata.data.uri, new_uri);
    }

    #[tokio::test]
    async fn update_pfnt_config() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer
        let (rule_set, _auth_data) =
            create_test_ruleset(context, authority, "royalty".to_string()).await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        da.create(
            context,
            TokenStandard::ProgrammableNonFungible,
            Some(rule_set),
        )
        .await
        .unwrap();

        let metadata = da.get_metadata(context).await;

        if let Some(config) = metadata.programmable_config {
            assert_eq!(config.rule_set, Some(rule_set));
        } else {
            panic!("Missing rule set programmable config");
        }

        let mut update_args = UpdateArgs::default();
        let UpdateArgs::V1 { rule_set, .. } = &mut update_args;
        // remove the rule set
        *rule_set = RuleSetToggle::Clear;

        let UpdateArgs::V1 {
            authority_type: current_authority_type,
            ..
        } = &mut update_args;
        *current_authority_type = AuthorityType::Metadata;

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey());

        if let Some(edition) = da.master_edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&update_authority.pubkey()),
            &[&update_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values
        let metadata = da.get_metadata(context).await;

        assert_eq!(
            metadata.programmable_config,
            Some(ProgrammableConfig {
                state: ProgrammableState::Unlocked,
                rule_set: None,
            })
        );
    }
}
