use solana_program::program_option::COption;

use super::*;

use crate::{
    pda::find_token_record_account,
    processor::burn::{fungible::burn_fungible, nonfungible_edition::burn_nonfungible_edition},
    state::{AuthorityRequest, AuthorityType, TokenDelegateRole, TokenRecord, TokenState},
    utils::{check_token_standard, thaw},
};

/// Burn an asset, closing associated accounts.
///
/// Supports burning the following asset types:
/// - ProgrammableNonFungible
/// - NonFungible
/// - NonFungigbleEdition
/// - Fungible
/// - FungibleAsset
///
/// Parent accounts only required for burning print editions are the accounts for the master edition
/// associated with the print edition.
/// The Token Record account is required for burning a ProgrammableNonFungible asset.
///
/// This handler closes the following accounts:
///
/// For ProgrammableNonFungible assets:
/// - Metadata, Edition, Token, TokenRecord
///
/// For NonFungible assets:
/// - Metadata, Edition, Token
///
/// For NonFungibleEdition assets:
/// - Metadata, Edition, Token, and the EditionMarker, if all prints for it are burned.
///
/// For Fungible assets:
/// - Only the token account, if all tokens are burned.
pub fn burn<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: BurnArgs,
) -> ProgramResult {
    let context = Burn::to_context(accounts)?;

    match args {
        BurnArgs::V1 { .. } => burn_v1(program_id, context, args),
    }
}

