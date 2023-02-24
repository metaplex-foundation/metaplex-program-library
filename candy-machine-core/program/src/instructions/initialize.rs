use anchor_lang::{prelude::*, Discriminator};
use mpl_token_metadata::state::{TokenStandard, MAX_SYMBOL_LENGTH};

use crate::{
    approve_collection_authority_helper,
    constants::{AUTHORITY_SEED, HIDDEN_SECTION},
    state::{CandyMachine, CandyMachineData},
    utils::fixed_length_string,
    AccountVersion, ApproveCollectionAuthorityHelperAccounts,
};

pub fn initialize(ctx: Context<Initialize>, data: CandyMachineData) -> Result<()> {
    let candy_machine_account = &mut ctx.accounts.candy_machine;

    let mut candy_machine = CandyMachine {
        data,
        version: AccountVersion::V1,
        token_standard: TokenStandard::NonFungible as u8,
        features: [0u8; 2],
        authority: ctx.accounts.authority.key(),
        mint_authority: ctx.accounts.authority.key(),
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

    let approve_accounts = ApproveCollectionAuthorityHelperAccounts {
        payer: ctx.accounts.payer.to_account_info(),
        authority_pda: ctx.accounts.authority_pda.to_account_info(),
        collection_mint: ctx.accounts.collection_mint.to_account_info(),
        collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
        collection_authority_record: ctx.accounts.collection_authority_record.to_account_info(),
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        collection_update_authority: ctx.accounts.collection_update_authority.to_account_info(),
    };

    approve_collection_authority_helper(approve_accounts)?;

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

    /// CHECK: account checked in seeds constraint
    #[account(
        mut,
        seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    authority_pda: UncheckedAccount<'info>,

    /// CHECK: authority can be any account and is not written to or read
    authority: UncheckedAccount<'info>,

    // payer of the transaction
    payer: Signer<'info>,

    /// CHECK: account checked in CPI
    collection_metadata: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    collection_master_edition: UncheckedAccount<'info>,

    #[account(mut)]
    collection_update_authority: Signer<'info>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,

    /// System program account.
    system_program: Program<'info, System>,
}
