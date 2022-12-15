use mpl_utils::{assert_signer, close_account_raw, cmp_pubkeys};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, program_option::COption, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::{
    assertions::{
        assert_derivation, assert_owned_by, metadata::assert_update_authority_is_correct,
    },
    error::MetadataError,
    instruction::{DelegateRole, RevokeArgs},
    processor::{try_get_account_info, try_get_optional_account_info},
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw},
};

// Number of expected accounts in the instruction (including optional accounts).
const EXPECTED_ACCOUNTS_LEN: usize = 13;

/// Revoke a delegation of the token.
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
pub fn revoke<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: RevokeArgs,
) -> ProgramResult {
    match args {
        RevokeArgs::CollectionV1 => revoke_collection_v1(program_id, accounts, args),
        RevokeArgs::SaleV1 => revoke_sale_v1(program_id, accounts, args),
    }
}

fn revoke_collection_v1<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: RevokeArgs,
) -> ProgramResult {
    let (delegate, delegate_owner, mint, metadata, authority, payer) =
        if let RevokeAccounts::CollectionV1 {
            delegate,
            delegate_owner,
            mint,
            metadata,
            authority,
            payer,
            _system_program,
            _sysvars,
            _authorization_rules,
            _auth_rules_program,
        } = args.get_accounts(accounts)?
        {
            (delegate, delegate_owner, mint, metadata, authority, payer)
        } else {
            unimplemented!();
        };

    // validates accounts

    assert_owned_by(metadata, program_id)?;
    assert_owned_by(mint, &spl_token::id())?;

    let asset_metadata = Metadata::from_account_info(metadata)?;
    assert_update_authority_is_correct(&asset_metadata, authority)?;

    if asset_metadata.mint != *mint.key {
        return Err(MetadataError::MintMismatch.into());
    }

    assert_signer(payer)?;
    assert_signer(authority)?;

    if delegate.data_is_empty() {
        return Err(MetadataError::Uninitialized.into());
    }

    // process the delegation creation (the derivation is checked
    // by the create helper)

    revoke_delegate(
        program_id,
        DelegateRole::Collection,
        delegate,
        delegate_owner,
        mint,
        authority,
        payer,
    )
}

fn revoke_sale_v1<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: RevokeArgs,
) -> ProgramResult {
    // get the accounts for the instruction
    let (
        delegate_owner,
        mint,
        metadata,
        master_edition,
        authority,
        payer,
        spl_token_program,
        token,
    ) = if let RevokeAccounts::SaleV1 {
        delegate_owner,
        mint,
        metadata,
        master_edition,
        authority,
        payer,
        spl_token_program,
        token,
        ..
    } = args.get_accounts(accounts)?
    {
        (
            delegate_owner,
            mint,
            metadata,
            master_edition,
            authority,
            payer,
            spl_token_program,
            token,
        )
    } else {
        unimplemented!();
    };

    // validates accounts

    assert_owned_by(metadata, program_id)?;
    assert_owned_by(mint, &spl_token::id())?;
    assert_signer(payer)?;

    let mut asset_metadata = Metadata::from_account_info(metadata)?;
    if asset_metadata.mint != *mint.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if let Some(existing) = &asset_metadata.delegate {
        if !cmp_pubkeys(existing, delegate_owner.key) {
            return Err(MetadataError::InvalidDelegate.into());
        }
    } else {
        return Err(MetadataError::DelegateNotFound.into());
    }

    // authority must be the owner of the token account
    let token_account = Account::unpack(&token.try_borrow_data()?).unwrap();
    if token_account.owner != *authority.key {
        return Err(MetadataError::IncorrectOwner.into());
    }

    if let COption::Some(existing) = &token_account.delegate {
        if !cmp_pubkeys(existing, delegate_owner.key) {
            return Err(MetadataError::InvalidDelegate.into());
        }
    } else {
        return Err(MetadataError::DelegateNotFound.into());
    }

    // and must be a signer of the transaction
    assert_signer(authority)?;

    // process the delegation

    if matches!(
        asset_metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        if let Some(master_edition) = master_edition {
            thaw(mint, token, master_edition, spl_token_program)?;
        } else {
            return Err(MetadataError::MissingEditionAccount.into());
        }
    }

    invoke(
        &spl_token::instruction::revoke(spl_token_program.key, token.key, authority.key, &[])?,
        &[token.clone(), delegate_owner.clone(), authority.clone()],
    )?;

    if matches!(
        asset_metadata.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        if let Some(master_edition) = master_edition {
            freeze(mint, token, master_edition, spl_token_program)?;
        } else {
            return Err(MetadataError::MissingEditionAccount.into());
        }
    }

    // sale delegate is set to the metadata account
    asset_metadata.delegate = None;
    asset_metadata.save(&mut metadata.try_borrow_mut_data()?)?;

    Ok(())
}

