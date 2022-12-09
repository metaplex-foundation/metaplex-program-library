use borsh::BorshSerialize;
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::{
    assertions::{assert_derivation, assert_owned_by},
    error::MetadataError,
    instruction::{DelegateArgs, DelegateRole},
    pda::PREFIX,
    state::{Delegate, Key, Metadata, TokenMetadataAccount},
};

/// Delegates an action over an asset to a specific account.
///
/// # Accounts:
///
///   0. `[writable]` Delegate account key
///   1. `[]` Delegated owner
///   2. `[signer]` Owner
///   3. `[signer, writable]` Payer
///   4. `[writable]` Token account
///   5. `[writable]` Metadata account
///   6. `[]` Mint account
///   7. `[]` System Program
///   8. `[]` Instructions sysvar account
///   9. `[]` SPL Token Program
///   10. `[optional]` Token Authorization Rules account
///   11. `[optional]` Token Authorization Rules program
pub fn delegate<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: DelegateArgs,
) -> ProgramResult {
    match args {
        DelegateArgs::V1 { .. } => delegate_v1(program_id, accounts, args),
    }
}

fn delegate_v1<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: DelegateArgs,
) -> ProgramResult {
    // get the accounts for the instruction
    let DelegateAccounts::V1 {
        delegate,
        delegate_owner,
        owner,
        payer,
        token,
        metadata,
        mint,
        system_program,
        ..
    } = args.get_accounts(accounts)?;
    // get the args for the instruction
    let DelegateArgs::V1 { role } = args;

    let mut asset_metadata = Metadata::from_account_info(metadata)?;
    assert_owned_by(metadata, program_id)?;
    assert_owned_by(mint, &spl_token::id())?;
    assert_signer(owner)?;
    assert_signer(payer)?;

    let token_account = Account::unpack(&token.try_borrow_data()?).unwrap();
    if token_account.owner != *owner.key {
        return Err(MetadataError::IncorrectOwner.into());
    }

    if asset_metadata.mint != *mint.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if role == DelegateRole::Sale {
        set_sale_delegate(
            program_id,
            &mut asset_metadata,
            delegate,
            delegate_owner,
            mint,
            owner,
            payer,
            system_program,
        )?;
    }

    asset_metadata.save(&mut metadata.try_borrow_mut_data()?)?;

    Ok(())
}

fn set_sale_delegate<'a>(
    program_id: &Pubkey,
    metadata: &mut Metadata,
    delegate: &'a AccountInfo<'a>,
    delegate_owner: &'a AccountInfo<'a>,
    mint: &'a AccountInfo<'a>,
    owner: &'a AccountInfo<'a>,
    payer: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    let role = DelegateRole::Sale.to_string();
    // validates the delegate derivation
    let mut delegate_seeds = vec![
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint.key.as_ref(),
        role.as_bytes(),
        delegate_owner.key.as_ref(),
        owner.key.as_ref(),
    ];
    let bump = &[assert_derivation(program_id, delegate, &delegate_seeds)?];

    delegate_seeds.push(bump);

    // allocate the delegate account

    create_or_allocate_account_raw(
        *program_id,
        delegate,
        system_program,
        payer,
        Delegate::size(),
        &delegate_seeds,
    )?;

    let mut delegate_account = Delegate::from_account_info(delegate)?;
    delegate_account.key = Key::Delegate;
    delegate_account.role = DelegateRole::Sale;
    delegate_account.bump = bump[0];
    delegate_account.serialize(&mut *delegate.try_borrow_mut_data()?)?;

    // save the delegate owner information

    if let Some(_delegate) = metadata.delegate {
        // revoke delegate
    }

    metadata.delegate = Some(*delegate_owner.key);

    Ok(())
}

enum DelegateAccounts<'a> {
    V1 {
        delegate: &'a AccountInfo<'a>,
        delegate_owner: &'a AccountInfo<'a>,
        owner: &'a AccountInfo<'a>,
        payer: &'a AccountInfo<'a>,
        token: &'a AccountInfo<'a>,
        metadata: &'a AccountInfo<'a>,
        mint: &'a AccountInfo<'a>,
        system_program: &'a AccountInfo<'a>,
        _sysvars: &'a AccountInfo<'a>,
        _spl_token_program: &'a AccountInfo<'a>,
        _authorization_rules: Option<&'a AccountInfo<'a>>,
        _auth_rules_program: Option<&'a AccountInfo<'a>>,
    },
}

impl DelegateArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<DelegateAccounts<'a>, ProgramError> {
        let account_info_iter = &mut accounts.iter();

        match *self {
            DelegateArgs::V1 { .. } => {
                let delegate = next_account_info(account_info_iter)?;
                let delegate_owner = next_account_info(account_info_iter)?;
                let owner = next_account_info(account_info_iter)?;
                let payer = next_account_info(account_info_iter)?;
                let token = next_account_info(account_info_iter)?;
                let metadata = next_account_info(account_info_iter)?;
                let mint = next_account_info(account_info_iter)?;
                let system_program = next_account_info(account_info_iter)?;
                let _sysvars = next_account_info(account_info_iter)?;
                let _spl_token_program = next_account_info(account_info_iter)?;

                let _authorization_rules =
                    if let Ok(authorization_rules) = next_account_info(account_info_iter) {
                        Some(authorization_rules)
                    } else {
                        None
                    };

                let _auth_rules_program =
                    if let Ok(auth_rules_program) = next_account_info(account_info_iter) {
                        Some(auth_rules_program)
                    } else {
                        None
                    };

                Ok(DelegateAccounts::V1 {
                    _sysvars,
                    _auth_rules_program,
                    _authorization_rules,
                    delegate,
                    delegate_owner,
                    metadata,
                    mint,
                    owner,
                    payer,
                    _spl_token_program,
                    system_program,
                    token,
                })
            }
        }
    }
}
