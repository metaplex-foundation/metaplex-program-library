use std::{fmt::Debug, str::FromStr};

use anchor_lang::AccountDeserialize;
use mpl_token_metadata::{pda::find_collection_authority_account, state::Metadata};
use solana_gateway::state::{get_expire_address_with_seed, get_gateway_token_address_with_seed};
use solana_program::{clock::Clock, program_option::COption, pubkey::Pubkey};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::AccountState;

use mpl_candy_machine::{
    constants::{BOT_FEE, FREEZE_FEATURE_INDEX, FREEZE_LOCK_FEATURE_INDEX},
    is_feature_active, CandyMachine, CandyMachineData, CollectionPDA, FreezePDA, WhitelistMintMode,
    WhitelistMintMode::{BurnEveryTime, NeverBurn},
};

use crate::{
    core::{
        helpers::{
            airdrop, assert_account_empty, clone_keypair, create_associated_token_account,
            create_mint, get_account, get_balance, get_token_account, get_token_balance,
            mint_to_wallets, prepare_nft,
        },
        MasterEditionManager, MetadataManager,
    },
    utils::{
        add_all_config_lines,
        helpers::{find_candy_creator, find_collection_pda, sol, CandyTestLogger},
        initialize_candy_machine, mint_nft, remove_collection, remove_freeze, set_collection,
        set_freeze, thaw_nft, unlock_funds, update_authority, update_candy_machine, withdraw_funds,
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
    pub gateway_info: GatekeeperInfo,
    pub freeze_info: FreezeInfo,
}

impl Clone for CandyManager {
    fn clone(&self) -> Self {
        CandyManager {
            candy_machine: clone_keypair(&self.candy_machine),
            authority: clone_keypair(&self.authority),
            wallet: self.wallet,
            minter: clone_keypair(&self.minter),
            collection_info: self.collection_info.clone(),
            token_info: self.token_info.clone(),
            whitelist_info: self.whitelist_info.clone(),
            gateway_info: self.gateway_info.clone(),
            freeze_info: self.freeze_info.clone(),
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
    pub sized: bool,
}

impl Clone for CollectionInfo {
    fn clone(&self) -> Self {
        CollectionInfo {
            set: self.set,
            pda: self.pda,
            mint: clone_keypair(&self.mint),
            metadata: self.metadata,
            master_edition: self.master_edition,
            token_account: self.token_account,
            authority_record: self.authority_record,
            sized: self.sized,
        }
    }
}

impl CollectionInfo {
    #[allow(dead_code)]
    pub fn new(
        set: bool,
        pda: Pubkey,
        mint: Keypair,
        metadata: Pubkey,
        master_edition: Pubkey,
        token_account: Pubkey,
        authority_record: Pubkey,
        sized: bool,
    ) -> Self {
        CollectionInfo {
            set,
            pda,
            mint,
            metadata,
            master_edition,
            token_account,
            authority_record,
            sized,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        set: bool,
        candy_machine: &Pubkey,
        authority: Keypair,
        sized: bool,
    ) -> Self {
        println!("Init Collection Info");
        let metadata_info = MetadataManager::new(&authority);
        metadata_info
            .create_v3(
                context,
                "Collection Name".to_string(),
                "COLLECTION".to_string(),
                "URI".to_string(),
                None,
                0,
                true,
                None,
                None,
                sized,
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
            mint: clone_keypair(&metadata_info.mint),
            metadata: metadata_info.pubkey,
            master_edition: master_edition_info.edition_pubkey,
            token_account: metadata_info.get_ata(),
            authority_record: collection_authority_record,
            sized,
        }
    }

    pub async fn get_metadata(&self, context: &mut ProgramTestContext) -> Metadata {
        MetadataManager::get_data_from_account(context, &self.metadata).await
    }
}

#[derive(Debug)]
pub struct TokenInfo {
    pub set: bool,
    pub mint: Pubkey,
    pub authority: Keypair,
    pub auth_account: Pubkey,
    pub minter_account: Pubkey,
}

impl TokenInfo {
    #[allow(dead_code)]
    pub fn new(
        set: bool,
        mint: Pubkey,
        authority: Keypair,
        auth_account: Pubkey,
        minter_account: Pubkey,
    ) -> Self {
        TokenInfo {
            set,
            mint,
            authority,
            auth_account,
            minter_account,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        set: bool,
        authority: &Keypair,
        authority_alloc: (Pubkey, u64),
        minter: (Pubkey, u64),
    ) -> Self {
        println!("Init token");
        let mint = create_mint(context, &authority.pubkey(), None, 0, None)
            .await
            .unwrap();
        let atas = mint_to_wallets(
            context,
            &mint.pubkey(),
            authority,
            vec![authority_alloc, minter],
        )
        .await
        .unwrap();

        TokenInfo {
            set,
            mint: mint.pubkey(),
            authority: clone_keypair(authority),
            auth_account: atas[0],
            minter_account: atas[1],
        }
    }
}

impl Clone for TokenInfo {
    fn clone(&self) -> Self {
        TokenInfo {
            set: self.set,
            mint: self.mint,
            authority: clone_keypair(&self.authority),
            auth_account: self.auth_account,
            minter_account: self.minter_account,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GatekeeperConfig {
    pub gatekeeper_network: Pubkey,
    pub expire_on_use: bool,
}

impl GatekeeperConfig {
    #[allow(dead_code)]
    pub fn new(gatekeeper_network: Pubkey, expire_on_use: bool) -> Self {
        GatekeeperConfig {
            gatekeeper_network,
            expire_on_use,
        }
    }
}

impl Default for GatekeeperConfig {
    fn default() -> Self {
        GatekeeperConfig {
            gatekeeper_network: Pubkey::from_str("ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6")
                .unwrap(),
            expire_on_use: false,
        }
    }
}

#[derive(Debug)]
pub struct GatekeeperInfo {
    pub set: bool,
    pub network_expire_feature: Option<Pubkey>,
    pub gateway_app: Pubkey,
    pub gateway_token_info: Pubkey,
    pub gatekeeper_config: GatekeeperConfig,
}

impl GatekeeperInfo {
    #[allow(dead_code)]
    pub fn new(
        set: bool,
        network_expire_feature: Option<Pubkey>,
        gateway_app: Pubkey,
        gateway_token_info: Pubkey,
        gatekeeper_config: GatekeeperConfig,
    ) -> Self {
        GatekeeperInfo {
            set,
            network_expire_feature,
            gateway_app,
            gateway_token_info,
            gatekeeper_config,
        }
    }

    pub async fn init(
        set: bool,
        gateway_app: Pubkey,
        gateway_token_info: Pubkey,
        gatekeeper_config: GatekeeperConfig,
        payer: Pubkey,
    ) -> Self {
        let network_token = get_gateway_token_address_with_seed(&payer, &None, &gateway_token_info);

        let expire_token: Option<Pubkey> = if gatekeeper_config.expire_on_use {
            let expire_token = get_expire_address_with_seed(&gateway_token_info);
            Some(expire_token.0)
        } else {
            None
        };

        GatekeeperInfo {
            set,
            network_expire_feature: expire_token,
            gateway_app,
            gateway_token_info: network_token.0,
            gatekeeper_config,
        }
    }
}

impl Clone for GatekeeperInfo {
    fn clone(&self) -> Self {
        GatekeeperInfo {
            set: self.set,
            network_expire_feature: self.network_expire_feature,
            gateway_app: self.gateway_app,
            gateway_token_info: self.gateway_token_info,
            gatekeeper_config: self.gatekeeper_config.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FreezeInfo {
    pub freeze_time: i64,
    pub set: bool,
    pub ata: Pubkey,
    pub pda: Pubkey,
}

impl FreezeInfo {
    pub fn new(set: bool, candy_machine: &Pubkey, freeze_time: i64, mint: Pubkey) -> Self {
        let seeds: &[&[u8]] = &[FreezePDA::PREFIX.as_bytes(), candy_machine.as_ref()];
        let pda = Pubkey::find_program_address(seeds, &mpl_candy_machine::ID).0;
        let freeze_ata = get_associated_token_address(&pda, &mint);
        FreezeInfo {
            set,
            pda,
            freeze_time,
            ata: freeze_ata,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        set: bool,
        candy_machine: &Pubkey,
        freeze_time: i64,
        mint: Pubkey,
    ) -> Self {
        let freeze_info = FreezeInfo::new(set, candy_machine, freeze_time, mint);
        create_associated_token_account(context, &freeze_info.pda, &mint)
            .await
            .unwrap();
        freeze_info
    }

    pub fn find_freeze_ata(&self, token_mint: &Pubkey) -> Pubkey {
        get_associated_token_address(&self.pda, token_mint)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FreezeConfig {
    pub set: bool,
    pub freeze_time: i64,
}

impl FreezeConfig {
    pub fn new(set: bool, freeze_time: i64) -> Self {
        Self { set, freeze_time }
    }
}

#[derive(Debug)]
pub struct WhitelistInfo {
    pub set: bool,
    pub mint: Pubkey,
    pub auth_account: Pubkey,
    pub minter_account: Pubkey,
    pub whitelist_config: WhitelistConfig,
}

#[derive(Debug, Clone)]
pub struct WhitelistConfig {
    pub burn: WhitelistMintMode,
    pub presale: bool,
    pub discount_price: Option<u64>,
}

impl WhitelistConfig {
    pub fn new(burn: WhitelistMintMode, presale: bool, discount_price: Option<u64>) -> Self {
        WhitelistConfig {
            burn,
            presale,
            discount_price,
        }
    }
}

impl Default for WhitelistConfig {
    fn default() -> Self {
        WhitelistConfig {
            burn: NeverBurn,
            presale: false,
            discount_price: None,
        }
    }
}

impl WhitelistInfo {
    #[allow(dead_code)]
    pub fn new(
        set: bool,
        mint: Pubkey,
        auth_account: Pubkey,
        minter_account: Pubkey,
        whitelist_config: WhitelistConfig,
    ) -> Self {
        WhitelistInfo {
            set,
            mint,
            auth_account,
            minter_account,
            whitelist_config,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        set: bool,
        authority: &Keypair,
        whitelist_config: WhitelistConfig,
        authority_alloc: (Pubkey, u64),
        minter: (Pubkey, u64),
    ) -> Self {
        println!("Init whitelist");
        let mint = create_mint(
            context,
            &authority.pubkey(),
            Some(&authority.pubkey()),
            0,
            None,
        )
        .await
        .unwrap();
        let atas = mint_to_wallets(
            context,
            &mint.pubkey(),
            authority,
            vec![authority_alloc, minter],
        )
        .await
        .unwrap();

        WhitelistInfo {
            set,
            mint: mint.pubkey(),
            whitelist_config,
            auth_account: atas[0],
            minter_account: atas[1],
        }
    }
}

impl Clone for WhitelistInfo {
    fn clone(&self) -> Self {
        WhitelistInfo {
            set: self.set,
            mint: self.mint,
            minter_account: self.minter_account,
            auth_account: self.auth_account,
            whitelist_config: self.whitelist_config.clone(),
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
        gateway_info: GatekeeperInfo,
        freeze_info: FreezeInfo,
    ) -> Self {
        CandyManager {
            candy_machine,
            authority,
            wallet,
            minter,
            collection_info,
            token_info,
            whitelist_info,
            gateway_info,
            freeze_info,
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        collection: Option<bool>,
        token: bool,
        freeze: Option<FreezeConfig>,
        whitelist: Option<WhitelistConfig>,
        gatekeeper: Option<GatekeeperInfo>,
    ) -> Self {
        let logger = CandyTestLogger::new_start("Init Candy Machine Manager");
        let candy_machine = Keypair::new();
        let authority = Keypair::new();
        let minter = Keypair::new();

        airdrop(context, &authority.pubkey(), sol(10.0))
            .await
            .unwrap();

        let sized = if let Some(sized) = &collection {
            *sized
        } else {
            false
        };

        let collection_info = CollectionInfo::init(
            context,
            collection.is_some(),
            &candy_machine.pubkey(),
            clone_keypair(&authority),
            sized,
        )
        .await;

        let token_info = TokenInfo::init(
            context,
            token,
            &authority,
            (authority.pubkey(), 10),
            (minter.pubkey(), 1),
        )
        .await;

        let freeze_info = match freeze {
            Some(config) => {
                FreezeInfo::init(
                    context,
                    config.set,
                    &candy_machine.pubkey(),
                    config.freeze_time,
                    token_info.mint,
                )
                .await
            }
            None => {
                FreezeInfo::init(context, false, &candy_machine.pubkey(), 0, token_info.mint).await
            }
        };

        let whitelist_info = match whitelist {
            Some(config) => {
                WhitelistInfo::init(
                    context,
                    true,
                    &authority,
                    config,
                    (authority.pubkey(), 10),
                    (minter.pubkey(), 1),
                )
                .await
            }
            None => {
                WhitelistInfo::init(
                    context,
                    false,
                    &authority,
                    WhitelistConfig::default(),
                    (authority.pubkey(), 10),
                    (minter.pubkey(), 1),
                )
                .await
            }
        };

        let gateway_info = match gatekeeper {
            Some(config) => {
                GatekeeperInfo::init(
                    true,
                    config.gateway_app,
                    config.gateway_token_info,
                    config.gatekeeper_config,
                    minter.pubkey(),
                )
                .await
            }
            None => {
                GatekeeperInfo::init(
                    false,
                    Pubkey::from_str("gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs").unwrap(),
                    Pubkey::from_str("ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6").unwrap(),
                    GatekeeperConfig::default(),
                    minter.pubkey(),
                )
                .await
            }
        };

        let wallet = match &token_info.set {
            true => token_info.auth_account,
            false => authority.pubkey(),
        };
        logger.end();
        CandyManager::new(
            candy_machine,
            authority,
            wallet,
            minter,
            collection_info,
            token_info,
            whitelist_info,
            gateway_info,
            freeze_info,
        )
    }

    pub async fn get_candy(&self, context: &mut ProgramTestContext) -> CandyMachine {
        let account = get_account(context, &self.candy_machine.pubkey()).await;
        CandyMachine::try_deserialize(&mut account.data.as_ref()).unwrap()
    }

    pub async fn get_collection_pda(&self, context: &mut ProgramTestContext) -> CollectionPDA {
        let account = get_account(context, &self.collection_info.pda).await;
        CollectionPDA::try_deserialize(&mut account.data.as_ref()).unwrap()
    }

    pub async fn get_freeze_pda(&self, context: &mut ProgramTestContext) -> FreezePDA {
        let account = get_account(context, &self.freeze_info.pda).await;
        FreezePDA::try_deserialize(&mut account.data.as_ref()).unwrap()
    }

    pub async fn assert_freeze_set(
        &self,
        context: &mut ProgramTestContext,
        expected_freeze_pda: &FreezePDA,
    ) -> FreezePDA {
        let freeze_pda_account = self.get_freeze_pda(context).await;
        let candy_machine_account = self.get_candy(context).await;
        assert_eq!(*expected_freeze_pda, freeze_pda_account);
        assert!(is_feature_active(
            &candy_machine_account.data.uuid,
            FREEZE_FEATURE_INDEX
        ));
        assert!(is_feature_active(
            &candy_machine_account.data.uuid,
            FREEZE_LOCK_FEATURE_INDEX
        ));
        freeze_pda_account
    }

    pub async fn assert_frozen(
        &self,
        context: &mut ProgramTestContext,
        new_nft: &MasterEditionManager,
    ) {
        let token_account = get_token_account(context, &new_nft.token_account)
            .await
            .unwrap();
        assert_eq!(
            token_account.state,
            AccountState::Frozen,
            "Token account state is not correct"
        );
        assert_eq!(
            token_account.delegate,
            COption::Some(self.freeze_info.pda),
            "Token account delegate is not correct"
        );
        assert_eq!(
            token_account.delegated_amount, 1,
            "Delegated amount is not correct"
        );
    }

    pub async fn assert_thawed(
        &self,
        context: &mut ProgramTestContext,
        new_nft: &MasterEditionManager,
        undelegated: bool,
    ) {
        let token_account = get_token_account(context, &new_nft.token_account)
            .await
            .unwrap();
        assert_eq!(
            token_account.state,
            AccountState::Initialized,
            "Token account state is not correct"
        );
        if undelegated {
            assert_eq!(
                token_account.delegate,
                COption::None,
                "Token account delegate is not None"
            );
            assert_eq!(
                token_account.delegated_amount, 0,
                "Delegated amount is not 0"
            );
        } else {
            assert_eq!(
                token_account.delegate,
                COption::Some(self.freeze_info.pda),
                "Token account delegate is not correct"
            );
            assert_eq!(
                token_account.delegated_amount, 1,
                "Delegated amount is not correct"
            );
        }
    }

    pub async fn create(
        &mut self,
        context: &mut ProgramTestContext,
        candy_data: CandyMachineData,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Initialize Candy Machine");
        initialize_candy_machine(
            context,
            &self.candy_machine,
            &self.authority,
            &self.wallet,
            candy_data,
            self.token_info.clone(),
        )
        .await?;
        logger.end();
        Ok(())
    }

    pub async fn set_collection(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Set Collection");
        set_collection(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.collection_info,
        )
        .await?;
        self.collection_info.set = true;
        logger.end();
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_collection(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Remove Collection");
        remove_collection(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.collection_info,
        )
        .await?;
        self.collection_info.set = false;
        logger.end();
        Ok(())
    }

    pub async fn fill_config_lines(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Fill Config Lines");
        add_all_config_lines(context, &self.candy_machine.pubkey(), &self.authority).await?;
        logger.end();
        Ok(())
    }

    pub async fn update(
        &mut self,
        context: &mut ProgramTestContext,
        new_wallet: Option<Pubkey>,
        new_data: CandyMachineData,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Update Candy Machine");
        if let Some(wallet) = new_wallet {
            self.wallet = wallet;
        }
        let token_info = if self.token_info.set {
            Some(self.token_info.mint)
        } else {
            None
        };
        update_candy_machine(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            new_data,
            &self.wallet,
            token_info,
        )
        .await?;
        logger.end();
        Ok(())
    }

    pub async fn update_authority(
        &mut self,
        context: &mut ProgramTestContext,
        new_authority: Pubkey,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Update Candy Machine Authority");
        update_authority(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.wallet,
            &new_authority,
        )
        .await?;
        logger.end();
        Ok(())
    }

    pub async fn set_freeze(&mut self, context: &mut ProgramTestContext) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Set freeze");
        set_freeze(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.freeze_info,
            &self.token_info,
        )
        .await?;
        self.freeze_info.set = true;
        logger.end();
        Ok(())
    }

    pub async fn remove_freeze(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Remove freeze");
        remove_freeze(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.freeze_info,
        )
        .await?;
        self.freeze_info.set = false;
        logger.end();
        Ok(())
    }

    pub async fn thaw_nft(
        &mut self,
        context: &mut ProgramTestContext,
        nft_info: &MasterEditionManager,
        authority: &Keypair,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Thaw NFT");
        thaw_nft(
            context,
            &self.candy_machine.pubkey(),
            authority,
            &self.freeze_info,
            nft_info,
        )
        .await?;
        logger.end();
        Ok(())
    }

    pub async fn unlock_funds(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        let logger = CandyTestLogger::new_start("Unlock Funds");
        unlock_funds(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.wallet,
            &self.freeze_info,
            &self.token_info,
        )
        .await?;
        logger.end();
        Ok(())
    }

    pub async fn mint_nft(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<MasterEditionManager> {
        let logger = CandyTestLogger::new_start("Mint NFT");
        let nft_info = prepare_nft(context, &self.minter).await;
        let (candy_machine_creator, creator_bump) =
            find_candy_creator(&self.candy_machine.pubkey());
        mint_nft(
            context,
            &self.candy_machine.pubkey(),
            &candy_machine_creator,
            creator_bump,
            &self.wallet,
            &self.authority.pubkey(),
            &self.minter,
            &nft_info,
            self.token_info.clone(),
            self.whitelist_info.clone(),
            self.collection_info.clone(),
            self.gateway_info.clone(),
            self.freeze_info.clone(),
        )
        .await?;
        logger.end();
        Ok(nft_info)
    }

    pub async fn withdraw(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<MasterEditionManager> {
        let logger = CandyTestLogger::new_start("Mint NFT");
        let nft_info = prepare_nft(context, &self.minter).await;
        withdraw_funds(
            context,
            &self.candy_machine.pubkey(),
            &self.authority,
            &self.collection_info,
        )
        .await?;
        logger.end();
        Ok(nft_info)
    }

    pub async fn mint_and_assert_successful(
        &mut self,
        context: &mut ProgramTestContext,
        balance_change: Option<u64>,
        auto_whitelist: bool,
    ) -> transport::Result<MasterEditionManager> {
        let candy_start = self.get_candy(context).await;
        let start_balance = get_balance(context, &self.minter.pubkey()).await;
        let wallet_to_use = if self.freeze_info.set && {
            let freeze = self.get_freeze_pda(context).await;
            let current_timestamp = context
                .banks_client
                .get_sysvar::<Clock>()
                .await?
                .unix_timestamp;
            !freeze.thaw_eligible(current_timestamp, &candy_start)
        } {
            if self.token_info.set {
                get_associated_token_address(&self.freeze_info.pda, &self.token_info.mint)
            } else {
                self.freeze_info.pda
            }
        } else {
            self.wallet
        };
        let start_wallet_balance = if self.token_info.set {
            get_token_balance(context, &wallet_to_use).await
        } else {
            get_balance(context, &wallet_to_use).await
        };
        let start_token_balance = get_token_balance(context, &self.token_info.minter_account).await;
        let start_whitelist_balance =
            get_token_balance(context, &self.whitelist_info.minter_account).await;
        let mut new_nft = self.mint_nft(context).await.unwrap();
        let candy_end = self.get_candy(context).await;
        let end_balance = get_balance(context, &self.minter.pubkey()).await;
        let end_wallet_balance = if self.token_info.set {
            get_token_balance(context, &wallet_to_use).await
        } else {
            get_balance(context, &wallet_to_use).await
        };
        let end_token_balance = get_token_balance(context, &self.token_info.minter_account).await;
        let end_whitelist_balance =
            get_token_balance(context, &self.whitelist_info.minter_account).await;
        let metadata =
            MetadataManager::get_data_from_account(context, &new_nft.metadata_pubkey).await;
        let associated_token_account =
            get_associated_token_address(&self.minter.pubkey(), &metadata.mint);
        let associated_token_account = get_token_account(context, &associated_token_account)
            .await
            .unwrap();

        assert_eq!(
            associated_token_account.amount, 1,
            "Minter is not the owner"
        );

        assert_eq!(
            candy_start.items_redeemed + 1,
            candy_end.items_redeemed,
            "Items redeemed wasn't 1"
        );
        if self.collection_info.set {
            assert_eq!(
                &metadata.collection.as_ref().unwrap().key,
                &self.collection_info.mint.pubkey(),
                "Collection key wasn't set correctly!"
            );
            assert!(
                &metadata.collection.as_ref().unwrap().verified,
                "Collection wasn't verified!"
            );
        } else {
            assert!(
                &metadata.collection.is_none(),
                "Collection was set when it shouldn't be!"
            );
        }

        let sol_fees = {
            let mut fees = 5000 + 5616720 + 2853600;
            if self.freeze_info.set {
                let freeze_pda = self.get_freeze_pda(context).await;
                fees += freeze_pda.freeze_fee;
            };
            fees
        };
        if let Some(change) = balance_change {
            assert_eq!(
                end_wallet_balance - start_wallet_balance,
                change,
                "CM wallet balance changed in a weird way!"
            );

            if self.token_info.set {
                assert_eq!(
                    start_token_balance - end_token_balance,
                    change,
                    "Token balance changed in a weird way!"
                );
                assert_eq!(
                    start_balance - end_balance,
                    sol_fees,
                    "Sol balance changed in a different way than it should have!"
                );
            } else {
                assert_eq!(
                    start_token_balance - end_token_balance,
                    0,
                    "Token balance changed when it shouldn't have!"
                );
                assert_eq!(
                    start_balance - end_balance,
                    sol_fees + change,
                    "Sol balance changed in a different way than it should have!"
                );
            }
        }
        if auto_whitelist {
            if self.whitelist_info.set
                && self.whitelist_info.whitelist_config.burn == BurnEveryTime
                && start_whitelist_balance > 0
            {
                assert_eq!(
                    start_whitelist_balance - end_whitelist_balance,
                    1,
                    "Whitelist token balance didn't decrease by 1!"
                );
            } else {
                assert_eq!(
                    start_whitelist_balance - end_whitelist_balance,
                    0,
                    "Whitelist token balance changed when it shouldn't have!"
                );
            }
        }
        new_nft.authority = clone_keypair(&self.authority);
        Ok(new_nft)
    }

    pub async fn mint_and_assert_bot_tax(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        let start_balance = get_balance(context, &self.minter.pubkey()).await;
        let start_token_balance = get_token_balance(context, &self.token_info.minter_account).await;
        let start_whitelist_balance =
            get_token_balance(context, &self.whitelist_info.minter_account).await;
        let candy_start = self.get_candy(context).await;
        let new_nft = self.mint_nft(context).await?;
        let candy_end = self.get_candy(context).await;
        let end_balance = get_balance(context, &self.minter.pubkey()).await;
        let end_token_balance = get_token_balance(context, &self.token_info.minter_account).await;
        let end_whitelist_balance =
            get_token_balance(context, &self.whitelist_info.minter_account).await;
        assert_eq!(
            start_balance - end_balance,
            BOT_FEE + 5000,
            "Balance changed in an unexpected way for this bot tax!"
        );
        assert_eq!(
            start_token_balance, end_token_balance,
            "SPL token balance changed!!"
        );
        assert_eq!(
            start_whitelist_balance, end_whitelist_balance,
            "Whitelist token balance changed!"
        );
        assert_eq!(
            candy_start.items_redeemed, candy_end.items_redeemed,
            "Items redeemed was not 0!"
        );
        assert_account_empty(context, &new_nft.metadata_pubkey).await;
        Ok(())
    }
}
