use crate::{solana::create_associated_token_account, utils::*};
use mpl_token_metadata::{
    id, instruction,
    instruction::{builders::*, CreateArgs, InstructionBuilder, MintArgs},
    pda::{find_master_edition_account, find_metadata_account, find_token_record_account},
    processor::AuthorizationData,
    state::{
        AssetData, Collection, CollectionDetails, Creator, DataV2, PrintSupply, TokenStandard, Uses,
    },
};
use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

#[derive(Debug)]
pub struct Metadata {
    pub mint: Keypair,
    pub token: Keypair,
    pub ata: Pubkey,
    pub pubkey: Pubkey,
    pub master_edition: Pubkey,
    pub token_record: Pubkey,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Metadata {
    pub fn new() -> Self {
        let mint = Keypair::new();
        // for builder functions, this is the token owner
        let token = Keypair::new();

        let (pubkey, _) = find_metadata_account(&mint.pubkey());
        let (master_edition, _) = find_master_edition_account(&mint.pubkey());
        let ata = get_associated_token_address(&token.pubkey(), &mint.pubkey());
        let (token_record, _) = find_token_record_account(&mint.pubkey(), &ata);

        Self {
            mint,
            pubkey,
            token,
            ata,
            token_record,
            master_edition,
        }
    }

    pub async fn get_data(
        &self,
        context: &mut ProgramTestContext,
    ) -> mpl_token_metadata::state::Metadata {
        let account = get_account(context, &self.pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    pub async fn create(
        &self,
        context: &mut ProgramTestContext,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        is_mutable: bool,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        create_mint(context, &self.mint, &context.payer.pubkey(), None).await?;

        let token = create_associated_token_account(context, &self.token, &self.mint.pubkey())
            .await
            .unwrap();
        mint_tokens(
            context,
            &self.mint.pubkey(),
            &token,
            amount,
            &context.payer.pubkey(),
            None,
        )
        .await?;

        let tx = Transaction::new_signed_with_payer(
            &[instruction::create_metadata_accounts_v3(
                id(),
                self.pubkey,
                self.mint.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                name,
                symbol,
                uri,
                creators,
                seller_fee_basis_points,
                false,
                is_mutable,
                None,
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn mint_via_builder(
        &self,
        context: &mut ProgramTestContext,
        amount: u64,
        authorization_data: Option<AuthorizationData>,
    ) -> Result<(), BanksClientError> {
        let ix = MintBuilder::new()
            .token(self.ata)
            .token_owner(self.token.pubkey())
            .metadata(self.pubkey)
            .master_edition(self.master_edition)
            .token_record(self.token_record)
            .mint(self.mint.pubkey())
            .authority(context.payer.pubkey())
            .payer(context.payer.pubkey())
            .system_program(solana_sdk::system_program::ID)
            .sysvar_instructions(solana_sdk::sysvar::instructions::ID)
            .spl_token_program(spl_token::id())
            .spl_ata_program(spl_associated_token_account::id())
            .build(MintArgs::V1 {
                amount,
                authorization_data,
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn create_via_builder(
        &self,
        context: &mut ProgramTestContext,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        is_mutable: bool,
        collection: Option<Collection>,
        uses: Option<Uses>,
        primary_sale_happened: bool,
        token_standard: TokenStandard,
        collection_details: Option<CollectionDetails>,
        rule_set: Option<Pubkey>,
        decimals: Option<u8>,
        print_supply: Option<PrintSupply>,
    ) -> Result<(), BanksClientError> {
        let ix = CreateBuilder::new()
            .metadata(self.pubkey)
            .master_edition(self.master_edition)
            .mint(self.mint.pubkey())
            .authority(context.payer.pubkey())
            .payer(context.payer.pubkey())
            .update_authority(context.payer.pubkey())
            .system_program(solana_sdk::system_program::ID)
            .sysvar_instructions(solana_sdk::sysvar::instructions::ID)
            .spl_token_program(spl_token::id())
            .initialize_mint(true)
            .update_authority_as_signer(true)
            .build(CreateArgs::V1 {
                asset_data: AssetData {
                    primary_sale_happened,
                    token_standard,
                    symbol,
                    name,
                    uri,
                    seller_fee_basis_points,
                    creators,
                    is_mutable,
                    collection,
                    uses,
                    collection_details,
                    rule_set,
                },
                decimals,
                print_supply,
            })
            .unwrap()
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.mint],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn create_v2(
        &self,
        context: &mut ProgramTestContext,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        is_mutable: bool,
        collection: Option<Collection>,
        uses: Option<Uses>,
    ) -> Result<(), BanksClientError> {
        create_mint(context, &self.mint, &context.payer.pubkey(), None).await?;
        create_token_account(
            context,
            &self.token,
            &self.mint.pubkey(),
            &context.payer.pubkey(),
        )
        .await?;
        mint_tokens(
            context,
            &self.mint.pubkey(),
            &self.token.pubkey(),
            1,
            &context.payer.pubkey(),
            None,
        )
        .await?;

        let tx = Transaction::new_signed_with_payer(
            &[instruction::create_metadata_accounts_v3(
                id(),
                self.pubkey,
                self.mint.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                name,
                symbol,
                uri,
                creators,
                seller_fee_basis_points,
                false,
                is_mutable,
                collection,
                uses,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn update_primary_sale_happened_via_token(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_primary_sale_happened_via_token(
                id(),
                self.pubkey,
                context.payer.pubkey(),
                self.token.pubkey(),
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn update(
        &self,
        context: &mut ProgramTestContext,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                id(),
                self.pubkey,
                context.payer.pubkey(),
                None,
                Some(DataV2 {
                    name,
                    symbol,
                    uri,
                    creators,
                    seller_fee_basis_points,
                    collection: None,
                    uses: None,
                }),
                None,
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn update_v2(
        &self,
        context: &mut ProgramTestContext,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        is_mutable: bool,
        collection: Option<Collection>,
        uses: Option<Uses>,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts_v2(
                id(),
                self.pubkey,
                context.payer.pubkey(),
                None,
                Some(DataV2 {
                    name,
                    symbol,
                    uri,
                    creators,
                    seller_fee_basis_points,
                    collection,
                    uses,
                }),
                None,
                Some(is_mutable),
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn verify_collection(
        &self,
        context: &mut ProgramTestContext,
        collection: Pubkey,
        collection_authority: Keypair,
        collection_mint: Pubkey,
        collection_master_edition_account: Pubkey,
        collection_authority_record: Option<Pubkey>,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::verify_collection(
                id(),
                self.pubkey,
                collection_authority.pubkey(),
                context.payer.pubkey(),
                collection_mint,
                collection,
                collection_master_edition_account,
                collection_authority_record,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &collection_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn unverify_collection(
        &self,
        context: &mut ProgramTestContext,
        collection: Pubkey,
        collection_authority: Keypair,
        collection_mint: Pubkey,
        collection_master_edition_account: Pubkey,
        collection_authority_record: Option<Pubkey>,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::unverify_collection(
                id(),
                self.pubkey,
                collection_authority.pubkey(),
                collection_mint,
                collection,
                collection_master_edition_account,
                collection_authority_record,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &collection_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
