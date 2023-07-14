use mpl_token_metadata::{
    instruction::{
        self,
        builders::{
            BurnBuilder, CreateBuilder, DelegateBuilder, LockBuilder, MintBuilder, RevokeBuilder,
            TransferBuilder, UnlockBuilder, UnverifyBuilder, UpdateBuilder, VerifyBuilder,
        },
        BurnArgs, CollectionDetailsToggle, CollectionToggle, CreateArgs, DelegateArgs,
        InstructionBuilder, LockArgs, MetadataDelegateRole, MintArgs, RevokeArgs, RuleSetToggle,
        TransferArgs, UnlockArgs, UpdateArgs, UsesToggle, VerificationArgs,
    },
    pda::{
        find_master_edition_account, find_metadata_account, find_metadata_delegate_record_account,
        find_token_record_account,
    },
    processor::AuthorizationData,
    state::{
        AssetData, Collection, CollectionDetails, Creator, MasterEditionV2, Metadata, PrintSupply,
        ProgrammableConfig, TokenDelegateRole, TokenMetadataAccount, TokenRecord, TokenStandard,
        CREATE_FEE, EDITION, EDITION_MARKER_BIT_SIZE, FEE_FLAG_SET, METADATA_FEE_FLAG_INDEX,
        PREFIX,
    },
    ID,
};
use solana_program::{
    borsh::try_from_slice_unchecked, program_option::COption, program_pack::Pack, pubkey::Pubkey,
};
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    account::AccountSharedData,
    compute_budget::ComputeBudgetInstruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::state::Account;

use super::{airdrop, create_mint, create_token_account, get_account, mint_tokens};

pub const DEFAULT_NAME: &str = "Digital Asset";
pub const DEFAULT_SYMBOL: &str = "DA";
pub const DEFAULT_URI: &str = "https://digital.asset.org";

// This represents a generic Metaplex Digital asset of various Token Standards.
// It is used to abstract away the various accounts that are created for a given
// Digital Asset. Since different asset types have different accounts, care
// should be taken that appropriate handlers update appropriate accounts, such as when
// transferring a DigitalAsset, the token account should be updated.
pub struct DigitalAsset {
    pub metadata: Pubkey,
    pub mint: Keypair,
    pub token: Option<Pubkey>,
    pub edition: Option<Pubkey>,
    pub token_record: Option<Pubkey>,
    pub token_standard: Option<TokenStandard>,
    pub edition_num: Option<u64>,
}

impl Default for DigitalAsset {
    fn default() -> Self {
        Self::new()
    }
}

impl DigitalAsset {
    pub fn new() -> Self {
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();
        let program_id = ID;

        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (metadata, _) = Pubkey::find_program_address(metadata_seeds, &program_id);

        Self {
            metadata,
            mint,
            token: None,
            edition: None,
            token_record: None,
            token_standard: None,
            edition_num: None,
        }
    }

    pub fn set_edition(&mut self) {
        let edition = find_master_edition_account(&self.mint.pubkey()).0;
        self.edition = Some(edition);
    }

