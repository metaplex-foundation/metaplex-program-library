use anchor_client::solana_sdk::{signature::Signer, system_program, sysvar};
use anchor_lang::*;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, transaction::Transaction, transport};

use mpl_candy_machine::{
    constants::{CONFIG_ARRAY_START, CONFIG_LINE_SIZE},
    CandyMachine, CandyMachineData, ConfigLine,
    WhitelistMintMode::BurnEveryTime,
};

use crate::{
    core::MasterEditionV2,
    utils::{
        candy_manager::{CollectionInfo, TokenInfo, WhitelistInfo},
        helpers::make_config_lines,
    },
};

pub fn candy_machine_program_test() -> ProgramTest {
    let mut program = ProgramTest::new("mpl_candy_machine", mpl_candy_machine::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    // program.add_program("solana_gateway_program", solana_gateway::id(), None);
    program
}

pub async fn initialize_candy_machine(
    context: &mut ProgramTestContext,
    candy_account: &Keypair,
    payer: &Keypair,
    wallet: &Pubkey,
    candy_data: CandyMachineData,
    token_info: TokenInfo,
) -> transport::Result<()> {
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
        &payer.pubkey(),
        &candy_account.pubkey(),
        lamports,
        candy_account_size as u64,
        &mpl_candy_machine::id(),
    );

    let mut accounts = mpl_candy_machine::accounts::InitializeCandyMachine {
        candy_machine: candy_account.pubkey(),
        wallet: *wallet,
        authority: payer.pubkey(),
        payer: payer.pubkey(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    if token_info.set {
        accounts.push(AccountMeta::new_readonly(token_info.mint, false));
    }

    let data = mpl_candy_machine::instruction::InitializeCandyMachine { data: candy_data }.data();

    let init_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, init_ix],
        Some(&payer.pubkey()),
        &[candy_account, payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn update_candy_machine(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    data: CandyMachineData,
    wallet: &Pubkey,
    token_mint: Option<Pubkey>,
) -> transport::Result<()> {
    let mut accounts = mpl_candy_machine::accounts::UpdateCandyMachine {
        candy_machine: *candy_machine,
        authority: authority.pubkey(),
        wallet: *wallet,
    }
    .to_account_metas(None);
    if let Some(token_mint) = token_mint {
        accounts.push(AccountMeta::new_readonly(token_mint, false));
    }

    let data = mpl_candy_machine::instruction::UpdateCandyMachine { data }.data();
    let update_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn add_config_lines(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    index: u32,
    config_lines: Vec<ConfigLine>,
) -> transport::Result<()> {
    let accounts = mpl_candy_machine::accounts::AddConfigLines {
        candy_machine: *candy_machine,
        authority: authority.pubkey(),
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
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn add_all_config_lines(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
) -> transport::Result<()> {
    let candy_machine_account = context
        .banks_client
        .get_account(*candy_machine)
        .await
        .expect("account not found")
        .expect("account empty");

    let candy_machine_data: CandyMachine =
        CandyMachine::try_deserialize(&mut candy_machine_account.data.as_ref()).unwrap();
    let total_items = candy_machine_data.data.items_available;
    for i in 0..total_items / 10 {
        let index = (i * 10) as u32;
        let config_lines = make_config_lines(index, 10);
        add_config_lines(context, candy_machine, authority, index, config_lines).await?;
    }
    let remainder = total_items & 10;
    if remainder > 0 {
        let index = (total_items as u32 / 10).saturating_sub(1);
        let config_lines = make_config_lines(index, remainder as u8);
        add_config_lines(context, candy_machine, authority, index, config_lines).await?;
    }

    Ok(())
}

pub async fn set_collection(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    collection_info: &CollectionInfo,
) -> transport::Result<()> {
    let accounts = mpl_candy_machine::accounts::SetCollection {
        candy_machine: *candy_machine,
        authority: authority.pubkey(),
        collection_pda: collection_info.pda,
        payer: authority.pubkey(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
        metadata: collection_info.metadata,
        mint: collection_info.mint.pubkey(),
        edition: collection_info.master_edition,
        collection_authority_record: collection_info.authority_record,
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
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

#[allow(dead_code)]
pub async fn remove_collection(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    collection_info: &CollectionInfo,
) -> transport::Result<()> {
    let accounts = mpl_candy_machine::accounts::RemoveCollection {
        candy_machine: *candy_machine,
        authority: authority.pubkey(),
        collection_pda: collection_info.pda,
        metadata: collection_info.metadata,
        mint: collection_info.mint.pubkey(),
        collection_authority_record: collection_info.authority_record,
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
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

#[allow(clippy::too_many_arguments)]
pub async fn mint_nft(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    candy_creator_pda: &Pubkey,
    creator_bump: u8,
    wallet: &Pubkey,
    authority: &Pubkey,
    payer: &Keypair,
    new_nft: &MasterEditionV2,
    token_info: TokenInfo,
    whitelist_info: WhitelistInfo,
    collection_info: CollectionInfo,
) -> transport::Result<()> {
    let metadata = new_nft.metadata_pubkey;
    let master_edition = new_nft.pubkey;
    let mint = new_nft.mint_pubkey;

    let mut accounts = mpl_candy_machine::accounts::MintNFT {
        candy_machine: *candy_machine,
        candy_machine_creator: *candy_creator_pda,
        payer: payer.pubkey(),
        wallet: *wallet,
        metadata,
        mint,
        mint_authority: payer.pubkey(),
        update_authority: payer.pubkey(),
        master_edition,
        token_metadata_program: mpl_token_metadata::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
        clock: sysvar::clock::id(),
        recent_blockhashes: sysvar::slot_hashes::id(),
        instruction_sysvar_account: sysvar::instructions::id(),
    }
    .to_account_metas(None);

    if whitelist_info.set {
        accounts.push(AccountMeta::new(whitelist_info.minter_account, false));
        if whitelist_info.whitelist_config.burn == BurnEveryTime {
            accounts.push(AccountMeta::new(whitelist_info.mint, false));
            accounts.push(AccountMeta::new_readonly(payer.pubkey(), true));
        }
    }

    if token_info.set {
        accounts.push(AccountMeta::new(token_info.minter_account, false));
        accounts.push(AccountMeta::new_readonly(payer.pubkey(), false));
    }

    let data = mpl_candy_machine::instruction::MintNft { creator_bump }.data();

    let mut instructions = Vec::new();
    let mut signers = Vec::new();

    let mint_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    instructions.push(mint_ix);
    signers.push(payer);

    if collection_info.set {
        let accounts = mpl_candy_machine::accounts::SetCollectionDuringMint {
            candy_machine: *candy_machine,
            metadata,
            payer: payer.pubkey(),
            collection_pda: collection_info.pda,
            token_metadata_program: mpl_token_metadata::id(),
            instructions: sysvar::instructions::id(),
            collection_mint: collection_info.mint.pubkey(),
            collection_metadata: collection_info.metadata,
            collection_master_edition: collection_info.master_edition,
            authority: *authority,
            collection_authority_record: collection_info.authority_record,
        }
        .to_account_metas(None);

        let data = mpl_candy_machine::instruction::SetCollectionDuringMint {}.data();
        let collection_ix = Instruction {
            program_id: mpl_candy_machine::id(),
            data,
            accounts,
        };
        instructions.push(collection_ix);
    }

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &signers,
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
