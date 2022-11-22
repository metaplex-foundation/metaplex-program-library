use mpl_token_metadata::{
    instruction::{self},
    state::{MasterEditionV2, Metadata, EDITION, PREFIX},
};
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    core::{
        helpers::{clone_keypair, get_account, update_blockhash},
        metadata_manager::MetadataManager,
    },
    *,
};

#[derive(Debug)]
pub struct MasterEditionManager {
    pub authority: Keypair,
    pub edition_pubkey: Pubkey,
    pub metadata_pubkey: Pubkey,
    pub mint: Keypair,
    pub token_account: Pubkey,
    pub owner: Keypair,
}

impl Clone for MasterEditionManager {
    fn clone(&self) -> Self {
        Self {
            authority: clone_keypair(&self.authority),
            edition_pubkey: self.edition_pubkey,
            metadata_pubkey: self.metadata_pubkey,
            mint: clone_keypair(&self.mint),
            token_account: self.token_account,
            owner: clone_keypair(&self.owner),
        }
    }
}

impl MasterEditionManager {
    pub fn new(metadata: &MetadataManager) -> Self {
        let program_id = mpl_token_metadata::id();
        let mint_pubkey = metadata.mint.pubkey();

        let master_edition_seeds = &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_pubkey.as_ref(),
            EDITION.as_bytes(),
        ];
        let edition_pubkey =
            Pubkey::find_program_address(master_edition_seeds, &mpl_token_metadata::id()).0;

        MasterEditionManager {
            authority: clone_keypair(&metadata.authority),
            edition_pubkey,
            metadata_pubkey: metadata.pubkey,
            mint: clone_keypair(&metadata.mint),
            token_account: get_associated_token_address(
                &metadata.owner.pubkey(),
                &metadata.mint.pubkey(),
            ),
            owner: clone_keypair(&metadata.owner),
        }
    }

    #[allow(dead_code)]
    pub async fn get_data(&self, context: &mut ProgramTestContext) -> MasterEditionV2 {
        let account = get_account(context, &self.edition_pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    pub async fn get_metadata(&self, context: &mut ProgramTestContext) -> Metadata {
        let account = get_account(context, &self.metadata_pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    #[allow(dead_code)]
    pub async fn get_data_from_account(
        context: &mut ProgramTestContext,
        pubkey: &Pubkey,
    ) -> MasterEditionV2 {
        let account = get_account(context, pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    pub async fn create_v3(
        &self,
        context: &mut ProgramTestContext,
        max_supply: Option<u64>,
    ) -> Result<(), BanksClientError> {
        update_blockhash(context).await?;
        let tx = Transaction::new_signed_with_payer(
            &[instruction::create_master_edition_v3(
                mpl_token_metadata::id(),
                self.edition_pubkey,
                self.mint.pubkey(),
                self.authority.pubkey(),
                self.authority.pubkey(),
                self.metadata_pubkey,
                self.authority.pubkey(),
                max_supply,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