    pub async fn burn(
        &mut self,
        context: &mut ProgramTestContext,
        authority: Keypair,
        args: BurnArgs,
        parent_asset: Option<DigitalAsset>,
        collection_metadata: Option<Pubkey>,
    ) -> Result<(), BanksClientError> {
        let md = self.get_metadata(context).await;
        let token_standard = md.token_standard.unwrap();

        let mut builder = BurnBuilder::new();
        builder
            .authority(authority.pubkey())
            .metadata(self.metadata)
            .mint(self.mint.pubkey())
            .token(self.token.unwrap());

        if let Some(parent_asset) = parent_asset {
            builder.master_edition_mint(parent_asset.mint.pubkey());
            builder.master_edition_token(parent_asset.token.unwrap());
            builder.master_edition(parent_asset.edition.unwrap());

            let edition_num = self.edition_num.unwrap();

            let marker_num = edition_num.checked_div(EDITION_MARKER_BIT_SIZE).unwrap();

            let (edition_marker, _) = Pubkey::find_program_address(
                &[
                    PREFIX.as_bytes(),
                    mpl_token_metadata::ID.as_ref(),
                    parent_asset.mint.pubkey().as_ref(),
                    EDITION.as_bytes(),
                    marker_num.to_string().as_bytes(),
                ],
                &mpl_token_metadata::ID,
            );
            builder.edition_marker(edition_marker);
        }

        if let Some(edition) = self.edition {
            builder.edition(edition);
        }

        if token_standard == TokenStandard::ProgrammableNonFungible {
            builder.token_record(self.token_record.unwrap());
        }

        if let Some(collection_metadata) = collection_metadata {
            builder.collection_metadata(collection_metadata);
        }

        let burn_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[burn_ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(transaction).await
    }

    // Note the authority is the payer of the transaction.
    pub async fn verify(
        &mut self,
        context: &mut ProgramTestContext,
        authority: Keypair,
        args: VerificationArgs,
        metadata: Option<Pubkey>,
        delegate_record: Option<Pubkey>,
        collection_mint: Option<Pubkey>,
        collection_metadata: Option<Pubkey>,
        collection_master_edition: Option<Pubkey>,
    ) -> Result<(), BanksClientError> {
        let mut builder = VerifyBuilder::new();
        builder
            .authority(authority.pubkey())
            .metadata(metadata.unwrap_or(self.metadata));

        match args {
            VerificationArgs::CreatorV1 => (),
            VerificationArgs::CollectionV1 => {
                if let Some(delegate_record) = delegate_record {
                    builder.delegate_record(delegate_record);
                }

                if let Some(collection_mint) = collection_mint {
                    builder.collection_mint(collection_mint);
                }

                if let Some(collection_metadata) = collection_metadata {
                    builder.collection_metadata(collection_metadata);
                }

                if let Some(collection_master_edition) = collection_master_edition {
                    builder.collection_master_edition(collection_master_edition);
                }
            }
        }

        let verify_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[verify_ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(transaction).await
    }

    // Note the authority is the payer of the transaction.
    pub async fn unverify(
        &mut self,
        context: &mut ProgramTestContext,
        authority: Keypair,
        args: VerificationArgs,
        metadata: Option<Pubkey>,
        delegate_record: Option<Pubkey>,
        collection_mint: Option<Pubkey>,
        collection_metadata: Option<Pubkey>,
    ) -> Result<(), BanksClientError> {
        let mut builder = UnverifyBuilder::new();
        builder
            .authority(authority.pubkey())
            .metadata(metadata.unwrap_or(self.metadata));

        match args {
            VerificationArgs::CreatorV1 => (),
            VerificationArgs::CollectionV1 => {
                if let Some(delegate_record) = delegate_record {
                    builder.delegate_record(delegate_record);
                }

                if let Some(collection_mint) = collection_mint {
                    builder.collection_mint(collection_mint);
                }

                if let Some(collection_metadata) = collection_metadata {
                    builder.collection_metadata(collection_metadata);
                }
            }
        }

        let unverify_ix = builder.build(args).unwrap().instruction();

        let transaction = Transaction::new_signed_with_payer(
            &[unverify_ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(transaction).await
    }

    pub async fn create(
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
            String::from(DEFAULT_NAME),
            String::from(DEFAULT_SYMBOL),
            String::from(DEFAULT_URI),
            500,
            creators,
            None,
            None,
            authorization_rules,
            PrintSupply::Zero,
        )
        .await
    }

    pub async fn create_advanced(
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

        let program_id = ID;
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
                let (edition, _) = Pubkey::find_program_address(edition_seeds, &ID);
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

    pub async fn mint(
        &mut self,
        context: &mut ProgramTestContext,
        authorization_rules: Option<Pubkey>,
        authorization_data: Option<AuthorizationData>,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let payer_pubkey = context.payer.pubkey();
        let (token, _) = Pubkey::find_program_address(
            &[
                &payer_pubkey.to_bytes(),
                &spl_token::ID.to_bytes(),
                &self.mint.pubkey().to_bytes(),
            ],
            &spl_associated_token_account::ID,
        );

        let (token_record, _) = find_token_record_account(&self.mint.pubkey(), &token);

        let token_record_opt = if self.is_pnft(context).await {
            Some(token_record)
        } else {
            None
        };

        let mut builder = MintBuilder::new();
        builder
            .token(token)
            .token_record(token_record)
            .token_owner(payer_pubkey)
            .metadata(self.metadata)
            .mint(self.mint.pubkey())
            .payer(payer_pubkey)
            .authority(payer_pubkey);

        if let Some(edition) = self.edition {
            builder.master_edition(edition);
        }

        if let Some(authorization_rules) = authorization_rules {
            builder.authorization_rules(authorization_rules);
        }

        let mint_ix = builder
            .build(MintArgs::V1 {
                amount,
                authorization_data,
            })
            .unwrap()
            .instruction();

        let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(800_000);

        let tx = Transaction::new_signed_with_payer(
            &[compute_ix, mint_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.map(|_| {
            self.token = Some(token);
            self.token_record = token_record_opt;
        })
    }

    pub async fn create_and_mint(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        authorization_rules: Option<Pubkey>,
        authorization_data: Option<AuthorizationData>,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        // creates the metadata
        self.create(context, token_standard, authorization_rules)
            .await
            .unwrap();
        // mints tokens
        self.mint(context, authorization_rules, authorization_data, amount)
            .await
    }

    pub async fn create_and_mint_with_supply(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        authorization_rules: Option<Pubkey>,
        authorization_data: Option<AuthorizationData>,
        amount: u64,
        print_supply: PrintSupply,
    ) -> Result<(), BanksClientError> {
        // creates the metadata

        let creators = Some(vec![Creator {
            address: context.payer.pubkey(),
            share: 100,
            verified: true,
        }]);

        self.create_advanced(
            context,
            token_standard,
            String::from(DEFAULT_NAME),
            String::from(DEFAULT_SYMBOL),
            String::from(DEFAULT_URI),
            500,
            creators,
            None,
            None,
            authorization_rules,
            print_supply,
        )
        .await?;

        // mints tokens
        self.mint(context, authorization_rules, authorization_data, amount)
            .await
    }

    pub async fn create_and_mint_with_creators(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        authorization_rules: Option<Pubkey>,
        authorization_data: Option<AuthorizationData>,
        amount: u64,
        creators: Option<Vec<Creator>>,
    ) -> Result<(), BanksClientError> {
        // creates the metadata
        self.create_advanced(
            context,
            token_standard,
            String::from(DEFAULT_NAME),
            String::from(DEFAULT_SYMBOL),
            String::from(DEFAULT_URI),
            500,
            creators,
            None,
            None,
            authorization_rules,
            PrintSupply::Zero,
        )
        .await
        .unwrap();

        // mints tokens
        self.mint(context, authorization_rules, authorization_data, amount)
            .await
    }

    pub async fn create_and_mint_item_with_collection(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        authorization_rules: Option<Pubkey>,
        authorization_data: Option<AuthorizationData>,
        amount: u64,
        collection: Option<Collection>,
    ) -> Result<(), BanksClientError> {
        // creates the metadata
        self.create_advanced(
            context,
            token_standard,
            String::from(DEFAULT_NAME),
            String::from(DEFAULT_SYMBOL),
            String::from(DEFAULT_URI),
            500,
            None,
            collection,
            None,
            authorization_rules,
            PrintSupply::Zero,
        )
        .await
        .unwrap();

        // mints tokens
        self.mint(context, authorization_rules, authorization_data, amount)
            .await
    }

    pub async fn create_and_mint_collection_parent(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        authorization_rules: Option<Pubkey>,
        authorization_data: Option<AuthorizationData>,
        amount: u64,
        collection_details: Option<CollectionDetails>,
    ) -> Result<(), BanksClientError> {
        // creates the metadata
        self.create_advanced(
            context,
            token_standard,
            String::from(DEFAULT_NAME),
            String::from(DEFAULT_SYMBOL),
            String::from(DEFAULT_URI),
            500,
            None,
            None,
            collection_details,
            authorization_rules,
            PrintSupply::Zero,
        )
        .await
        .unwrap();

        // mints tokens
        self.mint(context, authorization_rules, authorization_data, amount)
            .await
    }

    pub async fn create_and_mint_nonfungible(
        &mut self,
        context: &mut ProgramTestContext,
        print_supply: PrintSupply,
    ) -> Result<(), BanksClientError> {
        // creates the metadata
        self.create_advanced(
            context,
            TokenStandard::NonFungible,
            String::from(DEFAULT_NAME),
            String::from(DEFAULT_SYMBOL),
            String::from(DEFAULT_URI),
            500,
            None,
            None,
            None,
            None,
            print_supply,
        )
        .await
        .unwrap();

        // mints tokens
        self.mint(context, None, None, 1).await
    }

    pub async fn delegate(
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
            DelegateArgs::DataV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Data,
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
            DelegateArgs::AuthorityItemV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::AuthorityItem,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
                delegate_or_token_record = Some(delegate_record);
            }
            DelegateArgs::DataItemV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::DataItem,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
                delegate_or_token_record = Some(delegate_record);
            }
            DelegateArgs::CollectionItemV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::CollectionItem,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
                delegate_or_token_record = Some(delegate_record);
            }
            DelegateArgs::ProgrammableConfigItemV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::ProgrammableConfigItem,
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

    pub async fn print_edition(
        &self,
        context: &mut ProgramTestContext,
        edition_num: u64,
    ) -> Result<DigitalAsset, BanksClientError> {
        let print_mint = Keypair::new();
        let print_token = Keypair::new();
        let (print_metadata, _) = find_metadata_account(&print_mint.pubkey());
        let (print_edition, _) = find_master_edition_account(&print_mint.pubkey());

        create_mint(
            context,
            &print_mint,
            &context.payer.pubkey(),
            Some(&context.payer.pubkey()),
            0,
        )
        .await?;
        create_token_account(
            context,
            &print_token,
            &print_mint.pubkey(),
            &context.payer.pubkey(),
        )
        .await?;
        mint_tokens(
            context,
            &print_mint.pubkey(),
            &print_token.pubkey(),
            1,
            &context.payer.pubkey(),
            None,
        )
        .await?;

        let tx = Transaction::new_signed_with_payer(
            &[instruction::mint_new_edition_from_master_edition_via_token(
                ID,
                print_metadata,
                print_edition,
                self.edition.unwrap(),
                print_mint.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                self.token.unwrap(),
                context.payer.pubkey(),
                self.metadata,
                self.mint.pubkey(),
                edition_num,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction_with_commitment(
                tx,
                solana_sdk::commitment_config::CommitmentLevel::Confirmed,
            )
            .await
            .unwrap();

        Ok(DigitalAsset {
            mint: print_mint,
            token: Some(print_token.pubkey()),
            metadata: print_metadata,
            edition: Some(print_edition),
            token_standard: self.token_standard,
            token_record: None,
            edition_num: Some(edition_num),
        })
    }

    pub async fn revoke(
        &mut self,
        context: &mut ProgramTestContext,
        payer: Keypair,
        approver: Keypair,
        delegate: Pubkey,
        args: RevokeArgs,
    ) -> Result<(), BanksClientError> {
        let mut builder = RevokeBuilder::new();
        builder
            .delegate(delegate)
            .mint(self.mint.pubkey())
            .metadata(self.metadata)
            .payer(approver.pubkey())
            .authority(approver.pubkey())
            .spl_token_program(spl_token::ID);

        match args {
            // Token delegates.
            RevokeArgs::SaleV1
            | RevokeArgs::TransferV1
            | RevokeArgs::UtilityV1
            | RevokeArgs::StakingV1
            | RevokeArgs::LockedTransferV1
            | RevokeArgs::MigrationV1 => {
                let (token_record, _) =
                    find_token_record_account(&self.mint.pubkey(), &self.token.unwrap());
                builder.token_record(token_record);
            }
            RevokeArgs::StandardV1 { .. } => { /* nothing to add */ }

            // Metadata delegates.
            RevokeArgs::CollectionV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Collection,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            RevokeArgs::DataV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Data,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            RevokeArgs::ProgrammableConfigV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::ProgrammableConfig,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            RevokeArgs::AuthorityItemV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::AuthorityItem,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            RevokeArgs::DataItemV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::DataItem,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            RevokeArgs::CollectionItemV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::CollectionItem,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }

            RevokeArgs::ProgrammableConfigItemV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::ProgrammableConfigItem,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
        }

        if let Some(edition) = self.edition {
            builder.master_edition(edition);
        }

        if let Some(token) = self.token {
            builder.token(token);
        }

        let revoke_ix = builder.build(args.clone()).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[revoke_ix],
            Some(&payer.pubkey()),
            &[&approver, &payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    // This transfers a DigitalAsset from its existing Token Account to a new one
    // and should update the token account after a successful transfer, as well as the
    // token record if appropriate (for pNFTs).
    pub async fn transfer(&mut self, params: TransferParams<'_>) -> Result<(), BanksClientError> {
        let TransferParams {
            context,
            authority,
            source_owner,
            destination_owner,
            destination_token,
            authorization_rules,
            payer,
            args,
        } = params;

        let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(800_000);
        let mut instructions = vec![compute_ix];

        let destination_token = if let Some(destination_token) = destination_token {
            destination_token
        } else {
            instructions.push(create_associated_token_account(
                &authority.pubkey(),
                &destination_owner,
                &self.mint.pubkey(),
                &spl_token::ID,
            ));

            get_associated_token_address(&destination_owner, &self.mint.pubkey())
        };

        let mut builder = TransferBuilder::new();
        builder
            .authority(authority.pubkey())
            .token_owner(*source_owner)
            .token(self.token.unwrap())
            .destination_owner(destination_owner)
            .destination(destination_token)
            .metadata(self.metadata)
            .payer(payer.pubkey())
            .mint(self.mint.pubkey());

        if let Some(record) = self.token_record {
            builder.owner_token_record(record);
        }

        // This can be optional for non pNFTs but always include it for now.
        let (destination_token_record, _bump) =
            find_token_record_account(&self.mint.pubkey(), &destination_token);
        let destination_token_record_opt = if self.is_pnft(context).await {
            builder.destination_token_record(destination_token_record);
            Some(destination_token_record)
        } else {
            None
        };

        if let Some(edition) = self.edition {
            builder.edition(edition);
        }

        if let Some(authorization_rules) = authorization_rules {
            builder.authorization_rules(authorization_rules);
            builder.authorization_rules_program(mpl_token_auth_rules::ID);
        }

        let transfer_ix = builder.build(args).unwrap().instruction();

        instructions.push(transfer_ix);

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&authority.pubkey()),
            &[authority, payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.map(|_| {
            // Update token values for new owner.
            self.token = Some(destination_token);
            self.token_record = destination_token_record_opt;
        })
    }

    pub async fn lock(
        &mut self,
        context: &mut ProgramTestContext,
        delegate: Keypair,
        token_record: Option<Pubkey>,
        payer: Keypair,
    ) -> Result<(), BanksClientError> {
        let mut builder = LockBuilder::new();
        builder
            .authority(delegate.pubkey())
            .mint(self.mint.pubkey())
            .metadata(self.metadata)
            .payer(payer.pubkey())
            .spl_token_program(spl_token::ID);

        if let Some(token_record) = token_record {
            builder.token_record(token_record);
        }

        if let Some(edition) = self.edition {
            builder.edition(edition);
        }

        if let Some(token) = self.token {
            builder.token(token);
        }

        let utility_ix = builder
            .build(LockArgs::V1 {
                authorization_data: None,
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[utility_ix],
            Some(&payer.pubkey()),
            &[&delegate, &payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn unlock(
        &mut self,
        context: &mut ProgramTestContext,
        delegate: Keypair,
        token_record: Option<Pubkey>,
        payer: Keypair,
    ) -> Result<(), BanksClientError> {
        let mut builder = UnlockBuilder::new();
        builder
            .authority(delegate.pubkey())
            .mint(self.mint.pubkey())
            .metadata(self.metadata)
            .payer(payer.pubkey())
            .spl_token_program(spl_token::ID);

        if let Some(token_record) = token_record {
            builder.token_record(token_record);
        }

        if let Some(edition) = self.edition {
            builder.edition(edition);
        }

        if let Some(token) = self.token {
            builder.token(token);
        }

        let unlock_ix = builder
            .build(UnlockArgs::V1 {
                authorization_data: None,
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[unlock_ix],
            Some(&payer.pubkey()),
            &[&delegate, &payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn update(
        &self,
        context: &mut ProgramTestContext,
        authority: Keypair,
        update_args: UpdateArgs,
    ) -> Result<(), BanksClientError> {
        let mut builder = UpdateBuilder::new();
        builder
            .authority(authority.pubkey())
            .metadata(self.metadata)
            .payer(authority.pubkey())
            .mint(self.mint.pubkey());

        if let Some(master_edition) = self.edition {
            builder.edition(master_edition);
        }

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
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

    pub async fn get_asset_data(&self, context: &mut ProgramTestContext) -> AssetData {
        let metadata = self.get_metadata(context).await;

        metadata.into_asset_data()
    }

    pub async fn compare_asset_data(
        &self,
        context: &mut ProgramTestContext,
        asset_data: &AssetData,
    ) {
        let on_chain_asset_data = self.get_asset_data(context).await;

        assert_eq!(on_chain_asset_data, *asset_data);
    }

    pub async fn get_token_delegate_role(
        &self,
        context: &mut ProgramTestContext,
        token: &Pubkey,
    ) -> Option<TokenDelegateRole> {
        let (delegate_record_pubkey, _) = find_token_record_account(&self.mint.pubkey(), token);
        let delegate_record_account = context
            .banks_client
            .get_account(delegate_record_pubkey)
            .await
            .unwrap();

        if let Some(account) = delegate_record_account {
            let delegate_record = TokenRecord::safe_deserialize(&account.data).unwrap();
            delegate_record.delegate_role
        } else {
            None
        }
    }

    pub async fn get_master_edition(&self, context: &mut ProgramTestContext) -> MasterEditionV2 {
        let master_edition_account = context
            .banks_client
            .get_account(self.edition.unwrap())
            .await
            .unwrap()
            .unwrap();

        MasterEditionV2::safe_deserialize(&master_edition_account.data).unwrap()
    }

    pub async fn is_pnft(&self, context: &mut ProgramTestContext) -> bool {
        let md = self.get_metadata(context).await;
        if let Some(standard) = md.token_standard {
            if standard == TokenStandard::ProgrammableNonFungible {
                return true;
            }
        }

        false
    }

    pub async fn inject_close_authority(
        &self,
        context: &mut ProgramTestContext,
        close_authority: &Pubkey,
    ) {
        // To simulate the state where the close authority is set delegate instead of
        // the asset's master edition account, we need to inject modified token account state.
        let mut token_account = get_account(context, &self.token.unwrap()).await;
        let mut token = Account::unpack(&token_account.data).unwrap();

        token.close_authority = COption::Some(*close_authority);
        let mut data = vec![0u8; Account::LEN];
        Account::pack(token, &mut data).unwrap();
        token_account.data = data;

        let token_account_shared_data: AccountSharedData = token_account.into();
        context.set_account(&self.token.unwrap(), &token_account_shared_data);
    }

    pub async fn assert_creators_matches_on_chain(
        &self,
        context: &mut ProgramTestContext,
        creators: &Option<Vec<Creator>>,
    ) {
        let metadata = self.get_metadata(context).await;
        let on_chain_creators = metadata.data.creators;
        assert_eq!(on_chain_creators, *creators);
    }

    pub async fn assert_item_collection_matches_on_chain(
        &self,
        context: &mut ProgramTestContext,
        collection: &Option<Collection>,
    ) {
        let metadata = self.get_metadata(context).await;
        let on_chain_collection = metadata.collection;
        assert_eq!(on_chain_collection, *collection);
    }

    pub async fn assert_collection_details_matches_on_chain(
        &self,
        context: &mut ProgramTestContext,
        collection_details: &Option<CollectionDetails>,
    ) {
        let metadata = self.get_metadata(context).await;
        let on_chain_collection_details = metadata.collection_details;
        assert_eq!(on_chain_collection_details, *collection_details);
    }

    pub async fn assert_burned(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<(), BanksClientError> {
        match self.token_standard.unwrap() {
            TokenStandard::NonFungible => {
                self.non_fungigble_accounts_closed(context).await?;
            }
            TokenStandard::ProgrammableNonFungible => {
                self.programmable_non_fungigble_accounts_closed(context)
                    .await?;
            }
            _ => unimplemented!(),
        }

        Ok(())
    }

    async fn non_fungigble_accounts_closed(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<(), BanksClientError> {
        // Metadata, Master Edition and token account are burned.
        let md_account = context.banks_client.get_account(self.metadata).await?;
        let token_account = context
            .banks_client
            .get_account(self.token.unwrap())
            .await?;
        let edition_account = context
            .banks_client
            .get_account(self.edition.unwrap())
            .await?;

        // Token Metadata accounts may still be open because they are no longer being re-assigned
        // to the system program immediately, but if they exist they should have a
        // data length of 1 (just the disciriminator byte, set to Uninitialized).

        if let Some(account) = md_account {
            assert_eq!(account.data.len(), 1);
        }

        assert!(edition_account.is_none());
        assert!(token_account.is_none());

        Ok(())
    }

    async fn programmable_non_fungigble_accounts_closed(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<(), BanksClientError> {
        self.non_fungigble_accounts_closed(context).await?;

        // Token record is burned.
        let token_record_account = context
            .banks_client
            .get_account(self.token_record.unwrap())
            .await?;

        assert!(token_record_account.is_none());

        Ok(())
    }

    pub async fn change_update_authority(
        &self,
        context: &mut ProgramTestContext,
        new_update_authority: Pubkey,
    ) -> Result<(), BanksClientError> {
        airdrop(context, &new_update_authority, 1_000_000_000)
            .await
            .unwrap();

        let mut builder = UpdateBuilder::new();
        builder
            .authority(context.payer.pubkey())
            .metadata(self.metadata)
            .payer(context.payer.pubkey())
            .mint(self.mint.pubkey());

        if let Some(master_edition) = self.edition {
            builder.edition(master_edition);
        }

        let update_args = UpdateArgs::V1 {
            new_update_authority: Some(new_update_authority),
            data: None,
            primary_sale_happened: None,
            is_mutable: None,
            collection: CollectionToggle::None,
            collection_details: CollectionDetailsToggle::None,
            uses: UsesToggle::None,
            rule_set: RuleSetToggle::None,
            authorization_data: None,
        };

        let update_ix = builder.build(update_args).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn assert_token_record_closed(
        &self,
        context: &mut ProgramTestContext,
        token: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let (token_record_pubkey, _) = find_token_record_account(&self.mint.pubkey(), token);

        let token_record_account = context
            .banks_client
            .get_account(token_record_pubkey)
            .await?;

        if let Some(account) = token_record_account {
            assert_eq!(account.data.len(), 0);
        }
        Ok(())
    }

    pub async fn assert_create_fees_charged(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<(), BanksClientError> {
        let account = get_account(context, &self.metadata).await;

        let rent = context.banks_client.get_rent().await.unwrap();
        let rent_exempt = rent.minimum_balance(account.data.len());

        let expected_lamports = rent_exempt + CREATE_FEE;

        assert_eq!(account.lamports, expected_lamports);
        assert_eq!(account.data[METADATA_FEE_FLAG_INDEX], FEE_FLAG_SET);

        Ok(())
    }

    pub async fn assert_fee_flag_set(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<(), BanksClientError> {
        let account = get_account(context, &self.metadata).await;

        assert_eq!(account.data[METADATA_FEE_FLAG_INDEX], FEE_FLAG_SET);

        Ok(())
    }
}

pub struct TransferParams<'a> {
    pub context: &'a mut ProgramTestContext,
    pub authority: &'a Keypair,
    pub source_owner: &'a Pubkey,
    pub destination_owner: Pubkey,
    pub destination_token: Option<Pubkey>,
    pub payer: &'a Keypair,
    pub authorization_rules: Option<Pubkey>,
    pub args: TransferArgs,
}
