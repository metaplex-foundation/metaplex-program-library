use mpl_token_metadata::{
    instruction::{self},
    state::{EDITION, PREFIX},
};
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction, transport};

use crate::{
    core::{get_account, metadata::Metadata},
    *,
};

#[derive(Debug)]
pub struct MasterEditionV2 {
    pub pubkey: Pubkey,
    pub metadata_pubkey: Pubkey,
    pub mint_pubkey: Pubkey,
}

impl MasterEditionV2 {
    pub fn new(metadata: &Metadata) -> Self {
        let program_id = mpl_token_metadata::id();
        let mint_pubkey = metadata.mint.pubkey();

        let master_edition_seeds = &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_pubkey.as_ref(),
            EDITION.as_bytes(),
        ];
        let (pubkey, _) =
            Pubkey::find_program_address(master_edition_seeds, &mpl_token_metadata::id());

        MasterEditionV2 {
            pubkey,
            metadata_pubkey: metadata.pubkey,
            mint_pubkey,
        }
    }

    pub async fn get_data(
        &self,
        context: &mut ProgramTestContext,
    ) -> mpl_token_metadata::state::MasterEditionV2 {
        let account = get_account(context, &self.pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    pub async fn get_data_from_account(
        context: &mut ProgramTestContext,
        pubkey: &Pubkey,
    ) -> mpl_token_metadata::state::MasterEditionV2 {
        let account = get_account(context, pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    pub async fn create_v3(
        &self,
        context: &mut ProgramTestContext,
        max_supply: Option<u64>,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::create_master_edition_v3(
                mpl_token_metadata::id(),
                self.pubkey,
                self.mint_pubkey,
                context.payer.pubkey(),
                context.payer.pubkey(),
                self.metadata_pubkey,
                context.payer.pubkey(),
                max_supply,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
