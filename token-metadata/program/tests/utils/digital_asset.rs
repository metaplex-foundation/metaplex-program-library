use mpl_token_metadata::{
    id,
    instruction::{
        builders::{
            CreateBuilder, DelegateBuilder, LockBuilder, MigrateBuilder, MintBuilder,
            RevokeBuilder, TransferBuilder, UnlockBuilder,
        },
        CreateArgs, DelegateArgs, InstructionBuilder, LockArgs, MetadataDelegateRole, MigrateArgs,
        MintArgs, RevokeArgs, TransferArgs, UnlockArgs,
    },
    pda::{find_metadata_delegate_record_account, find_token_record_account},
    processor::AuthorizationData,
    state::{
        AssetData, Creator, Metadata, ProgrammableConfig, TokenDelegateRole, TokenMetadataAccount,
        TokenRecord, TokenStandard, EDITION, PREFIX,
    },
};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use super::get_account;

pub const DEFAULT_NAME: &str = "Digital Asset";
pub const DEFAULT_SYMBOL: &str = "DA";
pub const DEFAULT_URI: &str = "https://digital.asset.org";

pub struct DigitalAsset {
    pub metadata: Pubkey,
    pub mint: Keypair,
    pub token: Option<Pubkey>,
    pub master_edition: Option<Pubkey>,
    pub token_record: Option<Pubkey>,
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
        let program_id = id();

        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (metadata, _) = Pubkey::find_program_address(metadata_seeds, &program_id);

