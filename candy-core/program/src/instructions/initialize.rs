use anchor_lang::{prelude::*, Discriminator};
use mpl_token_metadata::state::MAX_SYMBOL_LENGTH;

use crate::{
    constants::HIDDEN_SECTION,
    set_collection_helper,
    state::{CandyMachine, CandyMachineData},
    utils::fixed_length_string,
    SetCollectionHelperAccounts,
};

pub fn initialize(ctx: Context<Initialize>, data: CandyMachineData) -> Result<()> {
    let candy_machine_account = &mut ctx.accounts.candy_machine;

    let mut candy_machine = CandyMachine {
        data,
        features: 0,
        authority: ctx.accounts.authority.key(),
        update_authority: ctx.accounts.update_authority.key(),
        collection_mint: ctx.accounts.collection_mint.key(),
        items_redeemed: 0,
    };

    candy_machine.data.symbol = fixed_length_string(candy_machine.data.symbol, MAX_SYMBOL_LENGTH)?;
    // validates the config lines settings
    candy_machine.data.validate()?;

    let mut struct_data = CandyMachine::discriminator().try_to_vec().unwrap();
    struct_data.append(&mut candy_machine.try_to_vec().unwrap());

    let mut account_data = candy_machine_account.data.borrow_mut();
    account_data[0..struct_data.len()].copy_from_slice(&struct_data);

    if candy_machine.data.hidden_settings.is_none() {
        // set the initial number of config lines
        account_data[HIDDEN_SECTION..HIDDEN_SECTION + 4].copy_from_slice(&u32::MIN.to_le_bytes());
    }

    let set_collection_helper_accounts = SetCollectionHelperAccounts {
        payer: ctx.accounts.payer.to_account_info(),
        update_authority: ctx.accounts.update_authority.to_account_info(),
        collection_mint: ctx.accounts.collection_mint.to_account_info(),
        collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
        collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
        collection_authority_record: ctx.accounts.collection_authority_record.to_account_info(),
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    set_collection_helper(set_collection_helper_accounts)?;

    Ok(())
}

/// Create a new candy machine.
#[derive(Accounts)]
#[instruction(data: CandyMachineData)]
pub struct Initialize<'info> {
    /// CHECK: account constraints checked in account trait
    #[account(
        zero,
        rent_exempt = skip,
        constraint = candy_machine.to_account_info().owner == program_id && candy_machine.to_account_info().data_len() >= data.get_space_for_candy()?
    )]
    candy_machine: UncheckedAccount<'info>,
    /// CHECK: authority can be any account and is not written to or read
    authority: UncheckedAccount<'info>,
    // The update authority is used when retain authority is true (in most cases it will
    // be the same as the authority)
    /// CHECK: update authority can be any account and is not written to or read
    update_authority: UncheckedAccount<'info>,
    // payer of the transaction
    payer: Signer<'info>,
    /// CHECK: account checked in CPI
    collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}
