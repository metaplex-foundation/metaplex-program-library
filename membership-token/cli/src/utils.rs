//! Module define application utils.

use crate::error;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;

/// Return `Mint` account state from `spl_token` program.
pub fn get_mint(client: &RpcClient, mint: &Pubkey) -> Result<Mint, error::Error> {
    let data = client.get_account_data(mint)?;
    Ok(Mint::unpack(&data)?)
}
