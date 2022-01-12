//! Module provide handler for `CreateMarket` command.

use super::UiTransactionInfo;
use crate::error;
use anchor_lang::{InstructionData, ToAccountMetas};
use mpl_membership_token::utils::find_treasury_owner_address;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Signer, signer::keypair::Keypair,
    system_program, transaction::Transaction,
};

/// Additional `CreateMarket` instruction info, that need to be displayed in TUI.
#[derive(Debug)]
pub struct CreateMarketUiInfo {
    market: Keypair,
    owner: Pubkey,
}

impl UiTransactionInfo for CreateMarketUiInfo {
    fn print(&self) {
        println!("CreateMarket::market(pubkey) - {}", self.market.pubkey());
        println!("CreateMarket::market(bytes) - {:?}", self.market.to_bytes());
        println!("CreateMarket::owner - {}", self.owner);
    }
}

pub fn create_market(
    client: &RpcClient,
    payer: &Keypair,
    store: &Pubkey,
    selling_resource_owner: &Keypair,
    selling_resource: &Pubkey,
    mint: &Pubkey,
    treasury_holder: &Pubkey,
    name: &String,
    description: &String,
    mutable: bool,
    price: u64,
    pieces_in_one_wallet: Option<u64>,
    start_date: i64,
    end_date: Option<i64>,
) -> Result<(Transaction, Box<dyn UiTransactionInfo>), error::Error> {
    let (owner, treasury_owner_bump) = find_treasury_owner_address(mint, selling_resource);

    let market = Keypair::new();

    let accounts = mpl_membership_token::accounts::CreateMarket {
        market: market.pubkey(),
        store: *store,
        selling_resource_owner: selling_resource_owner.pubkey(),
        selling_resource: *selling_resource,
        mint: *mint,
        treasury_holder: *treasury_holder,
        owner,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = mpl_membership_token::instruction::CreateMarket {
        _treasyry_owner_bump: treasury_owner_bump,
        name: name.clone(),
        description: description.clone(),
        mutable,
        price,
        pieces_in_one_wallet,
        start_date: start_date as u64,
        end_date: if let Some(end_date) = end_date {
            Some(end_date as u64)
        } else {
            None
        },
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_membership_token::id(),
        data,
        accounts,
    };

    let recent_blockhash = client.get_latest_blockhash()?;

    Ok((
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer, &market, selling_resource_owner],
            recent_blockhash,
        ),
        Box::new(CreateMarketUiInfo { market, owner }),
    ))
}
