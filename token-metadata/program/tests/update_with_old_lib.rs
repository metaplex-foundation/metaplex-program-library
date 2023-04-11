#![cfg(feature = "test-bpf")]

use old_token_metadata::{
    id,
    instruction::{builders::UpdateBuilder, InstructionBuilder},
    instruction::{
        builders::{CreateBuilder, DelegateBuilder},
        CreateArgs, MetadataDelegateRole,
    },
    instruction::{CollectionToggle, DelegateArgs, UpdateArgs},
    pda::{find_metadata_delegate_record_account, find_token_record_account},
    state::{
        AssetData, CollectionDetails, Metadata, PrintSupply, TokenMetadataAccount, EDITION, PREFIX,
    },
    state::{Collection, Creator, ProgrammableConfig, TokenStandard},
};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

// This tests backwards compatibility of v1.10 changes with an older version of Token Metadata.
// It uses the binary created from the v1.10 version of token metadata but imports older instructions from
// the 1.9.1 version to ensure that the old instructions still work.

// Note that to avoid version conflicts, requied test utilities are re-implemented in this file, including
// an `OldDigitalAsset` struct that is a limited version of `DigitalAsset` and compatible with 1.9.1.

mod update {
    use super::*;

    #[tokio::test]
    async fn success_update_by_items_collection_item_delegate() {
        let context = &mut ProgramTest::new("mpl_token_metadata", mpl_token_metadata::id(), None)
            .start_with_context()
            .await;

        let update_authority = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        let mut da = OldDigitalAsset::new();
        da.create(context, TokenStandard::NonFungible, None)
            .await
            .unwrap();

        let metadata = da.get_metadata(context).await;
        assert_eq!(metadata.collection, None);

        // Create metadata delegate.
        let delegate = Keypair::new();
        airdrop(context, &delegate.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let delegate_args = DelegateArgs::CollectionV1 {
            authorization_data: None,
        };
        let delegate_record = da
            .delegate(context, update_authority, delegate.pubkey(), delegate_args)
            .await
            .unwrap()
            .unwrap();

        // Change a value that this delegate is allowed to change.
        let mut update_args = UpdateArgs::default();
        let UpdateArgs::V1 {
            collection: collection_toggle,
            ..
        } = &mut update_args;
        let new_collection = Collection {
            verified: false,
            key: Keypair::new().pubkey(),
        };
        *collection_toggle = CollectionToggle::Set(new_collection.clone());

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

        assert_eq!(metadata.collection, Some(new_collection));
    }
}

async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
    Ok(())
}

async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

// This represents a generic Metaplex Digital asset of various Token Standards.
// It is used to abstract away the various accounts that are created for a given
// Digital Asset. Since different asset types have different accounts, care
// should be taken that appropriate handlers update appropriate accounts, such as when
// transferring a DigitalAsset, the token account should be updated.
struct OldDigitalAsset {
    pub metadata: Pubkey,
    pub mint: Keypair,
    pub token: Option<Pubkey>,
    pub edition: Option<Pubkey>,
    pub token_standard: Option<TokenStandard>,
}

impl Default for OldDigitalAsset {
    fn default() -> Self {
        Self::new()
    }
}

impl OldDigitalAsset {
    fn new() -> Self {
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();
        let program_id = id();

        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (metadata, _) = Pubkey::find_program_address(metadata_seeds, &program_id);

        Self {
            metadata,
            mint,
            token: None,
            edition: None,
            token_standard: None,
        }
    }

    async fn create(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        authorization_rules: Option<Pubkey>,
    ) -> Result<(), BanksClientError> {
        let creators = Some(vec![Creator {
            address: context.payer.pubkey(),
            share: 100,
            verified: true,
        }]);

        self.create_advanced(
            context,
            token_standard,
            String::from("Old Digital Asset"),
            String::from("DA"),
            String::from("https://digital.asset.org"),
            500,
            creators,
            None,
            None,
            authorization_rules,
            PrintSupply::Zero,
        )
        .await
    }

