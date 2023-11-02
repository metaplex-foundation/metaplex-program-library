use anchor_client::solana_sdk::{signature::Signer, system_program, sysvar};
use anchor_lang::*;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, transaction::Transaction};

use mpl_candy_machine::{
    constants::{CONFIG_ARRAY_START, CONFIG_LINE_SIZE},
    CandyMachine, CandyMachineData, ConfigLine,
    WhitelistMintMode::BurnEveryTime,
};

use crate::{
    core::{helpers::update_blockhash, MasterEditionManager},
    utils::{
        candy_manager::{CollectionInfo, GatekeeperInfo, TokenInfo, WhitelistInfo},
        helpers::make_config_lines,
        FreezeInfo,
    },
};
use std::result::Result;

pub fn candy_machine_program_test() -> ProgramTest {
    let mut program = ProgramTest::new("mpl_candy_machine", mpl_candy_machine::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    program.set_compute_max_units(400_000);
    program
}

pub async fn initialize_candy_machine(
    context: &mut ProgramTestContext,
    candy_account: &Keypair,
    payer: &Keypair,
    wallet: &Pubkey,
    candy_data: CandyMachineData,
    token_info: TokenInfo,
) -> Result<(), BanksClientError> {
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

    update_blockhash(context).await?;
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
) -> Result<(), BanksClientError> {
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

    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn update_authority(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    wallet: &Pubkey,
    new_authority: &Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = mpl_candy_machine::accounts::UpdateCandyMachine {
        candy_machine: *candy_machine,
        authority: authority.pubkey(),
        wallet: *wallet,
    }
    .to_account_metas(None);
    let data = mpl_candy_machine::instruction::UpdateAuthority {
        new_authority: Some(*new_authority),
    }
    .data();
    let update_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };

    update_blockhash(context).await?;
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
) -> Result<(), BanksClientError> {
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

    update_blockhash(context).await?;

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
) -> Result<(), BanksClientError> {
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
) -> Result<(), BanksClientError> {
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

    update_blockhash(context).await?;
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
) -> Result<(), BanksClientError> {
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

    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &[set_ix],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn set_freeze(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    freeze_info: &FreezeInfo,
    token_info: &TokenInfo,
) -> Result<(), BanksClientError> {
    let mut accounts = mpl_candy_machine::accounts::SetFreeze {
        candy_machine: *candy_machine,
        freeze_pda: freeze_info.pda,
        authority: authority.pubkey(),
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    if token_info.set {
        accounts.push(AccountMeta::new(freeze_info.ata, false));
    }

    let data = mpl_candy_machine::instruction::SetFreeze {
        freeze_time: freeze_info.freeze_time,
    }
    .data();
    let set_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };

    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &[set_ix],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn remove_freeze(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    freeze_info: &FreezeInfo,
) -> Result<(), BanksClientError> {
    let accounts = mpl_candy_machine::accounts::RemoveFreeze {
        candy_machine: *candy_machine,
        authority: authority.pubkey(),
        freeze_pda: freeze_info.pda,
    }
    .to_account_metas(None);

    let data = mpl_candy_machine::instruction::RemoveFreeze {}.data();
    let set_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &[set_ix],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn thaw_nft(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    signer: &Keypair,
    freeze_info: &FreezeInfo,
    nft_info: &MasterEditionManager,
) -> Result<(), BanksClientError> {
    let accounts = mpl_candy_machine::accounts::ThawNFT {
        freeze_pda: freeze_info.pda,
        candy_machine: *candy_machine,
        token_account: nft_info.token_account,
        owner: nft_info.owner.pubkey(),
        mint: nft_info.mint.pubkey(),
        edition: nft_info.edition_pubkey,
        payer: signer.pubkey(),
        token_program: spl_token::ID,
        token_metadata_program: mpl_token_metadata::ID,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = mpl_candy_machine::instruction::ThawNft {}.data();
    let set_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &[set_ix],
        Some(&signer.pubkey()),
        &[signer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn unlock_funds(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    treasury: &Pubkey,
    freeze_info: &FreezeInfo,
    token_info: &TokenInfo,
) -> Result<(), BanksClientError> {
    let mut accounts = mpl_candy_machine::accounts::UnlockFunds {
        freeze_pda: freeze_info.pda,
        candy_machine: *candy_machine,
        authority: authority.pubkey(),
        wallet: *treasury,
        system_program: system_program::id(),
    }
    .to_account_metas(None);
    if token_info.set {
        accounts.push(AccountMeta::new_readonly(spl_token::id(), false));
        accounts.push(AccountMeta::new(
            freeze_info.find_freeze_ata(&token_info.mint),
            false,
        ));
        accounts.push(AccountMeta::new(token_info.auth_account, false));
    }

    let data = mpl_candy_machine::instruction::UnlockFunds {}.data();
    let set_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &[set_ix],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn withdraw_funds(
    context: &mut ProgramTestContext,
    candy_machine: &Pubkey,
    authority: &Keypair,
    collection_info: &CollectionInfo,
) -> Result<(), BanksClientError> {
    let mut accounts = mpl_candy_machine::accounts::WithdrawFunds {
        candy_machine: *candy_machine,
        authority: authority.pubkey(),
    }
    .to_account_metas(None);
    if collection_info.set {
        accounts.push(AccountMeta::new(collection_info.pda, false));
    }

    let data = mpl_candy_machine::instruction::WithdrawFunds {}.data();
    let set_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &[set_ix],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

#[allow(clippy::too_many_arguments)]
pub fn mint_nft_ix(
    candy_machine: &Pubkey,
    candy_creator_pda: &Pubkey,
    creator_bump: u8,
    wallet: &Pubkey,
    authority: &Pubkey,
    payer: &Keypair,
    new_nft: &MasterEditionManager,
    token_info: TokenInfo,
    whitelist_info: WhitelistInfo,
    collection_info: CollectionInfo,
    gateway_info: GatekeeperInfo,
    freeze_info: FreezeInfo,
) -> Vec<Instruction> {
    let metadata = new_nft.metadata_pubkey;
    let master_edition = new_nft.edition_pubkey;
    let mint = new_nft.mint.pubkey();

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

    if gateway_info.set {
        accounts.push(AccountMeta::new(gateway_info.gateway_token_info, false));

        if gateway_info.gatekeeper_config.expire_on_use {
            accounts.push(AccountMeta::new_readonly(gateway_info.gateway_app, false));
            if let Some(expire_token) = gateway_info.network_expire_feature {
                accounts.push(AccountMeta::new_readonly(expire_token, false));
            }
        }
    }

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

    if freeze_info.set {
        accounts.push(AccountMeta::new(freeze_info.pda, false));
        accounts.push(AccountMeta::new(new_nft.token_account, false));
        if token_info.set {
            accounts.push(AccountMeta::new(
                freeze_info.find_freeze_ata(&token_info.mint),
                false,
            ));
        }
    }

    let data = mpl_candy_machine::instruction::MintNft { creator_bump }.data();

    let mut instructions = Vec::new();

    let mint_ix = Instruction {
        program_id: mpl_candy_machine::id(),
        data,
        accounts,
    };
    instructions.push(mint_ix);

    if collection_info.set {
        let mut accounts = mpl_candy_machine::accounts::SetCollectionDuringMint {
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
        if collection_info.sized {
            accounts
                .iter_mut()
                .find(|m| m.pubkey == collection_info.metadata)
                .unwrap()
                .is_writable = true;
        }
        let data = mpl_candy_machine::instruction::SetCollectionDuringMint {}.data();
        let set_ix = Instruction {
            program_id: mpl_candy_machine::id(),
            data,
            accounts,
        };
        instructions.push(set_ix)
    }
    instructions
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
    new_nft: &MasterEditionManager,
    token_info: TokenInfo,
    whitelist_info: WhitelistInfo,
    collection_info: CollectionInfo,
    gateway_info: GatekeeperInfo,
    freeze_info: FreezeInfo,
) -> Result<(), BanksClientError> {
    let ins = mint_nft_ix(
        candy_machine,
        candy_creator_pda,
        creator_bump,
        wallet,
        authority,
        payer,
        new_nft,
        token_info,
        whitelist_info,
        collection_info,
        gateway_info,
        freeze_info,
    );
    let signers = vec![payer];
    update_blockhash(context).await?;
    let tx = Transaction::new_signed_with_payer(
        &ins,
        Some(&payer.pubkey()),
        &signers,
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
