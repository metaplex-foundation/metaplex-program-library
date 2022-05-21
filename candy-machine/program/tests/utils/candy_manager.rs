use std::fmt::Debug;

use anchor_lang::AccountDeserialize;
use mpl_token_metadata::pda::find_collection_authority_account;
use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};

use mpl_candy_machine::{constants::BOT_FEE, CandyMachine, CandyMachineData};

use crate::{
    add_all_config_lines, clone_keypair,
    core::{
        assert_acount_empty, clone_pubkey, create_mint, get_balance, get_token_balance,
        master_edition_v2::MasterEditionV2 as MasterEditionManager, metadata, mint_to_wallets,
        prepare_nft,
    },
    get_account,
    helper_transactions::{remove_collection, set_collection},
    utils::{
        find_candy_creator, find_collection_pda, initialize_candy_machine, mint_nft,
        update_candy_machine,
    },
};

#[derive(Debug)]
pub struct CandyManager {
    pub candy_machine: Keypair,
    pub authority: Keypair,
    pub wallet: Pubkey,
    pub minter: Keypair,
    pub collection_info: CollectionInfo,
    pub token_info: TokenInfo,
    pub whitelist_info: WhitelistInfo,
}

impl Clone for CandyManager {
    fn clone(&self) -> Self {
        CandyManager {
            candy_machine: clone_keypair(&self.candy_machine),
            authority: clone_keypair(&self.authority),
            wallet: clone_pubkey(&self.wallet),
            minter: clone_keypair(&self.minter),
            collection_info: self.collection_info.clone(),
            token_info: self.token_info.clone(),
            whitelist_info: self.whitelist_info.clone(),
        }
    }
}

#[derive(Debug)]
pub struct CollectionInfo {
    pub set: bool,
    pub pda: Pubkey,
    pub mint: Keypair,
    pub metadata: Pubkey,
    pub master_edition: Pubkey,
    pub token_account: Pubkey,
    pub authority_record: Pubkey,
}

impl Clone for CollectionInfo {
    fn clone(&self) -> Self {
        CollectionInfo {
            set: self.set,
            pda: clone_pubkey(&self.pda),
            mint: clone_keypair(&self.mint),
            metadata: clone_pubkey(&self.metadata),
            master_edition: clone_pubkey(&self.master_edition),
            token_account: clone_pubkey(&self.token_account),
            authority_record: clone_pubkey(&self.authority_record),
        }
    }
}

