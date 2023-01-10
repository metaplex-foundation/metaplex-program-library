use borsh::BorshSerialize;
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke, program_pack::Pack,
    pubkey::Pubkey, system_program, sysvar,
};
use spl_token::state::Account;

use crate::{
    assertions::{
        assert_derivation, assert_keys_equal, assert_owned_by,
        metadata::assert_update_authority_is_correct,
    },
    error::MetadataError,
    instruction::{Context, Delegate, DelegateArgs, DelegateRole},
    pda::PREFIX,
    state::{
        DelegateRecord, Key, Metadata, TokenMetadataAccount, TokenStandard, PERSISTENT_DELEGATE,
    },
    utils::{freeze, thaw},
};

/// Delegates an action over an asset to a specific account.
pub fn delegate<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: DelegateArgs,
) -> ProgramResult {
    let context = Delegate::to_context(accounts)?;

    match args {
        DelegateArgs::CollectionV1 { .. } => {
            create_delegate_v1(program_id, context, args, DelegateRole::Collection)
        }
        DelegateArgs::SaleV1 { amount, .. } => {
            // the sale delegate is a special type of transfer
            create_persistent_delegate_v1(program_id, context, args, DelegateRole::Sale, amount)
        }
        DelegateArgs::TransferV1 { amount, .. } => {
            create_persistent_delegate_v1(program_id, context, args, DelegateRole::Transfer, amount)
        }
        DelegateArgs::UpdateV1 { .. } => {
            create_delegate_v1(program_id, context, args, DelegateRole::Update)
        }
        DelegateArgs::UtilityV1 { amount, .. } => {
            create_persistent_delegate_v1(program_id, context, args, DelegateRole::Utility, amount)
        }
    }
}

/// Creates a `DelegateRole::Collection` delegate.
///
/// There can be multiple collections delegates set at any time.
fn create_delegate_v1(
    program_id: &Pubkey,
    ctx: Context<Delegate>,
    _args: DelegateArgs,
    role: DelegateRole,
) -> ProgramResult {
    // signers

    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.approver_info)?;

    // ownership

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;

    // key match

    assert_keys_equal(ctx.accounts.system_program_info.key, &system_program::ID)?;
    assert_keys_equal(ctx.accounts.sysvar_instructions_info.key, &sysvar::instructions::ID)?;

    // account relationships

    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    // authority must match update authority
    assert_update_authority_is_correct(&metadata, ctx.accounts.approver_info)?;

    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // process the delegation creation (the derivation is checked
    // by the create helper)

    let delegate_role = role.to_string();

    create_pda_account(
        program_id,
        DelegateRole::Collection,
        ctx.accounts.delegate_record_info,
        ctx.accounts.delegate_info,
        // delegate seeds
        vec![
            PREFIX.as_bytes(),
            program_id.as_ref(),
            ctx.accounts.mint_info.key.as_ref(),
            delegate_role.as_bytes(),
            ctx.accounts.approver_info.key.as_ref(),
            ctx.accounts.delegate_info.key.as_ref(),
        ],
        ctx.accounts.payer_info,
        ctx.accounts.system_program_info,
    )
}

