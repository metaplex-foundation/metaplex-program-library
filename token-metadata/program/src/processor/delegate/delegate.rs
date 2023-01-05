use borsh::BorshSerialize;
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, program_pack::Pack,
    pubkey::Pubkey,
    msg,
};
use spl_token::state::Account;

use crate::{
    assertions::{
        assert_derivation, assert_owned_by, metadata::assert_update_authority_is_correct,
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
        DelegateArgs::CollectionV1 { .. } => collection_delegate_v1(program_id, context, args),
        DelegateArgs::SaleV1 { amount } => {
            // the sale delegate is a special type of transfer
            sale_or_transfer_delegate_v1(program_id, context, args, DelegateRole::Sale, amount)
        }
        DelegateArgs::TransferV1 { amount } => {
            sale_or_transfer_delegate_v1(program_id, context, args, DelegateRole::Transfer, amount)
        }
    }
}

/// Creates a `DelegateRole::Collection` delegate.
///
/// There can be multiple collections delegates set at any time.
fn collection_delegate_v1(
    program_id: &Pubkey,
    ctx: Context<Delegate>,
    _args: DelegateArgs,
) -> ProgramResult {
    // validates accounts

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;

    let asset_metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    assert_update_authority_is_correct(&asset_metadata, ctx.accounts.namespace_info)?;

    if asset_metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.namespace_info)?;

    if !ctx.accounts.delegate_record_info.data_is_empty() {
        return Err(MetadataError::DelegateAlreadyExists.into());
    }

    // process the delegation creation (the derivation is checked
    // by the create helper)

    let role = DelegateRole::Collection.to_string();

    create_delegate_pda(
        program_id,
        DelegateRole::Collection,
        ctx.accounts.delegate_record_info,
        ctx.accounts.delegate_info,
        // delegate seeds
        vec![
            PREFIX.as_bytes(),
            program_id.as_ref(),
            ctx.accounts.mint_info.key.as_ref(),
            role.as_bytes(),
            ctx.accounts.namespace_info.key.as_ref(),
            ctx.accounts.delegate_info.key.as_ref(),
        ],
        ctx.accounts.payer_info,
        ctx.accounts.system_program_info,
    )
}

/// Creates a presistent delegate.
///
/// Note that `DelegateRole::Sale` is only available for programmable assets.
fn sale_or_transfer_delegate_v1(
    program_id: &Pubkey,
    ctx: Context<Delegate>,
    _args: DelegateArgs,
    role: DelegateRole,
    amount: u64,
) -> ProgramResult {
    // validates accounts

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;
    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.namespace_info)?;

    // sale or transfer delegates are always authorised by the token owner
    let token_info = if let Some(token_info) = ctx.accounts.token_info {
        token_info
    } else {
        return Err(MetadataError::MissingTokenAccount.into());
    };

    // and must have a token account and spl token program
    let spl_token_program_info =
        if let Some(spl_token_program_info) = ctx.accounts.spl_token_program_info {
            spl_token_program_info
        } else {
            return Err(MetadataError::MissingSplTokenProgram.into());
        };

    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // authority must be the owner of the token account
    let token_account = Account::unpack(&token_info.try_borrow_data()?).unwrap();
    if token_account.owner != *ctx.accounts.namespace_info.key {
        return Err(MetadataError::IncorrectOwner.into());
    }

    // process the delegation

    if matches!(
        metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        if let Some(master_edition_info) = ctx.accounts.master_edition_info {
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
            ctx.accounts.namespace_info.key,
            &[],
            amount,
        )?,
        &[
            token_info.clone(),
            ctx.accounts.delegate_info.clone(),
            ctx.accounts.namespace_info.clone(),
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
            return Err(MetadataError::MissingEditionAccount.into());
        }
    }

    let delegate_seeds = vec![
        PREFIX.as_bytes(),
        program_id.as_ref(),
        ctx.accounts.mint_info.key.as_ref(),
        PERSISTENT_DELEGATE.as_bytes(),
        ctx.accounts.namespace_info.key.as_ref(),
    ];

    // we create or replace the existing delegate (if there is one)
    if ctx.accounts.delegate_record_info.data_is_empty() {
        msg!("Creating delegate pda");
        create_delegate_pda(
            program_id,
            role,
            ctx.accounts.delegate_record_info,
            ctx.accounts.delegate_info,
            delegate_seeds,
            ctx.accounts.payer_info,
            ctx.accounts.system_program_info,
        )?;
    } else {
        msg!("Updating delegate pda");
        // validates the delegate derivation
        assert_derivation(program_id, ctx.accounts.delegate_record_info, &delegate_seeds)?;

        // updates the pda information
        let mut pda = DelegateRecord::from_account_info(ctx.accounts.delegate_record_info)?;
        pda.role = role;
        pda.delegate = *ctx.accounts.delegate_info.key;
        pda.serialize(&mut *ctx.accounts.delegate_record_info.try_borrow_mut_data()?)?;
    }

    Ok(())
}

fn create_delegate_pda<'a>(
    program_id: &Pubkey,
    delegate_role: DelegateRole,
    delegate_record: &'a AccountInfo<'a>,
    delegate: &'a AccountInfo<'a>,
    seeds: Vec<&[u8]>,
    payer: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    // validates the delegate derivation

    let mut signer_seeds = seeds;
    let bump = &[assert_derivation(program_id, delegate_record, &signer_seeds)?];
    signer_seeds.push(bump);

    // allocate the delegate account

    create_or_allocate_account_raw(
        *program_id,
        delegate_record,
        system_program,
        payer,
        DelegateRecord::size(),
        &signer_seeds,
    )?;

    let mut pda = DelegateRecord::from_account_info(delegate_record)?;
    pda.key = Key::Delegate;
    pda.bump = bump[0];
    pda.role = delegate_role;
    pda.delegate = *delegate.key;
    pda.serialize(&mut *delegate_record.try_borrow_mut_data()?)?;

    Ok(())
}
