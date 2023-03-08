use anchor_lang::{prelude::*, solana_program::sysvar, Discriminator};
use mpl_token_metadata::{
    state::{TokenStandard, MAX_SYMBOL_LENGTH},
    utils::resize_or_reallocate_account_raw,
};

use crate::{
    approve_metadata_delegate, assert_token_standard,
    constants::{AUTHORITY_SEED, HIDDEN_SECTION, RULE_SET_LENGTH, SET},
    state::{CandyMachine, CandyMachineData},
    utils::fixed_length_string,
    AccountVersion, ApproveMetadataDelegateHelperAccounts,
};

pub fn initialize_v2(
    ctx: Context<InitializeV2>,
    data: CandyMachineData,
    token_standard: u8,
) -> Result<()> {
    // make sure we got a valid token standard
    assert_token_standard(token_standard)?;

    let required_length = data.get_space_for_candy()?;

    if token_standard == TokenStandard::ProgrammableNonFungible as u8
        && ctx.accounts.candy_machine.data_len() < (required_length + RULE_SET_LENGTH + 1)
    {
        msg!("Allocating space to store the rule set");

        resize_or_reallocate_account_raw(
            &ctx.accounts.candy_machine.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            required_length + (1 + RULE_SET_LENGTH),
        )?;
    }

    let candy_machine_account = &mut ctx.accounts.candy_machine;

    let mut candy_machine = CandyMachine {
        data,
        version: AccountVersion::V2,
        token_standard,
        features: [0u8; 6],
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

    if token_standard == TokenStandard::ProgrammableNonFungible as u8 {
        if let Some(rule_set_info) = &ctx.accounts.rule_set {
            msg!("Storing rule set pubkey");

            let rule_set = rule_set_info.key();
            account_data[required_length] = SET;

            let index = required_length + 1;
            let mut storage = &mut account_data[index..index + RULE_SET_LENGTH];
            rule_set.serialize(&mut storage)?;
        }
    }

    // approves the metadata delegate so the candy machine can verify minted NFTs
    let delegate_accounts = ApproveMetadataDelegateHelperAccounts {
        authority_pda: ctx.accounts.authority_pda.to_account_info(),
        collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
        collection_mint: ctx.accounts.collection_mint.to_account_info(),
        collection_update_authority: ctx.accounts.collection_update_authority.to_account_info(),
        delegate_record: ctx.accounts.collection_delegate_record.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
        authorization_rules_program: ctx
            .accounts
            .authorization_rules_program
            .as_ref()
            .map(|authorization_rules_program| authorization_rules_program.to_account_info()),
        authorization_rules: ctx
            .accounts
            .authorization_rules
            .as_ref()
            .map(|authorization_rules| authorization_rules.to_account_info()),
    };

    approve_metadata_delegate(delegate_accounts)
}

/// Initializes a new candy machine.
#[derive(Accounts)]
#[instruction(data: CandyMachineData, token_standard: u8)]
pub struct InitializeV2<'info> {
    /// Candy Machine account. The account space must be allocated to allow accounts larger
    /// than 10kb.
    ///
    /// CHECK: account constraints checked in account trait
    #[account(
        zero,
        rent_exempt = skip,
        constraint = candy_machine.to_account_info().owner == program_id && candy_machine.to_account_info().data_len() >= data.get_space_for_candy()?
    )]
    candy_machine: UncheckedAccount<'info>,

    /// Authority PDA used to verify minted NFTs to the collection.
    ///
    /// CHECK: account checked in seeds constraint
    #[account(
        mut,
        seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    authority_pda: UncheckedAccount<'info>,

    /// Candy Machine authority. This is the address that controls the upate of the candy machine.
    ///
    /// CHECK: authority can be any account and is not written to or read
    authority: UncheckedAccount<'info>,

    /// Payer of the transaction.
    payer: Signer<'info>,

    /// Authorization rule set to be used by minted NFTs.
    ///
    /// CHECK: must be ownwed by mpl_token_auth_rules
    #[account(owner = mpl_token_auth_rules::id())]
    rule_set: Option<UncheckedAccount<'info>>,

    /// Metadata account of the collection.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_metadata: UncheckedAccount<'info>,

    /// Mint account of the collection.
    ///
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,

    /// Master Edition account of the collection.
    ///
    /// CHECK: account checked in CPI
    collection_master_edition: UncheckedAccount<'info>,

    /// Update authority of the collection. This needs to be a signer so the candy
    /// machine can approve a delegate to verify minted NFTs to the collection.
    #[account(mut)]
    collection_update_authority: Signer<'info>,

    /// Metadata delegate record. The delegate is used to verify NFTs.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_delegate_record: UncheckedAccount<'info>,

    /// Token Metadata program.
    ///
    /// CHECK: account constraint checked in account trait
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,

    /// System program.
    system_program: Program<'info, System>,

    /// Instructions sysvar account.
    ///
    /// CHECK: account constraint checked in account trait
    #[account(address = sysvar::instructions::id())]
    sysvar_instructions: UncheckedAccount<'info>,

    /// Token Authorization Rules program.
    ///
    /// CHECK: account constraint checked in account trait
    #[account(address = mpl_token_auth_rules::id())]
    authorization_rules_program: Option<UncheckedAccount<'info>>,

    /// Token Authorization rules account for the collection metadata (if any).
    ///
    /// CHECK: account checked in CPI
    #[account(owner = mpl_token_auth_rules::id())]
    authorization_rules: Option<UncheckedAccount<'info>>,
}
