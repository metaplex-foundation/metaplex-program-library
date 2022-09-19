use anchor_lang::{self, AccountDeserialize};
use bytemuck::try_from_bytes;
use mpl_bubblegum::{
    state::{leaf_schema::LeafSchema, TreeConfig},
    utils::get_asset_id,
};
use solana_program::{
    instruction::Instruction, pubkey::Pubkey, rent::Rent, system_instruction, system_program,
};
use solana_program_test::BanksClient;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    signer::signers::Signers,
    transaction::Transaction,
};
use spl_account_compression::state::ConcurrentMerkleTreeHeader;
use spl_concurrent_merkle_tree::concurrent_merkle_tree::ConcurrentMerkleTree;
use spl_merkle_tree_reference::Node;
use std::{
    cell::{RefCell, RefMut},
    convert::TryFrom,
    mem::size_of,
};

use super::{
    clone_keypair, compute_metadata_hashes,
    tx_builder::{
        BurnBuilder, CreateBuilder, DelegateBuilder, MintV1Builder, SetTreeDelegateBuilder,
        TransferBuilder, TxBuilder, UnverifyCreatorBuilder, VerifyCreatorBuilder,
    },
    Error, LeafArgs, Result,
};

// A convenience object that records some of the parameters for compressed
// trees and generates TX builders with the default configuration for each
// operation.
// TODO: finish implementing all operations.
pub struct Tree<const MAX_DEPTH: usize, const MAX_BUFFER_SIZE: usize> {
    pub tree_creator: Keypair,
    pub tree_delegate: Keypair,
    pub merkle_tree: Keypair,
    pub canopy_depth: u32,
    client: RefCell<BanksClient>,
}

impl<const MAX_DEPTH: usize, const MAX_BUFFER_SIZE: usize> Tree<MAX_DEPTH, MAX_BUFFER_SIZE> {
    // This and `with_creator` use a bunch of defaults; things can be
    // customized some more via the public access, or we can add extra
    // methods to make things even easier.
    pub fn new(client: BanksClient) -> Self {
        Self::with_creator(&Keypair::new(), client)
    }

    pub fn with_creator(tree_creator: &Keypair, client: BanksClient) -> Self {
        Tree {
            tree_creator: clone_keypair(tree_creator),
            tree_delegate: clone_keypair(tree_creator),
            merkle_tree: Keypair::new(),
            canopy_depth: 0,
            client: RefCell::new(client),
        }
    }

    pub fn creator_pubkey(&self) -> Pubkey {
        self.tree_creator.pubkey()
    }

    pub fn delegate_pubkey(&self) -> Pubkey {
        self.tree_delegate.pubkey()
    }

    pub fn tree_pubkey(&self) -> Pubkey {
        self.merkle_tree.pubkey()
    }

    pub fn authority(&self) -> Pubkey {
        Pubkey::find_program_address(&[self.tree_pubkey().as_ref()], &mpl_bubblegum::id()).0
    }

