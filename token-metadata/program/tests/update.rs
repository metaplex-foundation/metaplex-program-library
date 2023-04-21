#![cfg(feature = "test-bpf")]
pub mod utils;

use mpl_token_metadata::{
    error::MetadataError,
    instruction::{
        builders::UpdateBuilder, CollectionToggle, DelegateArgs, InstructionBuilder, RuleSetToggle,
        UpdateArgs,
    },
    state::{Collection, Creator, Data, ProgrammableConfig, TokenStandard},
    state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use num_traits::FromPrimitive;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::Keypair,
    signature::Signer,
    transaction::{Transaction, TransactionError},
};
use utils::{DigitalAsset, *};

mod update {
    use super::*;

    #[tokio::test]
    async fn success_update_by_update_authority() {
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
            creators: metadata.data.creators, // keep the same creators
            seller_fee_basis_points: 0,
        };

        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 {
                data: current_data, ..
            } => *current_data = Some(data),
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .payer(update_authority.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

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
    async fn success_update_by_authority_item_delegate() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        da.create(context, TokenStandard::NonFungible, None)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.update_authority, update_authority.pubkey());
        assert!(!metadata.primary_sale_happened);
        assert!(metadata.is_mutable);

        // Create metadata delegate.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_record = da
            .delegate(
                context,
                update_authority,
                delegate.pubkey(),
                DelegateArgs::AuthorityItemV1 {
                    authorization_data: None,
                },
            )
            .await
            .unwrap()
            .unwrap();

        // Change a few values that this delegate is allowed to change.
        let mut args = UpdateArgs::default_authority_item_delegate();

        match &mut args {
            UpdateArgs::AuthorityItemDelegateV2 {
                new_update_authority,
                primary_sale_happened,
                is_mutable,
                ..
            } => {
                *new_update_authority = Some(delegate.pubkey());
                *primary_sale_happened = Some(true);
                *is_mutable = Some(false);
            }
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values
        let metadata = da.get_metadata(context).await;

        assert_eq!(metadata.update_authority, delegate.pubkey());
        assert!(metadata.primary_sale_happened);
        assert!(!metadata.is_mutable);
    }

    #[tokio::test]
    async fn success_update_by_items_collection_delegate() {
        let delegate_args = DelegateArgs::CollectionV1 {
            authorization_data: None,
        };

        let new_collection = Collection {
            verified: false,
            key: Keypair::new().pubkey(),
        };

        let mut update_args = UpdateArgs::default_collection_delegate();
        match &mut update_args {
            UpdateArgs::CollectionDelegateV2 { collection, .. } => {
                *collection = CollectionToggle::Set(new_collection.clone())
            }
            _ => panic!("Unexpected enum variant"),
        }

        success_update_collection_by_items_delegate(delegate_args, new_collection, update_args)
            .await;
    }

    #[tokio::test]
    async fn success_update_by_items_collection_item_delegate() {
        let delegate_args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };

        let new_collection = Collection {
            verified: false,
            key: Keypair::new().pubkey(),
        };

        let mut update_args = UpdateArgs::default_collection_item_delegate();
        match &mut update_args {
            UpdateArgs::CollectionItemDelegateV2 { collection, .. } => {
                *collection = CollectionToggle::Set(new_collection.clone())
            }
            _ => panic!("Unexpected enum variant"),
        }