        Self {
            metadata,
            mint,
            token: None,
            master_edition: None,
            token_record: None,
        }
    }

    pub async fn create(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        authorization_rules: Option<Pubkey>,
    ) -> Result<(), BanksClientError> {
        let mut asset = AssetData::new(
            token_standard,
            String::from(DEFAULT_NAME),
            String::from(DEFAULT_SYMBOL),
            String::from(DEFAULT_URI),
            context.payer.pubkey(),
        );
        asset.seller_fee_basis_points = 500;

        let creators = vec![Creator {
            address: context.payer.pubkey(),
            share: 100,
            verified: true,
        }];
        asset.creators = Some(creators);
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

        let master_edition = match token_standard {
            TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible => {
                // master edition PDA address
                let master_edition_seeds = &[
                    PREFIX.as_bytes(),
                    program_id.as_ref(),
                    mint_pubkey.as_ref(),
                    EDITION.as_bytes(),
                ];
                let (master_edition, _) = Pubkey::find_program_address(master_edition_seeds, &id());
                // sets the master edition to the builder
                builder.master_edition(master_edition);
                Some(master_edition)
            }
            _ => None,
        };
        // builds the instruction
        let create_ix = builder
            .build(CreateArgs::V1 {
                asset_data: asset,
                decimals: Some(0),
                max_supply: Some(0),
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

        self.master_edition = master_edition;

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
                &spl_token::id().to_bytes(),
                &self.mint.pubkey().to_bytes(),
            ],
            &spl_associated_token_account::id(),
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

        if let Some(edition) = self.master_edition {
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

        match context.banks_client.process_transaction(tx).await {
            Ok(_) => {
                self.token = Some(token);
                self.token_record = token_record_opt;
                Ok(())
            }
            Err(error) => Err(error),
        }
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

    pub async fn delegate(
        &mut self,
        context: &mut ProgramTestContext,
        payer: Keypair,
        delegate: Pubkey,
        args: DelegateArgs,
    ) -> Result<(), BanksClientError> {
        let mut builder = DelegateBuilder::new();
        builder
            .delegate(delegate)
            .mint(self.mint.pubkey())
            .metadata(self.metadata)
            .payer(payer.pubkey())
            .authority(payer.pubkey());

        match args {
            DelegateArgs::CollectionV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Collection,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            DelegateArgs::SaleV1 { .. }
            | DelegateArgs::TransferV1 { .. }
            | DelegateArgs::UtilityV1 { .. }
            | DelegateArgs::StakingV1 { .. } => {
                let (token_record, _) =
                    find_token_record_account(&self.mint.pubkey(), &self.token.unwrap());
                builder.token_record(token_record);
            }
            DelegateArgs::UpdateV1 { .. } => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Update,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            DelegateArgs::StandardV1 { .. } => { /* nothing to add */ }
        }

        if let Some(edition) = self.master_edition {
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
        }

        let delegate_ix = builder.build(args.clone()).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&payer.pubkey()),
            &[&payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn migrate(
        &mut self,
        context: &mut ProgramTestContext,
        authority: Keypair,
        collection_metadata: Pubkey,
        args: MigrateArgs,
    ) -> Result<(), BanksClientError> {
        let mut builder = MigrateBuilder::new();
        builder
            .mint(self.mint.pubkey())
            .metadata(self.metadata)
            .edition(self.master_edition.unwrap())
            .token(self.token.unwrap())
            .payer(authority.pubkey())
            .collection_metadata(collection_metadata)
            .authority(authority.pubkey());

        let migrate_ix = builder.build(args.clone()).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[migrate_ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
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
            .authority(approver.pubkey());

        match args {
            RevokeArgs::CollectionV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Collection,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            RevokeArgs::SaleV1
            | RevokeArgs::TransferV1
            | RevokeArgs::UtilityV1
            | RevokeArgs::StakingV1 => {
                let (token_record, _) =
                    find_token_record_account(&self.mint.pubkey(), &self.token.unwrap());
                builder.token_record(token_record);
            }
            RevokeArgs::UpdateV1 => {
                let (delegate_record, _) = find_metadata_delegate_record_account(
                    &self.mint.pubkey(),
                    MetadataDelegateRole::Update,
                    &payer.pubkey(),
                    &delegate,
                );
                builder.delegate_record(delegate_record);
            }
            RevokeArgs::StandardV1 { .. } => { /* nothing to add */ }
        }

        if let Some(edition) = self.master_edition {
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

    pub async fn transfer_from(
        &self,
        params: TransferFromParams<'_>,
    ) -> Result<(), BanksClientError> {
        let TransferFromParams {
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
                &spl_token::id(),
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
        builder.destination_token_record(destination_token_record);

        if let Some(master_edition) = self.master_edition {
            builder.edition(master_edition);
        }

        if let Some(authorization_rules) = authorization_rules {
            builder.authorization_rules(authorization_rules);
        }

        let transfer_ix = builder.build(args).unwrap().instruction();

        instructions.push(transfer_ix);

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&authority.pubkey()),
            &[authority, payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
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
            .payer(payer.pubkey());

        if let Some(token_record) = token_record {
            builder.token_record(token_record);
        }

        if let Some(edition) = self.master_edition {
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
            .payer(payer.pubkey());

        if let Some(token_record) = token_record {
            builder.token_record(token_record);
        }

        if let Some(edition) = self.master_edition {
            builder.edition(edition);
        }

        if let Some(token) = self.token {
            builder.token(token);
        }

        let utility_ix = builder
            .build(UnlockArgs::V1 {
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

    pub async fn transfer_to(&self, params: TransferToParams<'_>) -> Result<(), BanksClientError> {
        let TransferToParams {
            context,
            authority,
            source_owner,
            source_token,
            destination_owner,
            destination_token,
            authorization_rules,
            payer,
            args,
        } = params;

        // Increase compute budget to handle larger test transactions.
        let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);
        let mut instructions = vec![compute_ix];

        let destination_token = if let Some(destination_token) = destination_token {
            destination_token
        } else {
            instructions.push(create_associated_token_account(
                &authority.pubkey(),
                &destination_owner,
                &self.mint.pubkey(),
                &spl_token::id(),
            ));

            get_associated_token_address(&destination_owner, &self.mint.pubkey())
        };

        let mut builder = TransferBuilder::new();
        builder
            .authority(authority.pubkey())
            .token_owner(*source_owner)
            .token(*source_token)
            .destination_owner(destination_owner)
            .destination(destination_token)
            .metadata(self.metadata)
            .payer(payer.pubkey())
            .mint(self.mint.pubkey());

        // This can be optional for non pNFTs but always include it for now.
        let (owner_token_record, _bump) =
            find_token_record_account(&self.mint.pubkey(), source_token);
        builder.owner_token_record(owner_token_record);

        // This can be optional for non pNFTs but always include it for now.
        let (destination_token_record, _bump) =
            find_token_record_account(&self.mint.pubkey(), &destination_token);
        builder.destination_token_record(destination_token_record);

        if let Some(master_edition) = self.master_edition {
            builder.edition(master_edition);
        }

        if let Some(authorization_rules) = authorization_rules {
            builder.authorization_rules(authorization_rules);
        }

        let transfer_ix = builder.build(args).unwrap().instruction();

        instructions.push(transfer_ix);

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&authority.pubkey()),
            &[authority, payer],
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
        let (delegate_record_pubkey, _) =
            find_token_record_account(&self.mint.pubkey(), token);
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

    pub async fn is_pnft(&self, context: &mut ProgramTestContext) -> bool {
        let md = self.get_metadata(context).await;
        if let Some(standard) = md.token_standard {
            if standard == TokenStandard::ProgrammableNonFungible {
                return true;
            }
        }

        false
    }
}

pub struct TransferFromParams<'a> {
    pub context: &'a mut ProgramTestContext,
    pub authority: &'a Keypair,
    pub source_owner: &'a Pubkey,
    pub destination_owner: Pubkey,
    pub destination_token: Option<Pubkey>,
    pub payer: &'a Keypair,
    pub authorization_rules: Option<Pubkey>,
    pub args: TransferArgs,
}

pub struct TransferToParams<'a> {
    pub context: &'a mut ProgramTestContext,
    pub authority: &'a Keypair,
    pub source_owner: &'a Pubkey,
    pub source_token: &'a Pubkey,
    pub destination_owner: Pubkey,
    pub destination_token: Option<Pubkey>,
    pub payer: &'a Keypair,
    pub authorization_rules: Option<Pubkey>,
    pub args: TransferArgs,
}