    pub fn mint_authority_request(&self, authority: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[self.tree_pubkey().as_ref(), authority.as_ref()],
            &mpl_bubblegum::id(),
        )
        .0
    }

    pub fn merkle_tree_account_size(&self) -> usize {
        size_of::<ConcurrentMerkleTreeHeader>()
            + size_of::<ConcurrentMerkleTree<MAX_DEPTH, MAX_BUFFER_SIZE>>()
    }

    pub fn client(&self) -> RefMut<BanksClient> {
        self.client.borrow_mut()
    }

    // Helper method to execute a transaction with the specified arguments
    // (i.e. single instruction) via the inner Banks client.
    pub async fn process_tx<T: Signers>(
        &self,
        instruction: Instruction,
        payer: &Pubkey,
        signing_keypairs: &T,
    ) -> Result<()> {
        let recent_blockhash = self
            .client()
            .get_latest_blockhash()
            .await
            .map_err(Error::BanksClient)?;

        self.client()
            .process_transaction(Transaction::new_signed_with_payer(
                &[instruction],
                Some(payer),
                signing_keypairs,
                recent_blockhash,
            ))
            .await
            .map_err(Error::BanksClient)
    }

    pub async fn rent(&self) -> Result<Rent> {
        self.client().get_rent().await.map_err(Error::BanksClient)
    }

    // Allocates and pays for an account to hold the tree.
    pub async fn alloc(&self, payer: &Keypair) -> Result<()> {
        let rent = self.rent().await?;
        let account_size = self.merkle_tree_account_size();

        // u64 -> usize conversion should never fail on the platforms we're running on.
        let lamports = rent.minimum_balance(usize::try_from(account_size).unwrap());

        let ix = system_instruction::create_account(
            &payer.pubkey(),
            &self.tree_pubkey(),
            lamports,
            // The `usize -> u64` conversion should never fail.
            u64::try_from(account_size).unwrap(),
            &spl_account_compression::id(),
        );

        self.process_tx(ix, &payer.pubkey(), &[payer, &self.merkle_tree])
            .await
    }

    // Helper fn to instantiate the various `TxBuilder` based concrete types
    // associated with each operation.
    fn tx_builder<T, U, V>(
        &self,
        accounts: T,
        data: U,
        inner: V,
        payer: Pubkey,
        default_signers: &[&Keypair],
    ) -> TxBuilder<T, U, V, MAX_DEPTH, MAX_BUFFER_SIZE> {
        let def_signers = default_signers.iter().map(|k| clone_keypair(k)).collect();

        TxBuilder {
            accounts,
            additional_accounts: Vec::new(),
            data,
            payer,
            client: self.client.clone(),
            signers: def_signers,
            tree: self,
            inner,
        }
    }

    // The `operation_tx` method instantiate a default builder object for a
    // transaction that can be used to execute that particular operation (tree
    // create in this case). The object can be modified (i.e. to use a
    // different signer, payer, accounts, data, etc.) before execution.
    // Moreover executions don't consume the builder, which can be modified
    // some more and executed again etc.
    pub fn create_tree_tx(&self, payer: &Keypair) -> CreateBuilder<MAX_DEPTH, MAX_BUFFER_SIZE> {
        let accounts = mpl_bubblegum::accounts::CreateTree {
            tree_authority: self.authority(),
            payer: payer.pubkey(),
            tree_creator: self.creator_pubkey(),
            log_wrapper: spl_noop::id(),
            system_program: system_program::id(),
            compression_program: spl_account_compression::id(),
            merkle_tree: self.tree_pubkey(),
        };

        // The conversions below should not fail.
        let data = mpl_bubblegum::instruction::CreateTree {
            max_depth: u32::try_from(MAX_DEPTH).unwrap(),
            max_buffer_size: u32::try_from(MAX_BUFFER_SIZE).unwrap(),
        };

        self.tx_builder(accounts, data, (), payer.pubkey(), &[payer])
    }

    // Shorthand method for executing a create tree tx with the default config
    // defined in the `_tx` method.
    pub async fn create(&self, payer: &Keypair) -> Result<()> {
        self.create_tree_tx(payer).execute().await
    }

    pub fn mint_v1_tx(
        &self,
        tree_delegate: &Keypair,
        args: &LeafArgs,
    ) -> MintV1Builder<MAX_DEPTH, MAX_BUFFER_SIZE> {
        let accounts = mpl_bubblegum::accounts::MintV1 {
            tree_authority: self.authority(),
            tree_delegate: tree_delegate.pubkey(),
            payer: args.owner.pubkey(),
            log_wrapper: spl_noop::id(),
            compression_program: spl_account_compression::id(),
            leaf_owner: args.owner.pubkey(),
            leaf_delegate: args.delegate.pubkey(),
            merkle_tree: self.tree_pubkey(),
        };

        let data = mpl_bubblegum::instruction::MintV1 {
            message: args.metadata.clone(),
        };

        self.tx_builder(
            accounts,
            data,
            (),
            args.owner.pubkey(),
            &[tree_delegate, &args.owner],
        )
    }

    // This assumes the owner is the account paying for the tx. We can make things
    // more configurable for any of the methods.
    pub async fn mint_v1(&self, tree_delegate: &Keypair, args: &LeafArgs) -> Result<()> {
        self.mint_v1_tx(tree_delegate, args).execute().await
    }

    pub async fn decode_root(&self) -> Result<[u8; 32]> {
        let mut tree_account = self.read_account(self.tree_pubkey()).await?;

        let merkle_tree_bytes = tree_account.data.as_mut_slice();
        let (_header_bytes, rest) =
            merkle_tree_bytes.split_at_mut(size_of::<ConcurrentMerkleTreeHeader>());

        let merkle_tree_size = size_of::<ConcurrentMerkleTree<MAX_DEPTH, MAX_BUFFER_SIZE>>();
        let tree_bytes = &mut rest[..merkle_tree_size];

        let tree = try_from_bytes::<ConcurrentMerkleTree<MAX_DEPTH, MAX_BUFFER_SIZE>>(tree_bytes)
            .map_err(Error::BytemuckPod)?;
        let root = tree.change_logs[tree.active_index as usize].root;

        Ok(root)
    }

    // This is currently async due to calling `decode_root` (same goes for a bunch of others).
    pub async fn burn_tx(
        &self,
        args: &LeafArgs,
    ) -> Result<BurnBuilder<MAX_DEPTH, MAX_BUFFER_SIZE>> {
        let root = self.decode_root().await?;

        let (data_hash, creator_hash) = compute_metadata_hashes(&args.metadata)?;

        let accounts = mpl_bubblegum::accounts::Burn {
            tree_authority: self.authority(),
            log_wrapper: spl_noop::id(),
            compression_program: spl_account_compression::id(),
            leaf_owner: args.owner.pubkey(),
            leaf_delegate: args.delegate.pubkey(),
            merkle_tree: self.tree_pubkey(),
        };

        let data = mpl_bubblegum::instruction::Burn {
            root,
            data_hash,
            creator_hash,
            nonce: args.nonce,
            index: args.index,
        };

        Ok(self.tx_builder(accounts, data, (), args.owner.pubkey(), &[&args.owner]))
    }

    pub async fn burn(&self, args: &LeafArgs) -> Result<()> {
        self.burn_tx(args).await?.execute().await
    }

    pub async fn verify_creator_tx(
        &self,
        args: &LeafArgs,
        creator: &Keypair,
    ) -> Result<VerifyCreatorBuilder<MAX_DEPTH, MAX_BUFFER_SIZE>> {
        let root = self.decode_root().await?;
        let (data_hash, creator_hash) = compute_metadata_hashes(&args.metadata)?;

        let accounts = mpl_bubblegum::accounts::CreatorVerification {
            tree_authority: self.authority(),
            leaf_owner: args.owner.pubkey(),
            leaf_delegate: args.delegate.pubkey(),
            payer: creator.pubkey(),
            creator: creator.pubkey(),
            log_wrapper: spl_noop::id(),
            compression_program: spl_account_compression::id(),
            merkle_tree: self.tree_pubkey(),
        };

        let data = mpl_bubblegum::instruction::VerifyCreator {
            root,
            data_hash,
            creator_hash,
            nonce: args.nonce,
            index: args.index,
            message: args.metadata.clone(),
        };

        Ok(self.tx_builder(accounts, data, (), creator.pubkey(), &[creator]))
    }

    pub async fn verify_creator(&self, args: &LeafArgs, creator: &Keypair) -> Result<()> {
        self.verify_creator_tx(args, creator).await?.execute().await
    }

    pub async fn unverify_creator_tx(
        &self,
        args: &LeafArgs,
        creator: &Keypair,
    ) -> Result<UnverifyCreatorBuilder<MAX_DEPTH, MAX_BUFFER_SIZE>> {
        let root = self.decode_root().await?;
        let (data_hash, creator_hash) = compute_metadata_hashes(&args.metadata)?;

        let accounts = mpl_bubblegum::accounts::CreatorVerification {
            tree_authority: self.authority(),
            leaf_owner: args.owner.pubkey(),
            leaf_delegate: args.delegate.pubkey(),
            payer: creator.pubkey(),
            creator: creator.pubkey(),
            log_wrapper: spl_noop::id(),
            compression_program: spl_account_compression::id(),
            merkle_tree: self.tree_pubkey(),
        };

        let data = mpl_bubblegum::instruction::UnverifyCreator {
            root,
            data_hash,
            creator_hash,
            nonce: args.nonce,
            index: args.index,
            message: args.metadata.clone(),
        };

        Ok(self.tx_builder(accounts, data, (), creator.pubkey(), &[creator]))
    }

    pub async fn unverify_creator(&self, args: &LeafArgs, creator: &Keypair) -> Result<()> {
        self.unverify_creator_tx(args, creator)
            .await?
            .execute()
            .await
    }

    pub async fn transfer_tx(
        &self,
        args: &LeafArgs,
        new_leaf_owner: Pubkey,
    ) -> Result<TransferBuilder<MAX_DEPTH, MAX_BUFFER_SIZE>> {
        let root = self.decode_root().await?;
        let (data_hash, creator_hash) = compute_metadata_hashes(&args.metadata)?;

        let accounts = mpl_bubblegum::accounts::Transfer {
            tree_authority: self.authority(),
            leaf_owner: args.owner.pubkey(),
            leaf_delegate: args.delegate.pubkey(),
            new_leaf_owner,
            log_wrapper: spl_noop::id(),
            compression_program: spl_account_compression::id(),
            merkle_tree: self.tree_pubkey(),
        };

        let data = mpl_bubblegum::instruction::Transfer {
            root,
            data_hash,
            creator_hash,
            nonce: args.nonce,
            index: args.index,
        };

        Ok(self.tx_builder(accounts, data, (), args.owner.pubkey(), &[&args.owner]))
    }

    pub async fn transfer(&self, args: &LeafArgs, new_owner: Pubkey) -> Result<()> {
        self.transfer_tx(args, new_owner).await?.execute().await
    }

    pub async fn delegate_tx(
        &self,
        args: &LeafArgs,
        new_leaf_delegate: Pubkey,
    ) -> Result<DelegateBuilder<MAX_DEPTH, MAX_BUFFER_SIZE>> {
        let root = self.decode_root().await?;
        let (data_hash, creator_hash) = compute_metadata_hashes(&args.metadata)?;

        let accounts = mpl_bubblegum::accounts::Delegate {
            tree_authority: self.authority(),
            leaf_owner: args.owner.pubkey(),
            previous_leaf_delegate: args.delegate.pubkey(),
            new_leaf_delegate,
            log_wrapper: spl_noop::id(),
            compression_program: spl_account_compression::id(),
            merkle_tree: self.tree_pubkey(),
        };

        let data = mpl_bubblegum::instruction::Delegate {
            root,
            data_hash,
            creator_hash,
            nonce: args.nonce,
            index: args.index,
        };

        Ok(self.tx_builder(accounts, data, (), args.owner.pubkey(), &[&args.owner]))
    }

    // Does the prev delegate need to sign as well?
    pub async fn delegate(&self, args: &LeafArgs, new_delegate: Pubkey) -> Result<()> {
        self.delegate_tx(args, new_delegate).await?.execute().await
    }

    pub fn set_tree_delegate_tx(
        &self,
        new_tree_delegate: Pubkey,
    ) -> SetTreeDelegateBuilder<MAX_DEPTH, MAX_BUFFER_SIZE> {
        let accounts = mpl_bubblegum::accounts::SetTreeDelegate {
            tree_creator: self.creator_pubkey(),
            new_tree_delegate,
            merkle_tree: self.tree_pubkey(),
            tree_authority: self.authority(),
        };

        let data = mpl_bubblegum::instruction::SetTreeDelegate;

        self.tx_builder(
            accounts,
            data,
            (),
            self.creator_pubkey(),
            &[&self.tree_creator],
        )
    }

    pub async fn set_tree_delegate(&mut self, new_delegate: &Keypair) -> Result<()> {
        self.set_tree_delegate_tx(new_delegate.pubkey())
            .execute()
            .await?;
        self.tree_delegate = clone_keypair(new_delegate);
        Ok(())
    }

    // The following methods provide convenience when reading data from accounts.
    async fn read_account(&self, key: Pubkey) -> Result<Account> {
        self.client()
            .get_account(key)
            .await
            .map_err(Error::BanksClient)?
            .ok_or(Error::AccountNotFound(key))
    }

    // This reads the `Account`, but also deserializes the data to return
    // the strongly typed inner contents.
    pub async fn read_account_data<T>(&self, key: Pubkey) -> Result<T>
    where
        T: AccountDeserialize,
    {
        self.read_account(key)
            .await
            .and_then(|acc| T::try_deserialize(&mut acc.data.as_slice()).map_err(Error::Anchor))
    }

    pub async fn read_tree_config(&self) -> Result<TreeConfig> {
        self.read_account_data(self.authority()).await
    }

    pub fn leaf_node(&self, args: &LeafArgs) -> Result<Node> {
        let (data_hash, creator_hash) = compute_metadata_hashes(&args.metadata)?;
        let asset_id = get_asset_id(&self.tree_pubkey(), args.nonce);

        let leaf = LeafSchema::new_v0(
            asset_id,
            args.owner.pubkey(),
            args.delegate.pubkey(),
            args.nonce,
            data_hash,
            creator_hash,
        );

        Ok(leaf.to_node())
    }
}