fn revoke_delegate<'a>(
    program_id: &Pubkey,
    delegate_role: DelegateRole,
    delegate: &'a AccountInfo<'a>,
    delegate_owner: &'a AccountInfo<'a>,
    mint: &'a AccountInfo<'a>,
    owner: &'a AccountInfo<'a>,
    payer: &'a AccountInfo<'a>,
) -> ProgramResult {
    let role = delegate_role.to_string();
    // validates the delegate derivation
    let delegate_seeds = vec![
        mint.key.as_ref(),
        role.as_bytes(),
        delegate_owner.key.as_ref(),
        owner.key.as_ref(),
    ];
    assert_derivation(program_id, delegate, &delegate_seeds)?;
    // closes the delegate account
    close_account_raw(payer, delegate)
}

enum RevokeAccounts<'a> {
    CollectionV1 {
        delegate: &'a AccountInfo<'a>,
        delegate_owner: &'a AccountInfo<'a>,
        mint: &'a AccountInfo<'a>,
        metadata: &'a AccountInfo<'a>,
        authority: &'a AccountInfo<'a>,
        payer: &'a AccountInfo<'a>,
        _system_program: &'a AccountInfo<'a>,
        _sysvars: &'a AccountInfo<'a>,
        _authorization_rules: Option<&'a AccountInfo<'a>>,
        _auth_rules_program: Option<&'a AccountInfo<'a>>,
    },
    SaleV1 {
        _delegate: &'a AccountInfo<'a>,
        delegate_owner: &'a AccountInfo<'a>,
        mint: &'a AccountInfo<'a>,
        metadata: &'a AccountInfo<'a>,
        master_edition: Option<&'a AccountInfo<'a>>,
        authority: &'a AccountInfo<'a>,
        payer: &'a AccountInfo<'a>,
        _system_program: &'a AccountInfo<'a>,
        _sysvars: &'a AccountInfo<'a>,
        spl_token_program: &'a AccountInfo<'a>,
        token: &'a AccountInfo<'a>,
        _authorization_rules: Option<&'a AccountInfo<'a>>,
        _auth_rules_program: Option<&'a AccountInfo<'a>>,
    },
}

impl RevokeArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<RevokeAccounts<'a>, ProgramError> {
        // validates that we got the correct number of accounts
        if accounts.len() < EXPECTED_ACCOUNTS_LEN {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        match *self {
            RevokeArgs::CollectionV1 { .. } => {
                let delegate = try_get_account_info(accounts, 0)?;
                let delegate_owner = try_get_account_info(accounts, 1)?;
                let mint = try_get_account_info(accounts, 2)?;
                let metadata = try_get_account_info(accounts, 3)?;
                let authority = try_get_account_info(accounts, 5)?;
                let payer = try_get_account_info(accounts, 6)?;
                let _system_program = try_get_account_info(accounts, 7)?;
                let _sysvars = try_get_account_info(accounts, 8)?;
                // optional accounts
                let _authorization_rules = try_get_optional_account_info(accounts, 11)?;
                let _auth_rules_program = try_get_optional_account_info(accounts, 12)?;

                Ok(RevokeAccounts::CollectionV1 {
                    delegate,
                    delegate_owner,
                    mint,
                    metadata,
                    authority,
                    payer,
                    _system_program,
                    _sysvars,
                    _authorization_rules,
                    _auth_rules_program,
                })
            }
            RevokeArgs::SaleV1 { .. } => {
                let _delegate = try_get_account_info(accounts, 0)?;
                let delegate_owner = try_get_account_info(accounts, 1)?;
                let mint = try_get_account_info(accounts, 2)?;
                let metadata = try_get_account_info(accounts, 3)?;
                let master_edition = try_get_optional_account_info(accounts, 4)?;
                let authority = try_get_account_info(accounts, 5)?;
                let payer = try_get_account_info(accounts, 6)?;
                let _system_program = try_get_account_info(accounts, 7)?;
                let _sysvars = try_get_account_info(accounts, 8)?;
                let spl_token_program = try_get_account_info(accounts, 9)?;
                let token = try_get_account_info(accounts, 10)?;
                // optional accounts
                let _authorization_rules = try_get_optional_account_info(accounts, 11)?;
                let _auth_rules_program = try_get_optional_account_info(accounts, 12)?;

                Ok(RevokeAccounts::SaleV1 {
                    _delegate,
                    delegate_owner,
                    mint,
                    metadata,
                    master_edition,
                    authority,
                    payer,
                    _system_program,
                    _sysvars,
                    spl_token_program,
                    token,
                    _authorization_rules,
                    _auth_rules_program,
                })
            }
        }
    }
}
