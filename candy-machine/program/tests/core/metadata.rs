use mpl_token_metadata::{
    instruction,
    state::{Collection, Creator, Uses, PREFIX},
};
use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair, transaction::Transaction,
    transport,
};

use crate::core::helpers::{clone_keypair, create_mint, get_account, mint_to_wallets};

#[derive(Debug)]
pub struct Metadata {
    pub authority: Keypair,
    pub mint: Keypair,
    pub pubkey: Pubkey,
    pub token: Pubkey,
}

impl Metadata {
    pub fn new(authority: &Keypair) -> Self {
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();
        let program_id = mpl_token_metadata::id();

        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (pubkey, _) = Pubkey::find_program_address(metadata_seeds, &program_id);

        Metadata {
            authority: clone_keypair(authority),
            mint: clone_keypair(&mint),
            pubkey,
            token: spl_associated_token_account::get_associated_token_address(
                &authority.pubkey(),
                &mint.pubkey(),
            ),
        }
    }

    pub async fn _get_data(
        &self,
        context: &mut ProgramTestContext,
    ) -> mpl_token_metadata::state::Metadata {
        let account = get_account(context, &self.pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    pub async fn get_data_from_account(
        context: &mut ProgramTestContext,
        pubkey: &Pubkey,
    ) -> mpl_token_metadata::state::Metadata {
        let account = get_account(context, pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
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
        freeze_authority: Option<&Pubkey>,
        collection: Option<Collection>,
        uses: Option<Uses>,
    ) -> transport::Result<()> {
        create_mint(
            context,
            &self.authority.pubkey(),
            freeze_authority,
            0,
            Some(clone_keypair(&self.mint)),
        )
        .await?;
        mint_to_wallets(
            context,
            &self.mint.pubkey(),
            &self.authority,
            vec![(self.authority.pubkey(), 1)],
        )
        .await?;

        let tx = Transaction::new_signed_with_payer(
            &[instruction::create_metadata_accounts_v2(
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
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new(&Keypair::new())
    }
}