/// Creates a presistent delegate.
///
/// Note that `DelegateRole::Sale` is only available for programmable assets.
fn create_persistent_delegate_v1(
    program_id: &Pubkey,
    ctx: Context<Delegate>,
    _args: DelegateArgs,
    role: DelegateRole,
    amount: u64,
) -> ProgramResult {
    // retrieving required optional accounts

    let token_info = match ctx.accounts.token_info {
        Some(token_info) => token_info,
        None => {
            return Err(MetadataError::MissingTokenAccount.into());
        }
    };

    let spl_token_program_info = match ctx.accounts.spl_token_program_info {
        Some(spl_token_program_info) => spl_token_program_info,
        None => {
            return Err(MetadataError::MissingSplTokenProgram.into());
        }
    };

    // signers

    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.approver_info)?;

    // ownership

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;
    assert_owned_by(token_info, &spl_token::id())?;

    // key match

    assert_keys_equal(ctx.accounts.system_program_info.key, &system_program::ID)?;
    assert_keys_equal(ctx.accounts.sysvar_instructions_info.key, &sysvar::instructions::ID)?;
    assert_keys_equal(spl_token_program_info.key, &spl_token::ID)?;

    // account relationships

    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if let Some(current_role) = metadata.persistent_delegate {
        // we only allow replacing a delegate if a sale delegate is not set, otherwise
        // the current delegate needs to be revoked first
        if matches!(current_role, DelegateRole::Sale) {
            return Err(MetadataError::DelegateAlreadyExists.into());
        }
    }

    // authority must be the owner of the token account
    let token_account = Account::unpack(&token_info.try_borrow_data()?).unwrap();
    if token_account.owner != *ctx.accounts.approver_info.key {
        return Err(MetadataError::IncorrectOwner.into());
    }

    // process the delegation

    if matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        if let Some(master_edition_info) = ctx.accounts.master_edition_info {
            assert_owned_by(master_edition_info, &crate::ID)?;
            // derivation is checked on the thaw function
            thaw(
                ctx.accounts.mint_info.clone(),
                token_info.clone(),
                master_edition_info.clone(),
                spl_token_program_info.clone(),
            )?;
        } else {
            return Err(MetadataError::MissingEditionAccount.into());
        }
    } else if matches!(role, DelegateRole::Sale) {
        // Sale delegate only available for programmable assets
        return Err(MetadataError::InvalidTokenStandard.into());
    }

    invoke(
        &spl_token::instruction::approve(
            spl_token_program_info.key,
            token_info.key,
            ctx.accounts.delegate_info.key,
            ctx.accounts.approver_info.key,
            &[],
            amount,
        )?,
        &[
            token_info.clone(),
            ctx.accounts.delegate_info.clone(),
            ctx.accounts.approver_info.clone(),
        ],
    )?;

    if matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        if let Some(master_edition_info) = ctx.accounts.master_edition_info {
            freeze(
                ctx.accounts.mint_info.clone(),
                token_info.clone(),
                master_edition_info.clone(),
                spl_token_program_info.clone(),
            )?;
        } else {
            // sanity check: this should not happen at this point since the master
            // edition account is validated before the delegation
            return Err(MetadataError::MissingEditionAccount.into());
        }
    }

    let delegate_seeds = vec![
        PREFIX.as_bytes(),
        program_id.as_ref(),
        ctx.accounts.mint_info.key.as_ref(),
        PERSISTENT_DELEGATE.as_bytes(),
        ctx.accounts.approver_info.key.as_ref(),
    ];

    // we create or replace the existing delegate (if there is one)
    if ctx.accounts.delegate_record_info.data_is_empty() {
        msg!("Creating delegate pda");
        create_pda_account(
            program_id,
            role,
            ctx.accounts.delegate_record_info,
            ctx.accounts.delegate_info,
            delegate_seeds,
            ctx.accounts.payer_info,
            ctx.accounts.system_program_info,
        )?;
    } else {
        assert_owned_by(ctx.accounts.delegate_record_info, &crate::ID)?;
        msg!("Updating delegate pda");
        // validates the delegate derivation
        assert_derivation(
            program_id,
            ctx.accounts.delegate_record_info,
            &delegate_seeds,
        )?;

        // updates the pda information
        let mut pda = DelegateRecord::from_account_info(ctx.accounts.delegate_record_info)?;
        pda.role = role;
        pda.delegate = *ctx.accounts.delegate_info.key;
        pda.serialize(&mut *ctx.accounts.delegate_record_info.try_borrow_mut_data()?)?;
    }

    metadata.persistent_delegate = Some(role);
    metadata.save(&mut ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

    Ok(())
}

fn create_pda_account<'a>(
    program_id: &Pubkey,
    delegate_role: DelegateRole,
    delegate_record_info: &'a AccountInfo<'a>,
    delegate_info: &'a AccountInfo<'a>,
    seeds: Vec<&[u8]>,
    payer_info: &'a AccountInfo<'a>,
    system_program_info: &'a AccountInfo<'a>,
) -> ProgramResult {
    // validates the delegate derivation

    let mut signer_seeds = seeds;
    let bump = &[assert_derivation(
        program_id,
        delegate_record_info,
        &signer_seeds,
    )?];
    signer_seeds.push(bump);

    if !delegate_record_info.data_is_empty() {
        return Err(MetadataError::DelegateAlreadyExists.into());
    }

    // allocate the delegate account

    create_or_allocate_account_raw(
        *program_id,
        delegate_record_info,
        system_program_info,
        payer_info,
        DelegateRecord::size(),
        &signer_seeds,
    )?;

    let mut pda = DelegateRecord::from_account_info(delegate_record_info)?;
    pda.key = Key::Delegate;
    pda.bump = bump[0];
    pda.role = delegate_role;
    pda.delegate = *delegate_info.key;
    pda.serialize(&mut *delegate_record_info.try_borrow_mut_data()?)?;

    Ok(())
}
