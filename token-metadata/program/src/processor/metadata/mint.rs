use mpl_token_auth_rules::payload::{PayloadKey, PayloadType};
use mpl_utils::{assert_signer, cmp_pubkeys};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::{Account, Mint};

use crate::{
    assertions::{
        assert_derivation, assert_initialized, assert_mint_authority_matches_mint, assert_owned_by,
        programmable::assert_valid_authorization,
    },
    error::MetadataError,
    instruction::MintArgs,
    pda::{EDITION, PREFIX},
    processor::next_optional_account_info,
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw},
};

/// Mints tokens from a mint account.
///
/// # Accounts:
///
///   0. `[writable`] Token account key
///   1. `[]` Metadata account key (pda of ['metadata', program id, mint id])")]
///   2. `[]` Mint of token asset
///   3. `[signer, writable]` Payer
///   4. `[signer]` Authority (mint authority or metadata's update authority for NonFungible asests)
///   5. `[]` System program
///   6. `[]` Instructions sysvar account
///   7. `[]` SPL Token program
///   8. `[]` SPL Associated Token Account program
///   9. `[optional]` Master Edition account
///   10. `[optional]` Token Authorization Rules program
///   11. `[optional]` Token Authorization Rules account
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
        token_info,
        metadata_info,
        mint_info,
        payer_info,
        spl_token_program_info,
        authority_info,
        master_edition_info,
        authorization_rules_info,
        auth_rules_program_info,
        ..
    } = args.get_accounts(accounts)?;
    // get the args for the instruction
    let MintArgs::V1 {
        amount,
        authorization_data,
    } = args;

    // checks that we have the required signers
    assert_signer(authority_info)?;
    assert_signer(payer_info)?;

    // validates the accounts

    assert_owned_by(metadata_info, program_id)?;
    let metadata = Metadata::from_account_info(metadata_info)?;

    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    assert_owned_by(mint_info, &spl_token::id())?;
    let mint: Mint = assert_initialized(mint_info)?;

    if !cmp_pubkeys(spl_token_program_info.key, &spl_token::id()) {
        return Err(ProgramError::IncorrectProgramId);
    }

    // validate authorization rules

    if let Some(programmable_config) = &metadata.programmable_config {
        if let Some(auth_rules_program_info) = auth_rules_program_info {
            if !cmp_pubkeys(auth_rules_program_info.key, &mpl_token_auth_rules::id()) {
                return Err(ProgramError::IncorrectProgramId);
            }
        } else {
            msg!("Missing authorization rules program account");
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        if let Some(authorization_rules) = authorization_rules_info {
            assert_owned_by(authorization_rules, &mpl_token_auth_rules::id())?;
        };

        assert_valid_authorization(
            &authorization_data,
            authorization_rules_info,
            programmable_config,
        )?;
        // safe to unwrap since the assert was valid
        // let auth_pda = authorization_rules_info.unwrap();
        let mut auth_data = authorization_data.unwrap();

        // add the required input for the operation; since we are minting
        // new tokens to a specific address, we validate the operation as
        // a transfer operetion
        auth_data
            .payload
            .insert(PayloadKey::Amount, PayloadType::Number(amount));
        auth_data
            .payload
            .insert(PayloadKey::Target, PayloadType::Pubkey(*token_info.key));
        /*
        validate(
            payer_info,
            auth_pda,
            Operation::MigrateClass,
            token_info,
            &auth_data,
        )?;
        */
    }

    // validates the authority
    // - NonFungible must have a "valid" master edition
    // - Fungible must have the authority as the mint_authority

    match metadata.token_standard {
        Some(TokenStandard::ProgrammableNonFungible) | Some(TokenStandard::NonFungible) => {
            // for NonFungible assets, the mint authority is the master edition
            if let Some(master_edition_info) = master_edition_info {
                assert_derivation(
                    program_id,
                    master_edition_info,
                    &[
                        PREFIX.as_bytes(),
                        program_id.as_ref(),
                        mint_info.key.as_ref(),
                        EDITION.as_bytes(),
                    ],
                )?;
            } else {
                return Err(MetadataError::InvalidMasterEdition.into());
            }

            if mint.supply > 0 || amount > 1 {
                return Err(MetadataError::EditionsMustHaveExactlyOneToken.into());
            }

            // authority must be the update_authority of the metadata account
            if !cmp_pubkeys(&metadata.update_authority, authority_info.key) {
                return Err(MetadataError::UpdateAuthorityIncorrect.into());
            }
        }
        _ => {
            assert_mint_authority_matches_mint(&mint.mint_authority, authority_info)?;
        }
    }

    // validates the ATA account

    assert_derivation(
        &spl_associated_token_account::id(),
        token_info,
        &[
            payer_info.key.as_ref(),
            spl_token::id().as_ref(),
            mint_info.key.as_ref(),
        ],
    )?;

    if token_info.data_is_empty() {
        // creating the associated token account
        invoke(
            &spl_associated_token_account::instruction::create_associated_token_account(
                payer_info.key,
                payer_info.key,
                mint_info.key,
                &spl_token::id(),
            ),
            &[payer_info.clone(), mint_info.clone(), token_info.clone()],
        )?;
    } else {
        assert_owned_by(token_info, &spl_token::id())?;
    }

    msg!("Minting {} token(s) from mint {}", amount, mint_info.key);
    let token_account: Account = assert_initialized(token_info)?;

    match metadata.token_standard {
        Some(TokenStandard::NonFungible) | Some(TokenStandard::ProgrammableNonFungible) => {
            let mut signer_seeds = vec![
                PREFIX.as_bytes(),
                program_id.as_ref(),
                mint_info.key.as_ref(),
                EDITION.as_bytes(),
            ];

            let (master_edition_key, bump) =
                Pubkey::find_program_address(&signer_seeds, &crate::id());
            let bump_seed = [bump];
            signer_seeds.push(&bump_seed);

            let master_edition_info = if let Some(master_edition_info) = master_edition_info {
                master_edition_info
            } else {
                msg!("Missing master edition account");
                return Err(ProgramError::NotEnoughAccountKeys);
            };

            if !cmp_pubkeys(master_edition_info.key, &master_edition_key) {
                return Err(MetadataError::InvalidMasterEdition.into());
            }

            if matches!(
                metadata.token_standard,
                Some(TokenStandard::ProgrammableNonFungible)
            ) && token_account.is_frozen()
            {
                // thaw the token account for programmable assets; the account
                // is not frozen if we just initialized it
                thaw(
                    mint_info,
                    token_info,
                    master_edition_info,
                    spl_token_program_info,
                )?;
            }

            invoke_signed(
                &spl_token::instruction::mint_to(
                    spl_token_program_info.key,
                    mint_info.key,
                    token_info.key,
                    &master_edition_key,
                    &[],
                    amount,
                )?,
                &[
                    mint_info.clone(),
                    token_info.clone(),
                    master_edition_info.clone(),
                ],
                &[&signer_seeds],
            )?;

            if matches!(
                metadata.token_standard,
                Some(TokenStandard::ProgrammableNonFungible)
            ) {
                // programmable assets are always in a frozen state
                freeze(
                    mint_info,
                    token_info,
                    master_edition_info,
                    spl_token_program_info,
                )?;
            }
        }
        _ => {
            invoke(
                &spl_token::instruction::mint_to(
                    spl_token_program_info.key,
                    mint_info.key,
                    token_info.key,
                    authority_info.key,
                    &[],
                    amount,
                )?,
                &[
                    mint_info.clone(),
                    token_info.clone(),
                    authority_info.clone(),
                ],
            )?;
        }
    }

    Ok(())
}

