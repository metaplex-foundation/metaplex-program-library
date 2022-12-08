use mpl_utils::cmp_pubkeys;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::Mint;

use crate::{
    assertions::{
        assert_derivation, assert_initialized, assert_mint_authority_matches_mint, assert_owned_by,
    },
    error::MetadataError,
    instruction::MintArgs,
    pda::{EDITION, PREFIX},
    state::{Metadata, TokenMetadataAccount, TokenStandard},
};

/// Mints tokens from a mint account.
///
/// # Accounts:
///
///   0. `[writable`] Token account key
///   1. `[]` Metadata account key (pda of ['metadata', program id, mint id])")]
///   2. `[]` Mint of token asset
///   3. `[signer, writable]` Payer
///   4. `[signer]` Mint authority
///   5. `[]` System program
///   6. `[]` Instructions sysvar account
///   7. `[]` SPL Token program
///   8. `[]` SPL Associated Token Account program
///   9. `[optional]` Token Authorization Rules account
pub fn mint<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: MintArgs,
) -> ProgramResult {
    match args {
        MintArgs::V1 { .. } => mint_v1(program_id, accounts, args),
    }
}

pub fn mint_v1<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: MintArgs,
) -> ProgramResult {
    // get the accounts for the instruction
    let MintAccounts::V1 {
        token,
        metadata,
        mint,
        payer,
        spl_token_program,
        mint_authority,
        authorization_rules,
        ..
    } = args.get_accounts(accounts)?;
    // get the args for the instruction
    let MintArgs::V1 { amount } = args;

    let asset_metadata = Metadata::from_account_info(metadata)?;

    assert_owned_by(mint, &spl_token::id())?;
    let token_mint: Mint = assert_initialized(mint)?;

    // validates the mint

    assert_owned_by(metadata, program_id)?;
    assert_owned_by(mint, &spl_token::id())?;

    if asset_metadata.mint != *mint.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // validates the mint authority (NonFungible must have a "valid" master edition)

    match asset_metadata.token_standard {
        Some(TokenStandard::ProgrammableNonFungible) | Some(TokenStandard::NonFungible) => {
            // for NonFungible assets, the mint authority is the master edition
            assert_derivation(
                program_id,
                mint_authority,
                &[
                    PREFIX.as_bytes(),
                    program_id.as_ref(),
                    mint.key.as_ref(),
                    EDITION.as_bytes(),
                ],
            )?;

            if token_mint.supply > 0 || amount > 1 {
                return Err(MetadataError::EditionsMustHaveExactlyOneToken.into());
            }
        }
        _ => {
            assert_mint_authority_matches_mint(&token_mint.mint_authority, mint_authority)?;
        }
    }

    // validates authorization rules

    if let Some(TokenStandard::ProgrammableNonFungible) = asset_metadata.token_standard {
        if let Some(config) = asset_metadata.programmable_config {
            if let Some(rules) = authorization_rules {
                if !cmp_pubkeys(&config.rule_set, rules.key) {
                    // wrong authorization rules
                }
            } else {
                // missing authorization rules
            }
        }
    }

    // validates the ATA account

    assert_derivation(
        &spl_associated_token_account::id(),
        token,
        &[
            payer.key.as_ref(),
            spl_token::id().as_ref(),
            mint.key.as_ref(),
        ],
    )?;

    if token.data_is_empty() {
        // creating the associated token account
        invoke(
            &spl_associated_token_account::instruction::create_associated_token_account(
                payer.key,
                payer.key,
                mint.key,
                &spl_token::id(),
            ),
            &[payer.clone(), mint.clone(), token.clone()],
        )?;
    }

    msg!("Minting {} token(s) from mint {}", amount, mint.key);

    match asset_metadata.token_standard {
        Some(TokenStandard::NonFungible) | Some(TokenStandard::ProgrammableNonFungible) => {
            let mut signer_seeds = vec![
                PREFIX.as_bytes(),
                program_id.as_ref(),
                mint.key.as_ref(),
                EDITION.as_bytes(),
            ];

            let (master_edition, bump) = Pubkey::find_program_address(&signer_seeds, &crate::id());
            let bump_seed = [bump];
            signer_seeds.push(&bump_seed);

            invoke_signed(
                &spl_token::instruction::mint_to(
                    spl_token_program.key,
                    mint.key,
                    token.key,
                    &master_edition,
                    &[],
                    amount,
                )?,
                &[mint.clone(), token.clone(), mint_authority.clone()],
                &[&signer_seeds],
            )?;
        }
        _ => {
            invoke(
                &spl_token::instruction::mint_to(
                    spl_token_program.key,
                    mint.key,
                    token.key,
                    mint_authority.key,
                    &[],
                    amount,
                )?,
                &[mint.clone(), token.clone(), mint_authority.clone()],
            )?;
        }
    }

    Ok(())
}

enum MintAccounts<'a> {
    V1 {
        token: &'a AccountInfo<'a>,
        metadata: &'a AccountInfo<'a>,
        mint: &'a AccountInfo<'a>,
        payer: &'a AccountInfo<'a>,
        mint_authority: &'a AccountInfo<'a>,
        _system_program: &'a AccountInfo<'a>,
        _sysvars: &'a AccountInfo<'a>,
        spl_token_program: &'a AccountInfo<'a>,
        _spl_ata_program: &'a AccountInfo<'a>,
        authorization_rules: Option<&'a AccountInfo<'a>>,
    },
}

impl MintArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<MintAccounts<'a>, ProgramError> {
        let account_info_iter = &mut accounts.iter();

        match *self {
            MintArgs::V1 { .. } => {
                let token = next_account_info(account_info_iter)?;
                let metadata = next_account_info(account_info_iter)?;
                let mint = next_account_info(account_info_iter)?;
                let payer = next_account_info(account_info_iter)?;
                let mint_authority = next_account_info(account_info_iter)?;
                let _system_program = next_account_info(account_info_iter)?;
                let _sysvars = next_account_info(account_info_iter)?;
                let spl_token_program = next_account_info(account_info_iter)?;
                let _spl_ata_program = next_account_info(account_info_iter)?;

                let authorization_rules =
                    if let Ok(authorization_rules) = next_account_info(account_info_iter) {
                        Some(authorization_rules)
                    } else {
                        None
                    };

                Ok(MintAccounts::V1 {
                    _sysvars,
                    authorization_rules,
                    metadata,
                    mint,
                    mint_authority,
                    payer,
                    _spl_ata_program,
                    spl_token_program,
                    _system_program,
                    token,
                })
            }
        }
    }
}