        success_update_collection_by_items_delegate(delegate_args, new_collection, update_args)
            .await;
    }

    async fn success_update_collection_by_items_delegate(
        delegate_args: DelegateArgs,
        collection: Collection,
        update_args: UpdateArgs,
    ) {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        da.create(context, TokenStandard::NonFungible, None)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);

        // Create metadata delegate.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_record = da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Change the collection.
        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values
        let metadata = da.get_metadata(context).await;

        assert_eq!(metadata.collection, Some(collection));
    }

    #[tokio::test]
    async fn success_update_by_items_data_delegate() {
        let delegate_args = DelegateArgs::DataV1 {
            authorization_data: None,
        };

        success_update_data_by_items_delegate(delegate_args, UpdateArgs::default_data_delegate())
            .await;
    }

    #[tokio::test]
    async fn success_update_by_items_data_item_delegate() {
        let delegate_args = DelegateArgs::DataItemV1 {
            authorization_data: None,
        };

        success_update_data_by_items_delegate(
            delegate_args,
            UpdateArgs::default_data_item_delegate(),
        )
        .await;
    }

    async fn success_update_data_by_items_delegate(
        delegate_args: DelegateArgs,
        mut update_args: UpdateArgs,
    ) {
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

        // Create metadata delegate.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_record = da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Change some values that this delegate is allowed to change.
        let new_name = puffed_out_string("New Name", MAX_NAME_LENGTH);
        let new_symbol = puffed_out_string("NEW", MAX_SYMBOL_LENGTH);
        let new_uri = puffed_out_string("https://new.digital.asset.org", MAX_URI_LENGTH);
        let data = Data {
            name: new_name.clone(),
            symbol: new_symbol.clone(),
            uri: new_uri.clone(),
            creators: metadata.data.creators, // keep the same creators
            seller_fee_basis_points: 0,
        };

        match &mut update_args {
            UpdateArgs::DataDelegateV2 {
                data: current_data, ..
            } => *current_data = Some(data),
            UpdateArgs::DataItemDelegateV2 {
                data: current_data, ..
            } => *current_data = Some(data),
            _ => panic!("Unexpected enum variant"),
        };

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
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
    async fn success_update_pfnt_config_by_update_authority() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority, false).await;

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

        // remove the rule set
        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 { rule_set, .. } => *rule_set = RuleSetToggle::Clear,
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(update_authority.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

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
    async fn success_update_pnft_by_items_programmable_config_delegate() {
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        let mut update_args = UpdateArgs::default_prog_config_delegate();
        match &mut update_args {
            UpdateArgs::ProgConfigDelegateV2 { rule_set, .. } => *rule_set = RuleSetToggle::Clear,
            _ => panic!("Unexpected enum variant"),
        }

        success_update_pnft_by_items_delegate(delegate_args, update_args).await;
    }

    #[tokio::test]
    async fn success_update_pnft_by_items_programmable_config_item_delegate() {
        let delegate_args = DelegateArgs::ProgrammableConfigItemV1 {
            authorization_data: None,
        };

        let mut update_args = UpdateArgs::default_prog_config_item_delegate();
        match &mut update_args {
            UpdateArgs::ProgConfigItemDelegateV2 { rule_set, .. } => {
                *rule_set = RuleSetToggle::Clear
            }
            _ => panic!("Unexpected enum variant"),
        }

        success_update_pnft_by_items_delegate(delegate_args, update_args).await;
    }

    async fn success_update_pnft_by_items_delegate(
        delegate_args: DelegateArgs,
        update_args: UpdateArgs,
    ) {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority, false).await;

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

        // Create metadata delegate.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_record = da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Change a value that this delegate is allowed to change.
        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values
        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.programmable_config, None);
    }

    #[tokio::test]
    async fn fail_update_by_items_authority_delegate() {
        let args = DelegateArgs::AuthorityItemV1 {
            authorization_data: None,
        };

        fail_update_by_items_delegate(args).await;
    }

    #[tokio::test]
    async fn fail_update_by_items_collection_delegate() {
        let args = DelegateArgs::CollectionV1 {
            authorization_data: None,
        };

        fail_update_by_items_delegate(args).await;
    }

    #[tokio::test]
    async fn fail_update_by_items_data_delegate() {
        let args = DelegateArgs::DataV1 {
            authorization_data: None,
        };

        fail_update_by_items_delegate(args).await;
    }

    #[tokio::test]
    async fn fail_update_by_items_programmable_config_delegate() {
        let args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };

        fail_update_by_items_delegate(args).await;
    }

    #[tokio::test]
    async fn fail_update_by_items_data_item_delegate() {
        let args = DelegateArgs::DataItemV1 {
            authorization_data: None,
        };

        fail_update_by_items_delegate(args).await;
    }

    #[tokio::test]
    async fn fail_update_by_items_collection_item_delegate() {
        let args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };

        fail_update_by_items_delegate(args).await;
    }

    #[tokio::test]
    async fn fail_update_by_items_programmable_config_item_delegate() {
        let args = DelegateArgs::ProgrammableConfigItemV1 {
            authorization_data: None,
        };

        fail_update_by_items_delegate(args).await;
    }

    async fn fail_update_by_items_delegate(delegate_args: DelegateArgs) {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority, false).await;

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

        // Create metadata delegate.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_record = da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Use update args variant that no delegates are allowed to use.
        let update_args = UpdateArgs::default_update_authority();
        match update_args {
            UpdateArgs::UpdateAuthorityV2 { .. } => (),
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidUpdateArgs);
    }

    #[tokio::test]
    async fn fail_update_by_items_persistent_delegate() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

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

        // Create `TokenDelegate` type of delegate.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_args = DelegateArgs::UtilityV1 {
            amount: 1,
            authorization_data: None,
        };
        let delegate_record = da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Use update args variant that no delegates are allowed to use.
        let update_args = UpdateArgs::default_update_authority();
        match update_args {
            UpdateArgs::UpdateAuthorityV2 { .. } => (),
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);
    }

    #[tokio::test]
    async fn success_update_token_standard() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        // This creates with update authority as a verified creator.
        da.create_and_mint(context, TokenStandard::FungibleAsset, None, None, 1)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));

        // Update token standard
        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 { token_standard, .. } => {
                *token_standard = Some(TokenStandard::Fungible)
            }
            _ => panic!("Unexpected enum variant"),
        }

        da.update(context, update_authority.dirty_clone(), args)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.token_standard, Some(TokenStandard::Fungible));
    }

    #[tokio::test]
    async fn fail_invalid_update_token_standard() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        // This creates with update authority as a verified creator.
        da.create_and_mint(context, TokenStandard::FungibleAsset, None, None, 1)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));

        // Update token standard
        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 { token_standard, .. } => {
                *token_standard = Some(TokenStandard::NonFungible)
            }
            _ => panic!("Unexpected enum variant"),
        }

        let err = da
            .update(context, update_authority.dirty_clone(), args)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidTokenStandard);

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.token_standard, Some(TokenStandard::FungibleAsset));
    }

    #[tokio::test]
    async fn fail_update_to_verified_collection() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        // This creates with update authority as a verified creator.
        da.create_and_mint(context, TokenStandard::FungibleAsset, None, None, 1)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);

        // Set collection to a value with verified set to true.
        let new_collection = Collection {
            verified: true,
            key: Keypair::new().pubkey(),
        };

        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 { collection, .. } => {
                *collection = CollectionToggle::Set(new_collection.clone())
            }
            _ => panic!("Unexpected enum variant"),
        }

        let err = da
            .update(context, update_authority.dirty_clone(), args)
            .await
            .unwrap_err();

        assert_custom_error!(
            err,
            MetadataError::CollectionCannotBeVerifiedInThisInstruction
        );

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);
    }

    #[tokio::test]
    async fn success_update_collection_by_collections_collection_delegate() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create metadata delegate on the collection.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_args = DelegateArgs::CollectionV1 {
            authorization_data: None,
        };
        let delegate_record = collection_parent_da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Create and mint item.
        let mut da = DigitalAsset::new();
        da.create_and_mint(context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);

        // Change collection.
        let new_collection = Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        };

        let mut args = UpdateArgs::default_collection_delegate();
        match &mut args {
            UpdateArgs::CollectionDelegateV2 { collection, .. } => {
                *collection = CollectionToggle::Set(new_collection.clone())
            }
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // Check that collection changed.
        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, Some(new_collection));
    }

    #[tokio::test]
    async fn fail_update_collection_by_collections_collection_item_delegate() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create metadata delegate on the collection.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_args = DelegateArgs::CollectionItemV1 {
            authorization_data: None,
        };
        let delegate_record = collection_parent_da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Create and mint item.
        let mut da = DigitalAsset::new();
        da.create_and_mint(context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);

        // Change collection.
        let new_collection = Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        };

        let mut args = UpdateArgs::default_collection_item_delegate();
        match &mut args {
            UpdateArgs::CollectionItemDelegateV2 { collection, .. } => {
                *collection = CollectionToggle::Set(new_collection.clone())
            }
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);

        // Check that collection not changed.
        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);
    }

    #[tokio::test]
    async fn fail_update_collection_by_collections_programmable_config_delegate() {
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create metadata delegate on the collection.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };
        let delegate_record = collection_parent_da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Create and mint item.
        let mut da = DigitalAsset::new();
        da.create_and_mint(context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);

        // Change collection.
        let new_collection = Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        };

        let mut args = UpdateArgs::default_collection_delegate();
        match &mut args {
            UpdateArgs::CollectionDelegateV2 { collection, .. } => {
                *collection = CollectionToggle::Set(new_collection.clone())
            }
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidUpdateArgs);

        // Check that collection not changed.
        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);
    }

    #[tokio::test]
    async fn success_update_prog_config_by_collections_prog_config_delegate_v2_args() {
        // Change programmable config, removing the RuleSet.
        let mut args = UpdateArgs::default_prog_config_delegate();
        match &mut args {
            UpdateArgs::ProgConfigDelegateV2 { rule_set, .. } => *rule_set = RuleSetToggle::Clear,
            _ => panic!("Unexpected enum variant"),
        }

        success_update_prog_config_by_collections_prog_config_delegate(args).await
    }

    #[tokio::test]
    async fn success_update_prog_config_by_collections_prog_config_delegate_v1_args() {
        // Change programmable config, removing the RuleSet.
        let mut args = UpdateArgs::default_v1();
        match &mut args {
            UpdateArgs::V1 { rule_set, .. } => *rule_set = RuleSetToggle::Clear,
            _ => panic!("Unexpected enum variant"),
        }

        success_update_prog_config_by_collections_prog_config_delegate(args).await
    }

    async fn success_update_prog_config_by_collections_prog_config_delegate(
        update_args: UpdateArgs,
    ) {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create metadata delegate on the collection.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };
        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let delegate_record = collection_parent_da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Create rule-set for the transfer
        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority, false).await;

        // Create and mint item with a collection.  THIS IS NEEDED so that the collection-level
        // delegate is authorized for this item.
        let collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            context,
            TokenStandard::ProgrammableNonFungible,
            Some(authorization_rules),
            Some(auth_data),
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        let metadata = da.get_metadata(context).await;

        // Check collection.
        assert_eq!(metadata.collection, collection);

        // Check programmable config.
        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(rule_set),
        }) = metadata.programmable_config
        {
            assert_eq!(rule_set, authorization_rules);
        } else {
            panic!("Missing rule set programmable config");
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        // checks the created metadata values
        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.programmable_config, None);
    }

    #[tokio::test]
    async fn fail_update_prog_config_by_col_prog_config_delegate_wrong_v1_args() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create metadata delegate on the collection.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };
        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let delegate_record = collection_parent_da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Create rule-set for the transfer
        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority, false).await;

        // Create and mint item with a collection.  THIS IS NEEDED so that the collection-level
        // delegate is authorized for this item.
        let collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            context,
            TokenStandard::ProgrammableNonFungible,
            Some(authorization_rules),
            Some(auth_data),
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        // Check collection.
        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, collection);

        // Check primary sale.
        let metadata = da.get_metadata(context).await;
        assert!(!metadata.primary_sale_happened);

        // Collection-level programmable config delegate is allowed to use V1 args for backwards
        // compatibility.  But RuleSet is the only allowed field this delegate can change.
        let mut args = UpdateArgs::default_v1();
        match &mut args {
            UpdateArgs::V1 {
                primary_sale_happened,
                ..
            } => *primary_sale_happened = Some(true),
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidUpdateArgs);

        // Check that metadata not changed.
        let metadata = da.get_metadata(context).await;
        assert!(!metadata.primary_sale_happened);
    }

    #[tokio::test]
    async fn fail_update_by_col_prog_config_delegate_using_new_collection_in_v1_args() {
        let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::ID, None);
        program_test.add_program("mpl_token_auth_rules", mpl_token_auth_rules::ID, None);
        let context = &mut program_test.start_with_context().await;

        // Create a collection parent NFT or pNFT with the CollectionDetails struct populated.
        let mut collection_parent_da = DigitalAsset::new();
        collection_parent_da
            .create_and_mint_collection_parent(
                context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
                DEFAULT_COLLECTION_DETAILS,
            )
            .await
            .unwrap();

        // Create metadata delegate on the collection.
        let delegate = Keypair::new();
        delegate.airdrop(context, 1_000_000_000).await.unwrap();
        let delegate_args = DelegateArgs::ProgrammableConfigV1 {
            authorization_data: None,
        };
        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let delegate_record = collection_parent_da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Create rule-set for the transfer
        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority, false).await;

        // Create and mint item with a collection.  THIS IS NEEDED so that the collection-level
        // delegate is authorized for this item.
        let collection = Some(Collection {
            key: collection_parent_da.mint.pubkey(),
            verified: false,
        });

        let mut da = DigitalAsset::new();
        da.create_and_mint_item_with_collection(
            context,
            TokenStandard::ProgrammableNonFungible,
            Some(authorization_rules),
            Some(auth_data),
            1,
            collection.clone(),
        )
        .await
        .unwrap();

        // Check collection.
        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, collection);

        let new_collection = Collection {
            key: Keypair::new().pubkey(),
            verified: false,
        };

        // Collection-level programmable config delegate is allowed to use V1 args for backwards
        // compatibility.  But RuleSet is the only allowed field this delegate can change.  But it
        // won't get to that point in the code because it will not be authorized as a collection-
        // level delegate based on the new collection it sent in.
        let mut args = UpdateArgs::default_v1();
        match &mut args {
            UpdateArgs::V1 { collection, .. } => {
                *collection = CollectionToggle::Set(new_collection.clone())
            }
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(delegate.pubkey())
            .delegate_record(delegate_record)
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(delegate.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&delegate.pubkey()),
            &[&delegate],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::InvalidAuthorityType);

        // Check that collection not changed.
        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, collection);
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
            create_default_metaplex_rule_set(context, authority, false).await;

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

        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 { rule_set, .. } => {
                *rule_set = RuleSetToggle::Set(invalid_rule_set)
            }
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .payer(update_authority.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args.clone()).unwrap().instruction();

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

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

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
        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 { rule_set, .. } => {
                *rule_set = RuleSetToggle::Set(authorization_rules)
            }
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(invalid_rule_set)
            .payer(update_authority.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

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
        program_test.set_compute_max_units(400_000);
        let context = &mut program_test.start_with_context().await;

        let authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        // Create rule-set for the transfer
        let (authorization_rules, auth_data) =
            create_default_metaplex_rule_set(context, authority.dirty_clone(), false).await;

        let (new_auth_rules, _) =
            create_default_metaplex_rule_set(context, authority.dirty_clone(), false).await;

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
            context,
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
        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 { rule_set, .. } => *rule_set = RuleSetToggle::Clear,
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(authorization_rules)
            .payer(update_authority.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

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
        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 { rule_set, .. } => {
                *rule_set = RuleSetToggle::Set(new_auth_rules)
            }
            _ => panic!("Unexpected enum variant"),
        }

        let mut builder = UpdateBuilder::new();
        builder
            .authority(update_authority.pubkey())
            .metadata(da.metadata)
            .mint(da.mint.pubkey())
            .token(da.token.unwrap())
            .authorization_rules(new_auth_rules)
            .payer(update_authority.pubkey());

        if let Some(edition) = da.edition {
            builder.edition(edition);
        }

        let update_ix = builder.build(args).unwrap().instruction();

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

    #[tokio::test]
    async fn none_does_not_erase_verified_creators() {
        // When passing in `None` for the creators field, it should not erase the verified creators.
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        // This creates with update authority as a verified creator.
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
            creators: None, // This should not erase the verified creator.
            seller_fee_basis_points: 0,
        };

        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 {
                data: current_data, ..
            } => *current_data = Some(data),
            _ => panic!("Unexpected enum variant"),
        }

        let err = da
            .update(context, update_authority.dirty_clone(), args)
            .await
            .unwrap_err();

        assert_custom_error!(err, MetadataError::CannotRemoveVerifiedCreator);
    }

    #[tokio::test]
    async fn set_creators_to_none_with_no_verified_creators() {
        // When passing in `None` for the creators field, it should set the creators
        // field to `None` if there are no verified creators.
        let context = &mut program_test().start_with_context().await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = DigitalAsset::new();
        // This creates with update authority as a verified creator.
        da.create_and_mint(context, TokenStandard::NonFungible, None, None, 1)
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

        // Unverify the creator.
        let creators = Some(vec![Creator {
            address: context.payer.pubkey(),
            share: 100,
            verified: false,
        }]);

        let data = Data {
            name: metadata.data.name,
            symbol: metadata.data.symbol,
            uri: metadata.data.uri,
            creators: creators.clone(),
            seller_fee_basis_points: metadata.data.seller_fee_basis_points,
        };

        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 {
                data: current_data, ..
            } => *current_data = Some(data),
            _ => panic!("Unexpected enum variant"),
        }

        da.update(context, update_authority.dirty_clone(), args)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;

        assert_eq!(metadata.data.creators, creators);
        assert!(!metadata.data.creators.unwrap()[0].verified);

        // Now set the creators to None.
        let data = Data {
            name: metadata.data.name,
            symbol: metadata.data.symbol,
            uri: metadata.data.uri,
            creators: None,
            seller_fee_basis_points: metadata.data.seller_fee_basis_points,
        };

        let mut args = UpdateArgs::default_update_authority();
        match &mut args {
            UpdateArgs::UpdateAuthorityV2 {
                data: current_data, ..
            } => *current_data = Some(data),
            _ => panic!("Unexpected enum variant"),
        }

        da.update(context, update_authority.dirty_clone(), args)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;

        assert_eq!(metadata.data.creators, None);
    }
}
