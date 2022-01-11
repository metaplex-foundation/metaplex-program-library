//! Module provide handler for `Buy` command.

use super::{get_account_state, UiTransactionInfo};
use crate::error;
use anchor_lang::{InstructionData, ToAccountMetas};
use mpl_membership_token::utils::{find_trade_history_address, find_vault_owner_address};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Signer,
    signer::keypair::Keypair,
    system_program,
    sysvar::{clock, rent},
    transaction::Transaction,
};

/// Additional `Buy` instruction info, that need to be displayed in TUI.
#[derive(Debug)]
pub struct BuyUiInfo {
    owner: Pubkey,
    trade_history: Pubkey,
    edition_marker: Pubkey,
    new_metadata: Pubkey,
    new_edition: Pubkey,
}

impl UiTransactionInfo for BuyUiInfo {
    fn print(&self) {
        println!("Buy::owner - {}", self.owner);
        println!("Buy::trade_history - {}", self.trade_history);
        println!("Buy::edition_marker - {}", self.edition_marker);
        println!("Buy::new_metadata - {}", self.new_metadata);
        println!("Buy::new_edition - {}", self.new_edition);
    }
}

pub fn buy(
    client: &RpcClient,
    payer: &Keypair,
    market: &Pubkey,
    selling_resource: &Pubkey,
    user_token_account: &Pubkey,
    user_wallet: &Keypair,
    treasury_holder: &Pubkey,
    master_edition: &Pubkey,
    new_mint: &Pubkey,
    vault: &Keypair,
    master_edition_metadata: &Pubkey,
    resource_mint: &Pubkey,
    store: &Pubkey,
) -> Result<(Transaction, Box<dyn UiTransactionInfo>), error::Error> {
    let (owner, vault_owner_bump) = find_vault_owner_address(resource_mint, store);
    let (trade_history, trade_history_bump) =
        find_trade_history_address(&user_wallet.pubkey(), market);

    let selling_resource_supply =
        get_account_state::<mpl_membership_token::state::SellingResource>(
            client,
            selling_resource,
        )?
        .supply;

    let (edition_marker, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            resource_mint.as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
            selling_resource_supply.to_string().as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    let (new_metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            new_mint.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let (new_edition, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            new_mint.as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    let accounts = mpl_membership_token::accounts::Buy {
        market: *market,
        selling_resource: *selling_resource,
        user_token_account: *user_token_account,
        user_wallet: user_wallet.pubkey(),
        trade_history,
        treasury_holder: *treasury_holder,
        new_metadata,
        new_edition,
        master_edition: *master_edition,
        new_mint: *new_mint,
        edition_marker,
        vault: vault.pubkey(),
        owner,
        master_edition_metadata: *master_edition_metadata,
        clock: clock::id(),
        rent: rent::id(),
        token_metadata_program: mpl_token_metadata::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = mpl_membership_token::instruction::Buy {
        _trade_history_bump: trade_history_bump,
        vault_owner_bump,
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
            &[payer, user_wallet, vault],
            recent_blockhash,
        ),
        Box::new(BuyUiInfo {
            owner,
            edition_marker,
            new_edition,
            new_metadata,
            trade_history,
        }),
    ))
}