    async fn create_advanced(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        name: String,
        symbol: String,
        uri: String,
        seller_fee_basis_points: u16,
        creators: Option<Vec<Creator>>,
        collection: Option<Collection>,
        collection_details: Option<CollectionDetails>,
        authorization_rules: Option<Pubkey>,
        print_supply: PrintSupply,
    ) -> Result<(), BanksClientError> {
        let mut asset = AssetData::new(token_standard, name, symbol, uri);
        asset.seller_fee_basis_points = seller_fee_basis_points;
        asset.creators = creators;
        asset.collection = collection;
        asset.collection_details = collection_details;
        asset.rule_set = authorization_rules;

        let payer_pubkey = context.payer.pubkey();
        let mint_pubkey = self.mint.pubkey();

        let program_id = id();
        let mut builder = CreateBuilder::new();
        builder
            .metadata(self.metadata)
            .mint(self.mint.pubkey())
            .authority(payer_pubkey)
            .payer(payer_pubkey)
            .update_authority(payer_pubkey)
            .initialize_mint(true)
            .update_authority_as_signer(true);

        let edition = match token_standard {
            TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible => {
                // master edition PDA address
                let edition_seeds = &[
                    PREFIX.as_bytes(),
                    program_id.as_ref(),
                    mint_pubkey.as_ref(),
                    EDITION.as_bytes(),
                ];
                let (edition, _) = Pubkey::find_program_address(edition_seeds, &id());
                // sets the master edition to the builder
                builder.master_edition(edition);
                Some(edition)
            }
            _ => None,
        };
        // builds the instruction
        let create_ix = builder
            .build(CreateArgs::V1 {
                asset_data: asset,
                decimals: Some(0),
                print_supply: Some(print_supply),
            })
            .unwrap()
            .instruction();

        let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(800_000);

        let tx = Transaction::new_signed_with_payer(
            &[compute_ix, create_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.mint],
            context.last_blockhash,
        );

        self.edition = edition;
        self.token_standard = Some(token_standard);

        context.banks_client.process_transaction(tx).await
    }

    async fn delegate(
        &mut self,
        context: &mut ProgramTestContext,
        payer: Keypair,
        delegate: Pubkey,
        args: DelegateArgs,
    ) -> Result<Option<Pubkey>, BanksClientError> {
        let mut builder = DelegateBuilder::new();
        builder
            .delegate(delegate)
            .mint(self.mint.pubkey())
            .metadata(self.metadata)
            .payer(payer.pubkey())
            .authority(payer.pubkey())
            .spl_token_program(spl_token::ID);

        let mut delegate_or_token_record = None;

        match args {
            // Token delegates.
            DelegateArgs::SaleV1 { .. }
            | DelegateArgs::TransferV1 { .. }
            | DelegateArgs::UtilityV1 { .. }
            | DelegateArgs::StakingV1 { .. }
            | DelegateArgs::LockedTransferV1 { .. } => {
                let (token_record, _) =
                    find_token_record_account(&self.mint.pubkey(), &self.token.unwrap());
                builder.token_record(token_record);
                delegate_or_token_record = Some(token_record);
            }
            DelegateArgs::StandardV1 { .. } => { /* nothing to add */ }

            // Metadata delegates.
            DelegateArgs::CollectionV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Collection,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
                delegate_or_token_record = Some(delegate_record);
            }
            DelegateArgs::UpdateV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Update,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
                delegate_or_token_record = Some(delegate_record);
            }
            DelegateArgs::ProgrammableConfigV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::ProgrammableConfig,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
                delegate_or_token_record = Some(delegate_record);
            }
        }

        if let Some(edition) = self.edition {
            builder.master_edition(edition);
        }

        if let Some(token) = self.token {
            builder.token(token);
        }

        // determines if we need to set the rule set
        let metadata_account = get_account(context, &self.metadata).await;
        let metadata: Metadata = try_from_slice_unchecked(&metadata_account.data).unwrap();

        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(rule_set),
        }) = metadata.programmable_config
        {
            builder.authorization_rules(rule_set);
            builder.authorization_rules_program(mpl_token_auth_rules::ID);
        }

        let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);

        let delegate_ix = builder.build(args.clone()).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[compute_ix, delegate_ix],
            Some(&payer.pubkey()),
            &[&payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await?;
        Ok(delegate_or_token_record)
    }

    pub async fn get_metadata(&self, context: &mut ProgramTestContext) -> Metadata {
        let metadata_account = context
            .banks_client
            .get_account(self.metadata)
            .await
            .unwrap()
            .unwrap();

        Metadata::safe_deserialize(&metadata_account.data).unwrap()
    }
}
