use std::{fmt::Debug, str::FromStr};

use anchor_lang::AccountDeserialize;
use mpl_token_metadata::pda::find_collection_authority_account;
use solana_gateway::state::{get_expire_address_with_seed, get_gateway_token_address_with_seed};
use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};
use spl_associated_token_account::get_associated_token_address;

use mpl_candy_machine::{
    constants::BOT_FEE,
    CandyMachine, CandyMachineData, WhitelistMintMode,
    WhitelistMintMode::{BurnEveryTime, NeverBurn},
};

use crate::{
    core::{
        helpers::{
            airdrop, assert_account_empty, clone_keypair, clone_pubkey, create_mint, get_account,
            get_balance, get_token_account, get_token_balance, mint_to_wallets, prepare_nft,
        },
        MasterEditionV2 as MasterEditionManager, Metadata as MetadataManager,
    },
    utils::{
        add_all_config_lines,
        helpers::{find_candy_creator, find_collection_pda, sol},
        initialize_candy_machine, mint_nft, remove_collection, set_collection,
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
    pub gateway_info: GatekeeperInfo,
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
            gateway_info: self.gateway_info.clone(),
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
    #[allow(dead_code)]
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
        println!("Init Collection Info");
        let metadata_info = MetadataManager::new(&authority);
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
            mint: clone_pubkey(&self.mint),
            authority: clone_keypair(&self.authority),
            auth_account: clone_pubkey(&self.auth_account),
            minter_account: clone_pubkey(&self.minter_account),
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
            gateway_app: clone_pubkey(&self.gateway_app),
            gateway_token_info: clone_pubkey(&self.gateway_token_info),
            gatekeeper_config: self.gatekeeper_config.clone(),
        }
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
            mint: clone_pubkey(&self.mint),
            minter_account: clone_pubkey(&self.minter_account),
            auth_account: clone_pubkey(&self.auth_account),
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
        }
    }

    pub async fn init(
        context: &mut ProgramTestContext,
        collection: bool,
        token: bool,
        whitelist: Option<WhitelistConfig>,
        gatekeeper: Option<GatekeeperInfo>,
    ) -> Self {
        println!("Init Candy Machine Manager");
        let candy_machine = Keypair::new();
        let authority = Keypair::new();
        let minter = Keypair::new();

        airdrop(context, &authority.pubkey(), sol(10.0))
            .await
            .unwrap();

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
            &authority,
            (authority.pubkey(), 10),
            (minter.pubkey(), 1),
        )
        .await;

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

        CandyManager::new(
            candy_machine,
            authority,
            wallet,
            minter,
            collection_info,
            token_info,
            whitelist_info,
            gateway_info,
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
        println!("Create");
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
        println!("Set collection");
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

    #[allow(dead_code)]
    pub async fn remove_collection(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        println!("Remove collection");
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
        println!("Fill config lines");
        add_all_config_lines(context, &self.candy_machine.pubkey(), &self.authority).await
    }

    pub async fn update(
        &mut self,
        context: &mut ProgramTestContext,
        new_wallet: Option<Pubkey>,
        new_data: CandyMachineData,
    ) -> transport::Result<()> {
        println!("Update");
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
        .await
    }

    pub async fn mint_nft(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<MasterEditionManager> {
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
        )
        .await?;
        Ok(nft_info)
    }

    pub async fn mint_and_assert_successful(
        &mut self,
        context: &mut ProgramTestContext,
        balance_change: Option<u64>,
        auto_whitelist: bool,
    ) {
        println!("Mint and assert successful");
        let candy_start = self.get_candy(context).await;
        let start_balance = get_balance(context, &self.minter.pubkey()).await;
        let start_wallet_balance = if self.token_info.set {
            get_token_balance(context, &self.wallet).await
        } else {
            get_balance(context, &self.wallet).await
        };
        let start_token_balance = get_token_balance(context, &self.token_info.minter_account).await;
        let start_whitelist_balance =
            get_token_balance(context, &self.whitelist_info.minter_account).await;
        let new_nft = self.mint_nft(context).await.unwrap();
        let candy_end = self.get_candy(context).await;
        let end_balance = get_balance(context, &self.minter.pubkey()).await;
        let end_wallet_balance = if self.token_info.set {
            get_token_balance(context, &self.wallet).await
        } else {
            get_balance(context, &self.wallet).await
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
        let sol_fees = 5000 + 5616720 + 2853600;
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
    }

    pub async fn mint_and_assert_bot_tax(
        &mut self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<()> {
        println!("Mint and assert bot tax");
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
