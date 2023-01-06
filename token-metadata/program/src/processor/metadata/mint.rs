use mpl_utils::{assert_signer, cmp_pubkeys};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::{Account, Mint as MintAccount};

use crate::{
    assertions::{
        assert_derivation, assert_initialized, assert_mint_authority_matches_mint, assert_owned_by,
    },
    error::MetadataError,
    instruction::{Context, Mint, MintArgs},
    pda::{EDITION, PREFIX},
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::{freeze, thaw},
};

/// Mints tokens from a mint account.
///
/// This instruction will also initialized the associated token account if it does not exist – in
/// this case the `token_owner` will be required. When minting `*NonFungible` assets, the `authority`
/// must be the update authority; in all other cases, it must be the mint authority from the mint
/// account.
pub fn mint<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: MintArgs,
) -> ProgramResult {
    let context = Mint::to_context(accounts)?;
    match args {
        MintArgs::V1 { .. } => mint_v1(program_id, context, args),
    }
}

pub fn mint_v1(program_id: &Pubkey, ctx: Context<Mint>, args: MintArgs) -> ProgramResult {
    // get the args for the instruction
    let MintArgs::V1 { amount, .. } = args;

    // checks that we have the required signers
    assert_signer(ctx.accounts.authority_info)?;
    assert_signer(ctx.accounts.payer_info)?;

    // validates the accounts

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    assert_owned_by(ctx.accounts.mint_info, &spl_token::id())?;
    let mint: MintAccount = assert_initialized(ctx.accounts.mint_info)?;

    if !cmp_pubkeys(ctx.accounts.spl_token_program_info.key, &spl_token::id()) {
        return Err(ProgramError::IncorrectProgramId);
    }

    // validate authorization rules
    /*
    if let Some(programmable_config) = &metadata.programmable_config {
        if let Some(auth_rules_program_info) = ctx.accounts.authorization_rules_program_info {
            if !cmp_pubkeys(auth_rules_program_info.key, &mpl_token_auth_rules::id()) {
                return Err(ProgramError::IncorrectProgramId);
            }
        } else {
            msg!("Missing authorization rules program account");
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        if let Some(authorization_rules) = ctx.accounts.authorization_rules_info {
            assert_owned_by(authorization_rules, &mpl_token_auth_rules::id())?;
        };

        assert_valid_authorization(ctx.accounts.authorization_rules_info, programmable_config)?;

        let mut auth_data = authorization_data.unwrap();

        // add the required input for the operation; since we are minting
        // new tokens to a specific address, we validate the operation as
        // a transfer operetion
        auth_data
            .payload
            .insert(PayloadKey::Amount, PayloadType::Number(amount));
        auth_data.payload.insert(
            PayloadKey::Target,
            PayloadType::Pubkey(*ctx.accounts.token_info.key),
        );

        validate(
            ctx.accounts.payer_info,
            auth_pda,
            Operation::MigrateClass,
            ctx.accounts.token_info,
            &auth_data,
        )?;
    }
    */

    // validates the authority:
    // - NonFungible must have a "valid" master edition
    // - Fungible must have the authority as the mint_authority

    match metadata.token_standard {
        Some(TokenStandard::ProgrammableNonFungible) | Some(TokenStandard::NonFungible) => {
            // for NonFungible assets, the mint authority is the master edition
            if let Some(master_edition_info) = ctx.accounts.master_edition_info {
                assert_derivation(
                    program_id,
                    master_edition_info,
                    &[
                        PREFIX.as_bytes(),
                        program_id.as_ref(),
                        ctx.accounts.mint_info.key.as_ref(),
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
            if !cmp_pubkeys(&metadata.update_authority, ctx.accounts.authority_info.key) {
                return Err(MetadataError::UpdateAuthorityIncorrect.into());
            }
        }
        _ => {
            assert_mint_authority_matches_mint(&mint.mint_authority, ctx.accounts.authority_info)?;
        }
    }

    // validates the token account

    if ctx.accounts.token_info.data_is_empty() {
        // if we are initializing a new account, we need the token_owner
        let token_owner_info = if let Some(token_owner_info) = ctx.accounts.token_owner_info {
            token_owner_info
        } else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // if the token account is empty, we will initialize a new one but it must
        // be a ATA account
        assert_derivation(
            &spl_associated_token_account::id(),
            ctx.accounts.token_info,
            &[
                token_owner_info.key.as_ref(),
                spl_token::id().as_ref(),
                ctx.accounts.mint_info.key.as_ref(),
            ],
        )?;

        msg!("Initializing associate token account");

        // creating the associated token account
        invoke(
            &spl_associated_token_account::instruction::create_associated_token_account(
                ctx.accounts.payer_info.key,
                token_owner_info.key,
                ctx.accounts.mint_info.key,
                &spl_token::id(),
            ),
            &[
                ctx.accounts.payer_info.clone(),
                token_owner_info.clone(),
                ctx.accounts.mint_info.clone(),
                ctx.accounts.token_info.clone(),
            ],
        )?;
    } else {
        assert_owned_by(ctx.accounts.token_info, &spl_token::id())?;
    }

    msg!(
        "Minting {} token(s) from mint {}",
        amount,
        ctx.accounts.mint_info.key
    );
    let token_account: Account = assert_initialized(ctx.accounts.token_info)?;

    match metadata.token_standard {
        Some(TokenStandard::NonFungible) | Some(TokenStandard::ProgrammableNonFungible) => {
            let mut signer_seeds = vec![
                PREFIX.as_bytes(),
                program_id.as_ref(),
                ctx.accounts.mint_info.key.as_ref(),
                EDITION.as_bytes(),
            ];

            let (master_edition_key, bump) =
                Pubkey::find_program_address(&signer_seeds, program_id);
            let bump_seed = [bump];
            signer_seeds.push(&bump_seed);

            let master_edition_info =
                if let Some(master_edition_info) = ctx.accounts.master_edition_info {
                    master_edition_info
                } else {
                    msg!("Missing master edition account");
                    return Err(ProgramError::NotEnoughAccountKeys);
                };

            if !cmp_pubkeys(master_edition_info.key, &master_edition_key) {
                return Err(MetadataError::InvalidMasterEdition.into());
            }

            // thaw the token account for programmable assets; the account
            // is not frozen if we just initialized it
            if matches!(
                metadata.token_standard,
                Some(TokenStandard::ProgrammableNonFungible)
            ) && token_account.is_frozen()
            {
                thaw(
                    ctx.accounts.mint_info.clone(),
                    ctx.accounts.token_info.clone(),
                    master_edition_info.clone(),
                    ctx.accounts.spl_token_program_info.clone(),
                )?;
            }

            invoke_signed(
                &spl_token::instruction::mint_to(
                    ctx.accounts.spl_token_program_info.key,
                    ctx.accounts.mint_info.key,
                    ctx.accounts.token_info.key,
                    &master_edition_key,
                    &[],
                    amount,
                )?,
                &[
                    ctx.accounts.mint_info.clone(),
                    ctx.accounts.token_info.clone(),
                    master_edition_info.clone(),
                ],
                &[&signer_seeds],
            )?;

            // programmable assets are always in a frozen state
            if matches!(
                metadata.token_standard,
                Some(TokenStandard::ProgrammableNonFungible)
            ) {
                freeze(
                    ctx.accounts.mint_info.clone(),
                    ctx.accounts.token_info.clone(),
                    master_edition_info.clone(),
                    ctx.accounts.spl_token_program_info.clone(),
                )?;
            }
        }
        _ => {
            invoke(
                &spl_token::instruction::mint_to(
                    ctx.accounts.spl_token_program_info.key,
                    ctx.accounts.mint_info.key,
                    ctx.accounts.token_info.key,
                    ctx.accounts.authority_info.key,
                    &[],
                    amount,
                )?,
                &[
                    ctx.accounts.mint_info.clone(),
                    ctx.accounts.token_info.clone(),
                    ctx.accounts.authority_info.clone(),
                ],
            )?;
        }
    }

    Ok(())
}
