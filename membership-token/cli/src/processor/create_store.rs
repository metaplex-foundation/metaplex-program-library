//! Module provide handler for `CreateStore` command.

use super::UiTransactionInfo;
use crate::error;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, signature::Signer, signer::keypair::Keypair, system_program,
    transaction::Transaction,
};

/// Additional `CreateStore` instruction info, that need to be displayed in TUI.
#[derive(Debug)]
pub struct CreateStoreUiInfo {
    store: Keypair,
}

impl UiTransactionInfo for CreateStoreUiInfo {
    fn print(&self) {
        println!("CreateStore::store(pubkey) - {}", self.store.pubkey());
        println!("CreateStore::store(bytes) - {:?}", self.store.to_bytes());
    }
}

pub fn create_store(
    client: &RpcClient,
    payer: &Keypair,
    admin_wallet: &Keypair,
    name: &String,
    description: &String,
) -> Result<(Transaction, Box<dyn UiTransactionInfo>), error::Error> {
    let store = Keypair::new();

    let accounts = mpl_membership_token::accounts::CreateStore {
        admin: admin_wallet.pubkey(),
        store: store.pubkey(),
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = mpl_membership_token::instruction::CreateStore {
        name: name.to_owned(),
        description: description.to_owned(),
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
            &[payer, admin_wallet, &store],
            recent_blockhash,
        ),
        Box::new(CreateStoreUiInfo { store }),
    ))
}
