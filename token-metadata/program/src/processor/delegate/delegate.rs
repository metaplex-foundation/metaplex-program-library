use borsh::BorshSerialize;
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::{
    assertions::{
        assert_derivation, assert_owned_by, metadata::assert_update_authority_is_correct,
    },
    error::MetadataError,
    instruction::{Context, Delegate, DelegateArgs, DelegateRole},
    state::{DelegateRecord, DelegateState, Key, Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw},
};

/// Delegates an action over an asset to a specific account.
///
/// # Accounts:
///
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated owner
///   2. `[]` Mint account
///   3. `[writable]` Metadata account
///   4. `[optional]` Master Edition account
///   5. `[signer]` Authority to approve the delegation
///   6. `[signer, writable]` Payer
///   7. `[]` System Program
///   8. `[]` Instructions sysvar account
///   9. `[optional]` SPL Token Program
///   10. `[optional, writable]` Token account
///   11. `[optional]` Token Authorization Rules program
///   12. `[optional]` Token Authorization Rules account
pub fn delegate<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: DelegateArgs,
) -> ProgramResult {
    let context = Delegate::to_context(accounts)?;

    match args {
        DelegateArgs::CollectionV1 { .. } => delegate_collection_v1(program_id, context, args),
        DelegateArgs::SaleV1 { amount } => {
            // the sale delegate is a special type of transfer
            delegate_transfer_v1(program_id, context, args, DelegateRole::Sale, amount)
        }
        DelegateArgs::TransferV1 { amount } => {
            delegate_transfer_v1(program_id, context, args, DelegateRole::Transfer, amount)
        }
    }
}

/// Creates a `DelegateRole::Collection` delegate.
///
/// There can be multiple collections delegates set at any time.
fn delegate_collection_v1(
    program_id: &Pubkey,
    ctx: Context<Delegate>,
    _args: DelegateArgs,
) -> ProgramResult {
    // validates accounts

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;

    let asset_metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    assert_update_authority_is_correct(&asset_metadata, ctx.accounts.authority_info)?;

    if asset_metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    assert_signer(ctx.accounts.payer_info)?;
    assert_signer(ctx.accounts.authority_info)?;

    if !ctx.accounts.delegate_record_info.data_is_empty() {
        return Err(MetadataError::DelegateAlreadyExists.into());
    }

    // process the delegation creation (the derivation is checked
    // by the create helper)

    create_delegate(
        program_id,
        DelegateRole::Collection,
        ctx.accounts.delegate_record_info,
        ctx.accounts.delegate_info,
        ctx.accounts.mint_info,
        ctx.accounts.authority_info,
        ctx.accounts.payer_info,
        ctx.accounts.system_program_info,
    )
}

/// Creates a transfer-related delegate.
///
/// The delegate can only be either `DelegateRole::Sale` or `DelegateRole::Transfer`. Note
/// that `DelegateRole::Sale` is only available for programmable assets.
fn delegate_transfer_v1(
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
    assert_signer(ctx.accounts.authority_info)?;

    // transfer delegate must have a token account and spl token program
    let token_info = if let Some(token_info) = ctx.accounts.token_info {
        token_info
    } else {
        return Err(MetadataError::MissingTokenAccount.into());
    };

    let spl_token_program_info =
        if let Some(spl_token_program_info) = ctx.accounts.spl_token_program_info {
            spl_token_program_info
        } else {
            return Err(MetadataError::MissingSplTokenProgram.into());
        };

    let mut asset_metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    if asset_metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // authority must be the owner of the token account
    let token_account = Account::unpack(&token_info.try_borrow_data()?).unwrap();
    if token_account.owner != *ctx.accounts.authority_info.key {
        return Err(MetadataError::IncorrectOwner.into());
    }

    // process the delegation

    if matches!(
        asset_metadata.token_standard,
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
            ctx.accounts.authority_info.key,
            &[],
            amount,
        )?,
        &[
            token_info.clone(),
            ctx.accounts.delegate_info.clone(),
            ctx.accounts.authority_info.clone(),
        ],
    )?;

    if matches!(
        asset_metadata.token_standard,
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

    // we replace the existing delegate (if there is one)
    asset_metadata.delegate_state = Some(DelegateState {
        role,
        delegate: *ctx.accounts.delegate_info.key,
        has_data: false,
    });
    asset_metadata.save(&mut ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

    Ok(())
}

fn create_delegate<'a>(
    program_id: &Pubkey,
    delegate_role: DelegateRole,
    delegate: &'a AccountInfo<'a>,
    delegate_owner: &'a AccountInfo<'a>,
    mint: &'a AccountInfo<'a>,
    authority: &'a AccountInfo<'a>,
    payer: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    let role = delegate_role.to_string();
    // validates the delegate derivation
    let mut delegate_seeds = vec![
        mint.key.as_ref(),
        role.as_bytes(),
        delegate_owner.key.as_ref(),
        authority.key.as_ref(),
    ];
    let bump = &[assert_derivation(program_id, delegate, &delegate_seeds)?];

    delegate_seeds.push(bump);

    // allocate the delegate account

    create_or_allocate_account_raw(
        *program_id,
        delegate,
        system_program,
        payer,
        DelegateRecord::size(),
        &delegate_seeds,
    )?;

    let mut delegate_account = DelegateRecord::from_account_info(delegate)?;
    delegate_account.key = Key::Delegate;
    delegate_account.role = delegate_role;
    delegate_account.bump = bump[0];
    delegate_account.serialize(&mut *delegate.try_borrow_mut_data()?)?;

    Ok(())
}