// V1 implementation of the burn instruction.
fn burn_v1(program_id: &Pubkey, ctx: Context<Burn>, args: BurnArgs) -> ProgramResult {
    let BurnArgs::V1 { amount } = args;

    // Validate accounts

    // Assert signer
    assert_signer(ctx.accounts.authority_info)?;

    // Assert program ownership.
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;
    assert_owned_by(ctx.accounts.token_info, &spl_token::ID)?;

    if let Some(edition_info) = ctx.accounts.edition_info {
        assert_owned_by(edition_info, program_id)?;
    }
    if let Some(master_edition) = ctx.accounts.master_edition_info {
        assert_owned_by(master_edition, program_id)?;
    }
    if let Some(master_edition_mint) = ctx.accounts.master_edition_mint_info {
        assert_owned_by(master_edition_mint, &spl_token::ID)?;
    }
    if let Some(master_edition_token) = ctx.accounts.master_edition_token_info {
        assert_owned_by(master_edition_token, &spl_token::ID)?;
    }
    if let Some(edition_marker) = ctx.accounts.edition_marker_info {
        assert_owned_by(edition_marker, program_id)?;
    }
    if let Some(token_record) = ctx.accounts.token_record_info {
        assert_owned_by(token_record, program_id)?;
    }

    // Check program IDs.
    if ctx.accounts.system_program_info.key != &system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if ctx.accounts.sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if ctx.accounts.spl_token_program_info.key != &spl_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize accounts.
    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    let token: TokenAccount = assert_initialized(ctx.accounts.token_info)?;

    let authority_response = AuthorityType::get_authority_type(AuthorityRequest {
        authority: ctx.accounts.authority_info.key,
        update_authority: &metadata.update_authority,
        mint: ctx.accounts.mint_info.key,
        token: Some(ctx.accounts.token_info.key),
        token_account: Some(&token),
        token_record_info: ctx.accounts.token_record_info,
        token_delegate_roles: vec![TokenDelegateRole::Utility],
        precedence: &[AuthorityType::Holder, AuthorityType::TokenDelegate],
        ..Default::default()
    })?;

    // Must be either the holder or a token delegate.
    if !matches!(
        authority_response.authority_type,
        AuthorityType::Holder | AuthorityType::TokenDelegate
    ) {
        return Err(MetadataError::InvalidAuthorityType.into());
    }

    // Validate relationships between accounts.

    // Mint account passed in matches the mint of the token account.
    if &token.mint != ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // Token account must have sufficient balance for burn.
    if token.amount < amount {
        return Err(MetadataError::InsufficientTokenBalance.into());
    }

    // Metadata account must match the mint.
    if token.mint != metadata.mint {
        return Err(MetadataError::MintMismatch.into());
    }

    let token_standard = if let Some(token_standard) = metadata.token_standard {
        token_standard
    } else {
        check_token_standard(ctx.accounts.mint_info, ctx.accounts.edition_info)?
    };

    // NonFungible types can only burn one item and must have the edition
    // account present.
    if matches!(
        token_standard,
        TokenStandard::NonFungibleEdition
            | TokenStandard::NonFungible
            | TokenStandard::ProgrammableNonFungible
            | TokenStandard::ProgrammableNonFungibleEdition
    ) {
        if amount != 1 {
            return Err(MetadataError::InvalidAmount.into());
        }

        if ctx.accounts.edition_info.is_none() {
            return Err(MetadataError::MissingEdition.into());
        }
    } else if amount < 1 {
        return Err(MetadataError::InvalidAmount.into());
    }

    match token_standard {
        TokenStandard::NonFungible => {
            let args = BurnNonFungibleArgs {
                metadata,
                me_close_authority: false,
            };

            burn_nonfungible(&ctx, args)?;
        }
        TokenStandard::NonFungibleEdition => {
            burn_nonfungible_edition(&ctx, false, &TokenStandard::NonFungibleEdition)?;
        }
        TokenStandard::ProgrammableNonFungible => {
            let token_record_info = ctx
                .accounts
                .token_record_info
                .ok_or(MetadataError::MissingTokenRecord)?;

            // All the checks are the same as burning a NonFungible token
            // except we also have to check the token state and derivation.
            let (pda_key, _) =
                find_token_record_account(ctx.accounts.mint_info.key, ctx.accounts.token_info.key);

            if pda_key != *token_record_info.key {
                return Err(MetadataError::InvalidTokenRecord.into());
            }

            let token_record = TokenRecord::from_account_info(token_record_info)?;

            // Locked and Listed states cannot be burned.
            if token_record.state != TokenState::Unlocked {
                return Err(MetadataError::IncorrectTokenState.into());
            }

            let edition_info = ctx
                .accounts
                .edition_info
                .ok_or(MetadataError::MissingEditionAccount)?;

            thaw(
                ctx.accounts.mint_info.clone(),
                ctx.accounts.token_info.clone(),
                edition_info.clone(),
                ctx.accounts.spl_token_program_info.clone(),
            )?;

            let mut args = BurnNonFungibleArgs {
                metadata,
                me_close_authority: false,
            };

            // Utility Delegate is the only delegate that can burn an asset.
            if let Some(TokenDelegateRole::Utility) = token_record.delegate_role {
                if let COption::Some(close_authority) = token.close_authority {
                    if &close_authority != edition_info.key {
                        return Err(MetadataError::InvalidCloseAuthority.into());
                    }
                    args.me_close_authority = true;
                }
            }

            burn_nonfungible(&ctx, args)?;

            // Also close the token_record account.
            close_program_account(
                &token_record_info.clone(),
                &ctx.accounts.authority_info.clone(),
                Key::TokenRecord,
            )?;
        }
        TokenStandard::ProgrammableNonFungibleEdition => {
            let token_record_info = ctx
                .accounts
                .token_record_info
                .ok_or(MetadataError::MissingTokenRecord)?;

            // All the checks are the same as burning a NonFungible token
            // except we also have to check the token state and derivation.
            let (pda_key, _) =
                find_token_record_account(ctx.accounts.mint_info.key, ctx.accounts.token_info.key);

            if pda_key != *token_record_info.key {
                return Err(MetadataError::InvalidTokenRecord.into());
            }

            let token_record = TokenRecord::from_account_info(token_record_info)?;

            // Locked and Listed states cannot be burned.
            if token_record.state != TokenState::Unlocked {
                return Err(MetadataError::IncorrectTokenState.into());
            }

            let edition_info = ctx
                .accounts
                .edition_info
                .ok_or(MetadataError::MissingEditionAccount)?;

            thaw(
                ctx.accounts.mint_info.clone(),
                ctx.accounts.token_info.clone(),
                edition_info.clone(),
                ctx.accounts.spl_token_program_info.clone(),
            )?;

            let mut is_close_auth = false;
            // Utility Delegate is the only delegate that can burn an asset.
            if let Some(TokenDelegateRole::Utility) = token_record.delegate_role {
                if let COption::Some(close_authority) = token.close_authority {
                    if &close_authority != edition_info.key {
                        return Err(MetadataError::InvalidCloseAuthority.into());
                    }
                    is_close_auth = true;
                }
            }

            burn_nonfungible_edition(
                &ctx,
                is_close_auth,
                &TokenStandard::ProgrammableNonFungibleEdition,
            )?;

            // Also close the token_record account.
            close_program_account(
                &token_record_info.clone(),
                &ctx.accounts.authority_info.clone(),
                Key::TokenRecord,
            )?;
        }
        TokenStandard::Fungible | TokenStandard::FungibleAsset => {
            burn_fungible(&ctx, amount)?;
        }
    }

    Ok(())
}