enum MintAccounts<'a> {
    V1 {
        token_info: &'a AccountInfo<'a>,
        metadata_info: &'a AccountInfo<'a>,
        mint_info: &'a AccountInfo<'a>,
        payer_info: &'a AccountInfo<'a>,
        authority_info: &'a AccountInfo<'a>,
        _system_program_info: &'a AccountInfo<'a>,
        _sysvars_info: &'a AccountInfo<'a>,
        spl_token_program_info: &'a AccountInfo<'a>,
        _spl_ata_program_info: &'a AccountInfo<'a>,
        master_edition_info: Option<&'a AccountInfo<'a>>,
        authorization_rules_info: Option<&'a AccountInfo<'a>>,
        auth_rules_program_info: Option<&'a AccountInfo<'a>>,
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
                let token_info = next_account_info(account_info_iter)?;
                let metadata_info = next_account_info(account_info_iter)?;
                let mint_info = next_account_info(account_info_iter)?;
                let payer_info = next_account_info(account_info_iter)?;
                let authority_info = next_account_info(account_info_iter)?;
                let _system_program_info = next_account_info(account_info_iter)?;
                let _sysvars_info = next_account_info(account_info_iter)?;
                let spl_token_program_info = next_account_info(account_info_iter)?;
                let _spl_ata_program_info = next_account_info(account_info_iter)?;
                // optional accounts
                let master_edition_info = next_optional_account_info(account_info_iter)?;
                let authorization_rules_info = next_optional_account_info(account_info_iter)?;
                let auth_rules_program_info = next_optional_account_info(account_info_iter)?;

                Ok(MintAccounts::V1 {
                    token_info,
                    metadata_info,
                    mint_info,
                    payer_info,
                    authority_info,
                    _system_program_info,
                    _sysvars_info,
                    spl_token_program_info,
                    _spl_ata_program_info,
                    master_edition_info,
                    authorization_rules_info,
                    auth_rules_program_info,
                })
            }
        }
    }
}
