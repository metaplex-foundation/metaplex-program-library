use std::sync::{Arc, Mutex};

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use console::style;
use mpl_candy_machine::{
    accounts as nft_accounts,
    constants::{FREEZE_FEATURE_INDEX, FREEZE_LOCK_FEATURE_INDEX},
    instruction as nft_instruction, is_feature_active,
};
use serde::{Deserialize, Serialize, Serializer};
use solana_client::{rpc_client::RpcClient, rpc_request::RpcRequest};
use solana_program::{instruction::AccountMeta, program_pack::Pack};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::state::Account as SplAccount;
use tokio::sync::Semaphore;

use crate::{
    cache::load_cache,
    candy_machine::{get_candy_machine_state, CANDY_MACHINE_ID},
    common::*,
    config::{get_config_data, Cluster, ConfigData, SugarConfig},
    pdas::*,
    setup::get_rpc_url,
    utils::{
        assert_correct_authority, get_cluster, get_cm_creator_mint_accounts,
        progress_bar_with_style, spinner_with_style,
    },
};

pub mod disable;
pub mod enable;
pub mod thaw;
pub mod unfreeze_funds;

pub use disable::*;
pub use enable::*;
pub use thaw::*;
pub use unfreeze_funds::*;
