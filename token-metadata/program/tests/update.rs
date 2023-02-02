#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    instruction::{builders::UpdateBuilder, InstructionBuilder},
    state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::Signer,
    transaction::{Transaction, TransactionError},
};
use utils::{DigitalAsset, *};

mod update {

    use mpl_token_metadata::{
        error::MetadataError,
        instruction::{DelegateArgs, RuleSetToggle, UpdateArgs},
        state::{Data, ProgrammableConfig, TokenStandard},
    };
    use solana_program::pubkey::Pubkey;
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

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .payer(update_authority.pubkey());

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
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority).await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            context,
            TokenStandard::ProgrammableNonFungible,
            Some(authorization_rules),
            Some(auth_data),
            1,
        )
        .await
        .unwrap();

        let metadata = da.get_metadata(context).await;

        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(rule_set),
        }) = metadata.programmable_config
        {
            assert_eq!(rule_set, authorization_rules);
        } else {
            panic!("Missing rule set programmable config");
        }

        let mut update_args = UpdateArgs::default();
        let UpdateArgs::V1 { rule_set, .. } = &mut update_args;
        // remove the rule set
        *rule_set = RuleSetToggle::Clear;

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(update_authority.pubkey());

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

        assert_eq!(metadata.programmable_config, None);
    }

    #[tokio::test]
    async fn update_invalid_rule_set() {
        // Currently users can add an invalid rule set to their pNFT which will effectively
        // prevent it from being updated again because it either won't be owned by the mpl-token-auth rules
        // program or it won't be a valid rule set to call validate on.
        // We relax the check a little to let users fix invalid rule sets.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        let invalid_rule_set = Pubkey::new_unique();

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let (authorization_rules, _auth_data) =
            create_default_metaplex_rule_set(context, authority).await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create with an empty rule set so we can test updating.
        let mut da = DigitalAsset::new();
        da.create_and_mint(
            context,
            TokenStandard::ProgrammableNonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let metadata = da.get_metadata(context).await;

        if let Some(ProgrammableConfig::V1 { rule_set }) = metadata.programmable_config {
            assert_eq!(rule_set, None);
        } else {
            panic!("Missing rule set programmable config");
        }

        let mut update_args = UpdateArgs::default();
        let UpdateArgs::V1 { rule_set, .. } = &mut update_args;
        *rule_set = RuleSetToggle::Set(invalid_rule_set);

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .payer(update_authority.pubkey());

        if let Some(edition) = da.master_edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args.clone()).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&update_authority.pubkey()),
            &[&update_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let metadata = da.get_metadata(context).await;

        // Should be successfully updated with the invalid rule set.
        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(rule_set),
        }) = metadata.programmable_config
        {
            assert_eq!(rule_set, invalid_rule_set);
        } else {
            panic!("Missing rule set programmable config");
        }

        // Now we pass in a valid authorization rules account owned by mpl-token-auth-rules
        // but which does not match the pubkey we are passing in to set as the rule set value.
        // This will fail with an "InvalidAuthorizationRules" error.
        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .authorization_rules(authorization_rules)
            .payer(update_authority.pubkey());

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

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorizationRules);

        // Finally, try to update with the valid rule set, and it should succeed.
        let mut update_args = UpdateArgs::default();
        let UpdateArgs::V1 { rule_set, .. } = &mut update_args;
        *rule_set = RuleSetToggle::Set(authorization_rules);

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(invalid_rule_set)
            .payer(update_authority.pubkey());

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

        let metadata = da.get_metadata(context).await;

        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(rule_set),
        }) = metadata.programmable_config
        {
            assert_eq!(rule_set, authorization_rules);
        } else {
            panic!("Missing rule set programmable config");
        }
    }

    #[tokio::test]
    async fn cannot_update_rule_set_when_delegate_set() {
        // When a delegate is set, the rule set cannot be updated.
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let mut context = &mut program_test.start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority.dirty_clone()).await;

        let (new_auth_rules, new_auth_data) =
            create_default_metaplex_rule_set(context, authority.dirty_clone()).await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        da.create_and_mint(
            context,
            TokenStandard::ProgrammableNonFungible,
            Some(authorization_rules),
            Some(auth_data),
            1,
        )
        .await
        .unwrap();

        let metadata = da.get_metadata(context).await;

        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(rule_set),
        }) = metadata.programmable_config
        {
            assert_eq!(rule_set, authorization_rules);
        } else {
            panic!("Missing rule set programmable config");
        }

        let delegate = Keypair::new();

        // Set a delegate
        da.delegate(
            &mut context,
            update_authority.dirty_clone(),
            delegate.pubkey(),
            DelegateArgs::TransferV1 {
                amount: 1,
                authorization_data: None,
            },
        )
        .await
        .unwrap();

        // Try to clear the rule set.
        let mut update_args = UpdateArgs::default();
        let UpdateArgs::V1 { rule_set, .. } = &mut update_args;
        // remove the rule set
        *rule_set = RuleSetToggle::Clear;

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(update_authority.pubkey());

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

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CannotUpdateAssetWithDelegate);

        // Try to update the rule set.
        let mut update_args = UpdateArgs::default();
        let UpdateArgs::V1 {
            rule_set,
            authorization_data,
            ..
        } = &mut update_args;
        // update the rule set
        *rule_set = RuleSetToggle::Set(new_auth_rules);
        *authorization_data = Some(new_auth_data);

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(new_auth_rules)
            .payer(update_authority.pubkey());

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

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CannotUpdateAssetWithDelegate);
    }
}
