use std::fmt::Display;

use mpl_bubblegum::state::metaplex_adapter::{Creator, MetadataArgs, TokenProgramVersion};
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClient, ProgramTestContext};
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

use super::{clone_keypair, program_test, tree::Tree, Error, LeafArgs, Result};

pub struct BubblegumTestContext {
    program_context: ProgramTestContext,
    pub default_creators: Vec<Keypair>,
}

pub const DEFAULT_LAMPORTS_FUND_AMOUNT: u64 = 1_000_000_000;

impl BubblegumTestContext {
    pub fn test_context(&self) -> &ProgramTestContext {
        &self.program_context
    }

    pub async fn new() -> Result<Self> {
        let program_context = program_test().start_with_context().await;

        let mut ctx = BubblegumTestContext {
            program_context,
            default_creators: Vec::new(),
        };

        let default_creators = vec![
            Keypair::new(),
            Keypair::new(),
            Keypair::new(),
            Keypair::new(),
        ];

        for creator in default_creators.iter() {
            ctx.fund_account(creator.pubkey(), DEFAULT_LAMPORTS_FUND_AMOUNT)
                .await?;
        }

        ctx.default_creators = default_creators;

        Ok(ctx)
    }

    pub fn client(&self) -> BanksClient {
        self.program_context.banks_client.clone()
    }

    // TODO: implement this based on stuff from `mpl-testing-utils` after we can add it
    // as a dev-dependency without conflicts/issues.
    pub async fn fund_account(&mut self, address: Pubkey, lamports: u64) -> Result<()> {
        let payer = &self.program_context.payer;

        // Create a transaction to send some funds to the `new_owner` account, which is used
        // as a payer in one of the operations below. Having the payer be an account with no
        // funds causes the Banks server to hang. Will find a better way to implement this
        // op.
        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::transfer(
                &payer.pubkey(),
                &address,
                lamports,
            )],
            Some(&payer.pubkey()),
            &[payer],
            self.program_context.last_blockhash,
        );

        self.program_context
            .banks_client
            .process_transaction(tx)
            .await
            .map_err(Error::BanksClient)
    }

    pub fn payer(&self) -> Keypair {
        clone_keypair(&self.program_context.payer)
    }

    pub fn default_metadata_args<T, U>(&self, name: T, symbol: U) -> MetadataArgs
    where
        T: Display,
        U: Display,
    {
        MetadataArgs {
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: "www.solana.pos".to_owned(),
            seller_fee_basis_points: 0,
            primary_sale_happened: false,
            is_mutable: false,
            edition_nonce: None,
            token_standard: None,
            token_program_version: TokenProgramVersion::Original,
            collection: None,
            uses: None,
            creators: vec![
                Creator {
                    address: self.default_creators[0].pubkey(),
                    verified: false,
                    share: 20,
                },
                Creator {
                    address: self.default_creators[1].pubkey(),
                    verified: false,
                    share: 20,
                },
                Creator {
                    address: self.default_creators[2].pubkey(),
                    verified: false,
                    share: 20,
                },
                Creator {
                    address: self.default_creators[3].pubkey(),
                    verified: false,
                    share: 40,
                },
            ],
        }
    }

    pub async fn default_create_tree<const MAX_DEPTH: usize, const MAX_BUFFER_SIZE: usize>(
        &self,
    ) -> Result<Tree<MAX_DEPTH, MAX_BUFFER_SIZE>> {
        let payer = self.payer();
        let tree = Tree::<MAX_DEPTH, MAX_BUFFER_SIZE>::with_creator(&payer, self.client());
        tree.alloc(&payer).await?;
        tree.create(&payer).await?;
        Ok(tree)
    }

    pub async fn create_public_tree<const MAX_DEPTH: usize, const MAX_BUFFER_SIZE: usize>(
        &self,
    ) -> Result<Tree<MAX_DEPTH, MAX_BUFFER_SIZE>> {
        let payer = self.payer();
        let tree = Tree::<MAX_DEPTH, MAX_BUFFER_SIZE>::with_creator(&payer, self.client());
        tree.alloc(&payer).await?;
        tree.create_public(&payer).await?;
        Ok(tree)
    }

    // The owner of the tree and leaves is `self.payer()`.
    pub async fn default_create_and_mint<const MAX_DEPTH: usize, const MAX_BUFFER_SIZE: usize>(
        &self,
        num_mints: u64,
    ) -> Result<(Tree<MAX_DEPTH, MAX_BUFFER_SIZE>, Vec<LeafArgs>)> {
        let tree = self
            .default_create_tree::<MAX_DEPTH, MAX_BUFFER_SIZE>()
            .await?;

        let payer = self.payer();

        let mut leaves = Vec::new();

        for i in 0..num_mints {
            let name = format!("test{}", i);
            let symbol = format!("tst{}", i);
            let mut args = LeafArgs::new(&payer, self.default_metadata_args(name, symbol));

            tree.mint_v1(&payer, &mut args).await?;
            assert_eq!(args.index, u32::try_from(i).unwrap());
            assert_eq!(args.nonce, i);

            leaves.push(args);
        }

        Ok((tree, leaves))
    }
}
