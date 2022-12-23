use mpl_token_metadata::{
    id,
    instruction::{self, DelegateArgs, DelegateRole, MintArgs, TransferArgs},
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

        if let Some(authorization_rules) = authorization_rules {
            asset.programmable_config = Some(ProgrammableConfig {
                rule_set: authorization_rules,
            });
        }

        let payer_pubkey = context.payer.pubkey();
        let mint_pubkey = self.mint.pubkey();

        let program_id = id();

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
                Some(master_edition)
            }
            _ => None,
        };

        let create_ix = instruction::create(
            /* metadata account */ self.metadata,
            /* master edition   */ master_edition,
            /* mint account     */ self.mint.pubkey(),
            /* mint authority   */ payer_pubkey,
            /* payer            */ payer_pubkey,
            /* update authority */ payer_pubkey,
            /* initialize mint  */ true,
            /* authority signer */ true,
            /* asset data       */ asset,
            /* decimals         */ Some(0),
            /* max supply       */ Some(0),
        );

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

        let mint_ix = instruction::mint(
            /* token account       */ token,
            /* metadata account    */ self.metadata,
            /* mint account        */ self.mint.pubkey(),
            /* payer               */ payer_pubkey,
            /* authority           */ payer_pubkey,
            /* master edition      */ self.master_edition,
            /* authorization rules */ authorization_rules,
            /* amount              */
            MintArgs::V1 {
                amount,
                authorization_data,
            },
        );

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
            _ => panic!("currently unsupported delegate role"),
        };

        let delegate_ix = instruction::delegate(
            /* delegate record       */ delegate_record,
            /* delegate              */ delegate,
            /* mint                  */ self.mint.pubkey(),
            /* metadata              */ self.metadata,
            /* master_edition        */ self.master_edition,
            /* authority             */ authority.pubkey(),
            /* payer                 */ authority.pubkey(),
            /* token                 */ self.token,
            /* authorization payload */ None,
            /* additional accounts   */ None,
            /* delegate args         */ args,
        );

        let tx = Transaction::new_signed_with_payer(
            &[delegate_ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn transfer(
        &mut self,
        context: &mut ProgramTestContext,
        owner: Keypair,
        destination: Pubkey,
        destination_token: Option<Pubkey>,
        authorization_rules: Option<Pubkey>,
        authorization_data: Option<AuthorizationData>,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let mut instructions = vec![];

        let destination_token = if let Some(destination_token) = destination_token {
            destination_token
        } else {
            instructions.push(create_associated_token_account(
                &owner.pubkey(),
                &destination,
                &self.mint.pubkey(),
                &spl_token::id(),
            ));

            get_associated_token_address(&destination, &self.mint.pubkey())
        };

        instructions.push(instruction::transfer(
            id(),
            owner.pubkey(),
            self.token.unwrap(),
            self.metadata,
            self.mint.pubkey(),
            self.master_edition,
            destination,
            destination_token,
            TransferArgs::V1 {
                authorization_data,
                amount,
            },
            authorization_rules,
            None,
        ));

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&owner.pubkey()),
            &[&owner],
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
