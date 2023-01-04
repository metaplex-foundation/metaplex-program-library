use mpl_token_metadata::{
    id,
    instruction::{
        builders::{
            CreateBuilder, DelegateBuilder, MigrateBuilder, MintBuilder, RevokeBuilder,
            TransferBuilder,
        },
        CreateArgs, DelegateArgs, DelegateRole, InstructionBuilder, MigrateArgs, MintArgs,
        RevokeArgs, TransferArgs,
    },
    pda::find_delegate_account,
    processor::AuthorizationData,
    state::{
        AssetData, Creator, Metadata, ProgrammableConfig, TokenMetadataAccount, TokenStandard,
        EDITION, PREFIX,
    },
};
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

pub const DEFAULT_NAME: &str = "Digital Asset";
pub const DEFAULT_SYMBOL: &str = "DA";
pub const DEFAULT_URI: &str = "https://digital.asset.org";

pub struct DigitalAsset {
    pub metadata: Pubkey,
    pub mint: Keypair,
    pub token: Option<Pubkey>,
    pub master_edition: Option<Pubkey>,
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
        }
    }

    pub async fn create(
        &mut self,
        context: &mut ProgramTestContext,
        token_standard: TokenStandard,
        authorization_rules: Option<Pubkey>,
    ) -> Result<(), BanksClientError> {
        println!("create: authorization rules: {authorization_rules:?}");
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

        let payer_pubkey = context.payer.pubkey();
        let mint_pubkey = self.mint.pubkey();

        let program_id = id();
        let mut builder = CreateBuilder::new();
        builder
            .metadata(self.metadata)
            .mint(self.mint.pubkey())
            .mint_authority(payer_pubkey)
            .payer(payer_pubkey)
            .update_authority(payer_pubkey)
            .initialize_mint(true)
            .update_authority_as_signer(true);

        if let Some(authorization_rules) = authorization_rules {
            asset.programmable_config = Some(ProgrammableConfig {
                rule_set: authorization_rules,
            });
        }

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

        let tx = Transaction::new_signed_with_payer(
            &[create_ix],
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

        let mut builder = MintBuilder::new();
        builder
            .token(token)
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

        let tx = Transaction::new_signed_with_payer(
            &[mint_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        match context.banks_client.process_transaction(tx).await {
            Ok(_) => {
                self.token = Some(token);
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
        println!("create and mint: authorization rules: {authorization_rules:?}");

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
        authority: Keypair,
        delegate: Pubkey,
        delegate_role: DelegateRole,
        amount_opt: Option<u64>,
    ) -> Result<(), BanksClientError> {
        // delegate PDA
        let (delegate_record, _) = find_delegate_account(
            &self.mint.pubkey(),
            delegate_role.clone(),
            &delegate,
            &authority.pubkey(),
        );

        let args = match delegate_role {
            DelegateRole::Transfer => DelegateArgs::TransferV1 {
                amount: amount_opt.unwrap(),
            },
            DelegateRole::Collection => DelegateArgs::CollectionV1,
            DelegateRole::Sale => DelegateArgs::SaleV1 {
                amount: amount_opt.unwrap(),
            },
            _ => panic!("currently unsupported delegate role"),
        };

        let mut builder = DelegateBuilder::new();
        builder
            .delegate(delegate)
            .delegate_record(delegate_record)
            .mint(self.mint.pubkey())
            .metadata(self.metadata)
            .payer(authority.pubkey())
            .authority(authority.pubkey());

        if let Some(edition) = self.master_edition {
            builder.master_edition(edition);
        }

        if let Some(token) = self.token {
            builder.token(token);
        }

        let delegate_ix = builder.build(args.clone()).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&authority.pubkey()),
            &[&authority],
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
        authority: Keypair,
        delegate: Pubkey,
        delegate_role: DelegateRole,
    ) -> Result<(), BanksClientError> {
        // delegate PDA
        let (delegate_record, _) = find_delegate_account(
            &self.mint.pubkey(),
            delegate_role.clone(),
            &delegate,
            &authority.pubkey(),
        );

        let args = match delegate_role {
            DelegateRole::Transfer => RevokeArgs::TransferV1,
            DelegateRole::Collection => RevokeArgs::CollectionV1,
            DelegateRole::Sale => RevokeArgs::SaleV1,
            _ => panic!("currently unsupported delegate role"),
        };

        let mut builder = RevokeBuilder::new();
        builder
            .delegate(delegate)
            .delegate_record(delegate_record)
            .mint(self.mint.pubkey())
            .metadata(self.metadata)
            .payer(authority.pubkey())
            .authority(authority.pubkey());

        if let Some(edition) = self.master_edition {
            builder.master_edition(edition);
        }

        if let Some(token) = self.token {
            builder.token(token);
        }

        let revoke_ix = builder.build(args.clone()).unwrap().instruction();

        let tx = Transaction::new_signed_with_payer(
            &[revoke_ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn transfer(&self, params: TransferParams<'_>) -> Result<(), BanksClientError> {
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

        let mut instructions = vec![];

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
            .payer(*payer)
            .mint(self.mint.pubkey());

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
            &[authority],
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

        let metadata = Metadata::safe_deserialize(&metadata_account.data).unwrap();

        metadata
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
}

pub struct TransferParams<'a> {
    pub context: &'a mut ProgramTestContext,
    pub authority: &'a Keypair,
    pub source_owner: &'a Pubkey,
    pub destination_owner: Pubkey,
    pub destination_token: Option<Pubkey>,
    pub payer: &'a Pubkey,
    pub authorization_rules: Option<Pubkey>,
    pub args: TransferArgs,
}
