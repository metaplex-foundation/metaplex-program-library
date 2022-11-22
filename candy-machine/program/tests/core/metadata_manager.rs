use mpl_token_metadata::{
    instruction,
    state::{Collection, CollectionDetails, Creator, Metadata, Uses, PREFIX},
};
use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair, transaction::Transaction,
};

use crate::core::{
    helpers::{clone_keypair, create_mint, get_account, mint_to_wallets, update_blockhash},
    MasterEditionManager,
};

#[derive(Debug)]
pub struct MetadataManager {
    pub authority: Keypair,
    pub mint: Keypair,
    pub pubkey: Pubkey,
    pub owner: Keypair,
}

impl Clone for MetadataManager {
    fn clone(&self) -> Self {
        Self {
            authority: clone_keypair(&self.authority),
            mint: clone_keypair(&self.mint),
            pubkey: self.pubkey,
            owner: clone_keypair(&self.owner),
        }
    }
}

impl From<MasterEditionManager> for MetadataManager {
    fn from(master: MasterEditionManager) -> Self {
        Self {
            authority: master.authority,
            owner: master.owner,
            mint: master.mint,
            pubkey: master.metadata_pubkey,
        }
    }
}

impl MetadataManager {
    pub fn new(authority: &Keypair) -> Self {
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();
        let program_id = mpl_token_metadata::id();

        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (pubkey, _) = Pubkey::find_program_address(metadata_seeds, &program_id);

        Self {
            authority: clone_keypair(authority),
            mint: clone_keypair(&mint),
            pubkey,
            owner: clone_keypair(authority),
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Metadata {
        let account = get_account(context, &self.pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    pub async fn get_data_from_account(
        context: &mut ProgramTestContext,
        pubkey: &Pubkey,
    ) -> Metadata {
        let account = get_account(context, pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    pub async fn create_v3(
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
        sized: bool,
    ) -> Result<(), BanksClientError> {
        create_mint(
            context,
            &self.authority.pubkey(),
            Some(&self.authority.pubkey()),
            0,
            Some(clone_keypair(&self.mint)),
        )
        .await?;
        mint_to_wallets(
            context,
            &self.mint.pubkey(),
            &self.authority,
            vec![(self.owner.pubkey(), 1)],
        )
        .await?;

        let collection_details = if sized {
            Some(CollectionDetails::V1 { size: 0 })
        } else {
            None
        };

        update_blockhash(context).await?;
        let tx = Transaction::new_signed_with_payer(
            &[instruction::create_metadata_accounts_v3(
                mpl_token_metadata::id(),
                self.pubkey,
                self.mint.pubkey(),
                self.authority.pubkey(),
                self.authority.pubkey(),
                self.authority.pubkey(),
                name,
                symbol,
                uri,
                creators,
                seller_fee_basis_points,
                false,
                is_mutable,
                collection,
                uses,
                collection_details,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await
    }

    pub fn get_ata(&self) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(
            &self.owner.pubkey(),
            &self.mint.pubkey(),
        )
    }
}

impl Default for MetadataManager {
    fn default() -> Self {
        Self::new(&Keypair::new())
    }
}
