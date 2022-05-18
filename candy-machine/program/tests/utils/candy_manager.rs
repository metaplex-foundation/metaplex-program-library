use std::fmt::Debug;

use anchor_lang::AccountDeserialize;
use mpl_token_metadata::pda::find_collection_authority_account;
use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transport;

use mpl_candy_machine::{CandyMachine, CandyMachineData};

use crate::core::master_edition_v2::MasterEditionV2;
use crate::core::{clone_pubkey, master_edition_v2, metadata};
use crate::helper_transactions::{remove_collection, set_collection};
use crate::utils::{find_candy_creator, find_collection_pda, initialize_candy_machine, mint_nft};
use crate::{add_all_config_lines, clone_keypair, get_account};

#[derive(Debug)]
pub struct CandyManager {
    pub candy_machine: Keypair,
    pub authority: Keypair,
    pub wallet: Keypair,
    pub collection_info: Option<CollectionInfo>,
    pub token_mint: Option<Pubkey>,
    pub whitelist_info: Option<WhitelistInfo>,
}

impl Clone for CandyManager {
    fn clone(&self) -> Self {
        CandyManager {
            candy_machine: clone_keypair(&self.candy_machine),
            authority: clone_keypair(&self.authority),
            wallet: clone_keypair(&self.wallet),
            collection_info: self.collection_info.clone(),
            token_mint: self.token_mint.map(|key| clone_pubkey(&key)),
            whitelist_info: self.whitelist_info.clone(),
        }
    }
}

#[derive(Debug)]
pub struct CollectionInfo {
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
    pub async fn new(
        context: &mut ProgramTestContext,
        candy_machine: &Pubkey,
        authority: Keypair,
    ) -> Self {
        let metadata_info = metadata::Metadata::new(authority);
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
        let master_edition_info = master_edition_v2::MasterEditionV2::new(&metadata_info);
        master_edition_info
            .create_v3(context, Some(0))
            .await
            .unwrap();

        let collection_pda = find_collection_pda(candy_machine).0;
        let collection_authority_record =
            find_collection_authority_account(&metadata_info.mint.pubkey(), &collection_pda).0;

        CollectionInfo {
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
pub struct WhitelistInfo {
    pub whitelist_token: Pubkey,
    pub whitelist_burn_info: Option<(Pubkey, Pubkey)>,
}

impl Clone for WhitelistInfo {
    fn clone(&self) -> Self {
        WhitelistInfo {
            whitelist_token: clone_pubkey(&self.whitelist_token),
            whitelist_burn_info: self
                .whitelist_burn_info
                .map(|info| (clone_pubkey(&info.0), clone_pubkey(&info.1))),
        }
    }
}

impl CandyManager {
    pub fn new() -> Self {
        CandyManager::new_with_custom_options(None, None)
    }

    pub fn new_with_custom_options(
        token_mint: Option<Pubkey>,
        whitelist_info: Option<WhitelistInfo>,
    ) -> Self {
        CandyManager {
            candy_machine: Keypair::new(),
            authority: Keypair::new(),
            wallet: Keypair::new(),
            collection_info: None,
            token_mint,
            whitelist_info,
        }
    }

    pub async fn new_with_options(
        token_mint: bool,
        whitelist_info: bool,
    ) -> transport::Result<Self> {
        todo!()
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
            &self.wallet.pubkey(),
            candy_data,
            self.token_mint,
        )
        .await
    }

    pub async fn set_collection(
        &mut self,
        context: &mut ProgramTestContext,
        collection_infos: Option<CollectionInfo>,
    ) -> transport::Result<CollectionInfo> {
        let collection_infos = match collection_infos {
            Some(collection_infos) => collection_infos,
            None => {
                CollectionInfo::new(
                    context,
                    &self.candy_machine.pubkey(),
                    clone_keypair(&self.authority),
                )
                .await
            }
        };

        self.collection_info = Some(collection_infos.clone());
        set_collection(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &collection_infos,
        )
        .await?;
        Ok(collection_infos)
    }

    pub async fn remove_collection(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<CollectionInfo> {
        let collection_info_to_return = self.collection_info.clone().unwrap();
        remove_collection(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &collection_info_to_return,
        )
        .await?;
        self.collection_info = None;
        Ok(collection_info_to_return)
    }

    pub async fn fill_config_lines(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        add_all_config_lines(context, &self.candy_machine.pubkey(), &self.authority).await
    }

    pub async fn mint_nft(
        &mut self,
        context: &mut ProgramTestContext,
        minter: &Keypair,
    ) -> transport::Result<MasterEditionV2> {
        let nft_info = metadata::Metadata::new(clone_keypair(minter));
        let new_nft = master_edition_v2::MasterEditionV2::new(&nft_info);
        let (candy_machine_creator, creator_bump) =
            find_candy_creator(&self.candy_machine.pubkey());
        mint_nft(
            context,
            &self.candy_machine.pubkey(),
            &candy_machine_creator,
            creator_bump,
            &self.wallet.pubkey(),
            &self.authority.pubkey(),
            minter,
            &new_nft,
            None,
            self.whitelist_info.clone(),
            self.collection_info.clone(),
        )
        .await?;
        Ok(new_nft)
    }
}
