use std::cmp::min;

use anchor_client::solana_sdk::{signature::Signer, system_program, sysvar};
use anchor_lang::*;
use mpl_token_metadata::pda::{
    find_collection_authority_account, find_master_edition_account, find_metadata_account,
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, transaction::Transaction, transport::TransportError};

use mpl_candy_machine::{
    constants::{CONFIG_ARRAY_START, CONFIG_LINE_SIZE},
    CandyError, CandyMachine, CandyMachineData, ConfigLine,
};

use crate::setup_functions::CandyResult::BotTax;

pub fn candy_machine_program_test() -> ProgramTest {
    let mut program = ProgramTest::new("mpl_candy_machine", mpl_candy_machine::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    program
}

pub async fn create_candy_machine(
    context: &mut ProgramTestContext,
    wallet: &Pubkey,
    candy_data: CandyMachineData,
    token_mint: Option<Pubkey>,
) -> std::result::Result<Pubkey, TransportError> {
    let candy_account = Keypair::new();

    let items_available = candy_data.items_available;
    let candy_account_size = if candy_data.hidden_settings.is_some() {
        CONFIG_ARRAY_START
    } else {
        CONFIG_ARRAY_START
            + 4
            + items_available as usize * CONFIG_LINE_SIZE
            + 8
            + 2 * (items_available as usize / 8 + 1)
    };

    let rent = context.banks_client.get_rent().await?;
    let lamports = rent.minimum_balance(candy_account_size);
    let create_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &candy_account.pubkey(),
        lamports,
        candy_account_size as u64,
        &mpl_candy_machine::id(),
    );

    let mut accounts = mpl_candy_machine::accounts::InitializeCandyMachine {
        candy_machine: candy_account.pubkey(),
        wallet: *wallet,
        authority: context.payer.pubkey(),
        payer: context.payer.pubkey(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    if let Some(token_mint) = token_mint {
        accounts.push(AccountMeta::new(token_mint, false));
    }

    let data = mpl_candy_machine::instruction::InitializeCandyMachine { data: candy_data }.data();

    let init_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, init_ix],
        Some(&context.payer.pubkey()),
        &[&candy_account, &context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .map(|_| candy_account.pubkey())
}

pub async fn set_collection(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    nft_mint: &Pubkey,
) -> std::result::Result<(), TransportError> {
    let collection_pda = Pubkey::find_program_address(
        &[b"collection".as_ref(), candy_machine.as_ref()],
        &mpl_candy_machine::id(),
    )
    .0;

    let master_edition_pda = find_master_edition_account(nft_mint).0;
    let collection_authority_record = find_collection_authority_account(nft_mint, candy_machine).0;
    let metadata_pda = find_metadata_account(nft_mint).0;

    let accounts = mpl_candy_machine::accounts::SetCollection {
        candy_machine: *candy_machine,
        authority: context.payer.pubkey(),
        collection_pda,
        payer: context.payer.pubkey(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
        metadata: metadata_pda,
        mint: *nft_mint,
        edition: master_edition_pda,
        collection_authority_record,
        token_metadata_program: mpl_token_metadata::id(),
    }
    .to_account_metas(None);

    let data = mpl_candy_machine::instruction::SetCollection {}.data();
    let set_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    let tx = Transaction::new_signed_with_payer(
        &[set_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn remove_collection(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    nft_mint: &Pubkey,
) -> std::result::Result<(), TransportError> {
    let collection_pda = Pubkey::find_program_address(
        &[b"collection".as_ref(), candy_machine.as_ref()],
        &mpl_candy_machine::id(),
    )
    .0;

    let collection_authority_record = find_collection_authority_account(nft_mint, candy_machine).0;
    let metadata_pda = find_metadata_account(nft_mint).0;

    let accounts = mpl_candy_machine::accounts::RemoveCollection {
        candy_machine: *candy_machine,
        authority: context.payer.pubkey(),
        collection_pda,
        metadata: metadata_pda,
        mint: *nft_mint,
        collection_authority_record,
        token_metadata_program: mpl_token_metadata::id(),
    }
    .to_account_metas(None);

    let data = mpl_candy_machine::instruction::RemoveCollection {}.data();
    let set_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    let tx = Transaction::new_signed_with_payer(
        &[set_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn add_config_lines(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    index: u32,
    config_lines: Vec<ConfigLine>,
) -> std::result::Result<(), TransportError> {
    let accounts = mpl_candy_machine::accounts::AddConfigLines {
        candy_machine: *candy_machine,
        authority: context.payer.pubkey(),
    }
    .to_account_metas(None);

    let data = mpl_candy_machine::instruction::AddConfigLines {
        index,
        config_lines,
    }
    .data();

    let add_config_line_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    let tx = Transaction::new_signed_with_payer(
        &[add_config_line_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn add_all_config_lines(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
) -> std::result::Result<(), TransportError> {
    let candy_machine_account = context
        .banks_client
        .get_account(*candy_machine)
        .await
        .expect("account not found")
        .expect("account empty");

    let candy_machine_data: CandyMachine =
        CandyMachine::try_deserialize(&mut candy_machine_account.data.as_ref()).unwrap();
    let total_items = candy_machine_data.data.items_available;
    for i in 0..total_items / 10 + 1 {
        let index = (i * 10) as u32;
        let lines_to_make = min(total_items - index as u64, 10);
        let config_lines = make_config_lines(index, lines_to_make as u8);
        add_config_lines(context, candy_machine, index, config_lines).await?;
    }

    Ok(())
}

pub fn make_config_lines(start_index: u32, total: u8) -> Vec<ConfigLine> {
    let mut config_lines = Vec::with_capacity(total as usize);
    for i in 0..total {
        config_lines.push(ConfigLine {
            name: format!("Item #{}", i as u32 + start_index),
            uri: format!("Item #{} URI", i as u32 + start_index),
        })
    }
    config_lines
}

pub struct CreateResult {
    pub result: CandyResult,
    pub candy_key: Pubkey,
}

pub enum CandyResult {
    Success,
    BotTax,
    Failure(CandyError),
}

impl Default for CandyResult {
    fn default() -> Self {
        BotTax
    }
}

pub fn check_result(data: &CandyMachineData) -> CandyResult {
    todo!()
}