impl CollectionInfo {
    pub fn new(
        set: bool,
        pda: Pubkey,
        mint: Keypair,
        metadata: Pubkey,
        master_edition: Pubkey,
        token_account: Pubkey,
        authority_record: Pubkey,
    ) -> Self {
        CollectionInfo {
            set,
            pda,
            mint,
            metadata,
            master_edition,
            token_account,
            authority_record,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        set: bool,
        candy_machine: &Pubkey,
        authority: Keypair,
    ) -> Self {
        let metadata_info = metadata::Metadata::new(&authority);
        metadata_info
            .create_v2(
                context,
                "Collection Name".to_string(),
                "COLLECTION".to_string(),
                "URI".to_string(),
                None,
                0,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let master_edition_info = MasterEditionManager::new(&metadata_info);
        master_edition_info
            .create_v3(context, Some(0))
            .await
            .unwrap();

        let collection_pda = find_collection_pda(candy_machine).0;
        let collection_authority_record =
            find_collection_authority_account(&metadata_info.mint.pubkey(), &collection_pda).0;

        CollectionInfo {
            set,
            pda: collection_pda,
            mint: metadata_info.mint,
            metadata: metadata_info.pubkey,
            master_edition: master_edition_info.pubkey,
            token_account: metadata_info.token,
            authority_record: collection_authority_record,
        }
    }
}

#[derive(Debug)]
pub struct TokenInfo {
    pub set: bool,
    pub mint: Pubkey,
    pub auth_account: Pubkey,
    pub minter_account: Pubkey,
}

impl TokenInfo {
    pub fn new(set: bool, mint: Pubkey, auth_account: Pubkey, minter_account: Pubkey) -> Self {
        TokenInfo {
            set,
            mint,
            auth_account,
            minter_account,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        set: bool,
        authority: (Pubkey, u64),
        minter: (Pubkey, u64),
    ) -> Self {
        let mint = create_mint(context, &authority.0, Some(&authority.0), 0, None)
            .await
            .unwrap();
        let atas = mint_to_wallets(context, &mint.pubkey(), vec![authority, minter])
            .await
            .unwrap();

        TokenInfo {
            set,
            mint: mint.pubkey(),
            auth_account: atas[0],
            minter_account: atas[1],
        }
    }
}

impl Clone for TokenInfo {
    fn clone(&self) -> Self {
        TokenInfo {
            set: self.set,
            mint: clone_pubkey(&self.mint),
            auth_account: clone_pubkey(&self.auth_account),
            minter_account: clone_pubkey(&self.minter_account),
        }
    }
}

#[derive(Debug)]
pub struct WhitelistInfo {
    pub set: bool,
    pub mint: Pubkey,
    pub auth_account: Pubkey,
    pub minter_account: Pubkey,
    pub burn: bool,
}

impl WhitelistInfo {
    pub fn new(
        set: bool,
        mint: Pubkey,
        auth_account: Pubkey,
        minter_account: Pubkey,
        burn: bool,
    ) -> Self {
        WhitelistInfo {
            set,
            mint,
            auth_account,
            minter_account,
            burn,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        set: bool,
        burn: bool,
        authority: (Pubkey, u64),
        minter: (Pubkey, u64),
    ) -> Self {
        let mint = create_mint(context, &authority.0, Some(&authority.0), 0, None)
            .await
            .unwrap();
        let atas = mint_to_wallets(context, &mint.pubkey(), vec![authority, minter])
            .await
            .unwrap();

        WhitelistInfo {
            set,
            mint: mint.pubkey(),
            burn,
            auth_account: atas[0],
            minter_account: atas[1],
        }
    }
}

impl Clone for WhitelistInfo {
    fn clone(&self) -> Self {
        WhitelistInfo {
            set: self.set,
            mint: clone_pubkey(&self.mint),
            minter_account: clone_pubkey(&self.minter_account),
            auth_account: clone_pubkey(&self.auth_account),
            burn: self.burn,
        }
    }
}

impl CandyManager {
    pub fn new(
        candy_machine: Keypair,
        authority: Keypair,
        wallet: Pubkey,
        minter: Keypair,
        collection_info: CollectionInfo,
        token_info: TokenInfo,
        whitelist_info: WhitelistInfo,
    ) -> Self {
        CandyManager {
            candy_machine,
            authority,
            wallet,
            minter,
            collection_info,
            token_info,
            whitelist_info,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        collection: bool,
        token: bool,
        whitelist: bool,
        burn: bool,
    ) -> Self {
        let candy_machine = Keypair::new();
        let authority = Keypair::new();
        let minter = Keypair::new();

        let collection_info = CollectionInfo::init(
            context,
            collection,
            &candy_machine.pubkey(),
            clone_keypair(&authority),
        )
        .await;
        let token_info = TokenInfo::init(
            context,
            token,
            (authority.pubkey(), 10),
            (minter.pubkey(), 1),
        )
        .await;
        let whitelist_info = WhitelistInfo::init(
            context,
            whitelist,
            burn,
            (authority.pubkey(), 10),
            (minter.pubkey(), 1),
        )
        .await;

        let wallet = match &token_info.set {
            true => token_info.minter_account,
            false => authority.pubkey(),
        };

        CandyManager::new(
            candy_machine,
            authority,
            wallet,
            minter,
            collection_info,
            token_info,
            whitelist_info,
        )
    }

    pub async fn get_candy(&self, context: &mut ProgramTestContext) -> CandyMachine {
        let account = get_account(context, &self.candy_machine.pubkey()).await;
        CandyMachine::try_deserialize(&mut account.data.as_ref()).unwrap()
    }

    pub async fn create(
        &mut self,
        context: &mut ProgramTestContext,
        candy_data: CandyMachineData,
    ) -> transport::Result<()> {
        initialize_candy_machine(
            context,
            &self.candy_machine,
            &self.authority,
            &self.wallet,
            candy_data,
            self.token_info.clone(),
        )
        .await
    }

    pub async fn set_collection(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        set_collection(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.collection_info,
        )
        .await?;
        self.collection_info.set = true;
        Ok(())
    }

    pub async fn remove_collection(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        remove_collection(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.collection_info,
        )
        .await?;
        self.collection_info.set = false;
        Ok(())
    }

    pub async fn fill_config_lines(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        add_all_config_lines(context, &self.candy_machine.pubkey(), &self.authority).await
    }

    pub async fn update(
        &mut self,
        context: &mut ProgramTestContext,
        new_wallet: Option<Pubkey>,
        new_data: CandyMachineData,
    ) -> transport::Result<()> {
        if let Some(wallet) = new_wallet {
            self.wallet = wallet;
        }
        update_candy_machine(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            new_data,
            &self.wallet,
        )
        .await
    }

    pub async fn mint_nft(
        &mut self,
        context: &mut ProgramTestContext,
        minter: &Keypair,
    ) -> transport::Result<MasterEditionManager> {
        let nft_info = prepare_nft(context, minter).await;
        let (candy_machine_creator, creator_bump) =
            find_candy_creator(&self.candy_machine.pubkey());
        mint_nft(
            context,
            &self.candy_machine.pubkey(),
            &candy_machine_creator,
            creator_bump,
            &self.wallet,
            &self.authority.pubkey(),
            minter,
            &nft_info,
            self.token_info.clone(),
            self.whitelist_info.clone(),
            self.collection_info.clone(),
        )
        .await?;
        Ok(nft_info)
    }

    pub async fn assert_mint_successful(
        &mut self,
        context: &mut ProgramTestContext,
        minter: &Keypair,
    ) {
        let candy_start = self.get_candy(context).await;
        let new_nft = self.mint_nft(context, minter).await.unwrap();
        let candy_end = self.get_candy(context).await;
        assert_eq!(candy_start.items_redeemed, candy_end.items_redeemed - 1);
        let metadata =
            metadata::Metadata::get_data_from_account(context, &new_nft.metadata_pubkey).await;
        if self.collection_info.set {
            assert_eq!(
                &metadata.collection.as_ref().unwrap().key,
                &self.collection_info.mint.pubkey()
            );
            assert!(&metadata.collection.as_ref().unwrap().verified);
        } else {
            assert!(&metadata.collection.is_none());
        }
    }

    pub async fn assert_bot_tax(&mut self, context: &mut ProgramTestContext, minter: &Keypair) {
        let start_balance = get_balance(context, &minter.pubkey()).await;
        let start_token_balance = get_token_balance(context, &self.token_info.minter_account).await;
        let start_whitelist_balance =
            get_token_balance(context, &self.whitelist_info.minter_account).await;
        let candy_start = self.get_candy(context).await;
        let new_nft = self.mint_nft(context, minter).await.unwrap();
        let candy_end = self.get_candy(context).await;
        let end_balance = get_balance(context, &minter.pubkey()).await;
        let end_token_balance = get_token_balance(context, &self.token_info.minter_account).await;
        let end_whitelist_balance =
            get_token_balance(context, &self.whitelist_info.minter_account).await;
        assert_eq!(start_balance - end_balance, BOT_FEE + 5000);
        assert_eq!(start_token_balance, end_token_balance);
        assert_eq!(start_whitelist_balance, end_whitelist_balance);
        assert_eq!(candy_start.items_redeemed, candy_end.items_redeemed);
        assert_acount_empty(context, &new_nft.metadata_pubkey).await;
    }
}
