use super::*;
use crate::{
    processor::burn::{fungible::burn_fungible, nonfungible_edition::burn_nonfungible_edition},
    state::{AuthorityRequest, AuthorityType, TokenDelegateRole, TokenRecord, TokenState},
    utils::thaw,
};

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

fn burn_v1(program_id: &Pubkey, ctx: Context<Burn>, args: BurnArgs) -> ProgramResult {
    msg!("Burn V1");
    let BurnArgs::V1 { amount } = args;

    // Validate accounts

    // Assert signer
    assert_signer(ctx.accounts.authority_info)?;

    // Assert program ownership.
    if let Some(collection_metadata_info) = ctx.accounts.collection_metadata_info {
        assert_owned_by(collection_metadata_info, program_id)?;
    }
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;
    assert_owned_by(ctx.accounts.token_info, &spl_token::ID)?;

    if let Some(edition_info) = ctx.accounts.edition_info {
        assert_owned_by(edition_info, program_id)?;
    }

    if let Some(parent_edition) = ctx.accounts.parent_edition_info {
        assert_owned_by(parent_edition, program_id)?;
    }
    if let Some(parent_mint) = ctx.accounts.parent_mint_info {
        assert_owned_by(parent_mint, &spl_token::ID)?;
    }
    if let Some(parent_token) = ctx.accounts.parent_token_info {
        assert_owned_by(parent_token, &spl_token::ID)?;
    }

    // Check program IDs.
    if ctx.accounts.spl_token_program_info.key != &spl_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if ctx.accounts.system_program_info.key != &system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if ctx.accounts.sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize accounts.
    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    let token: TokenAccount = assert_initialized(ctx.accounts.token_info)?;

    let authority_type = AuthorityType::get_authority_type(AuthorityRequest {
        authority: ctx.accounts.authority_info.key,
        update_authority: &metadata.update_authority,
        mint: ctx.accounts.mint_info.key,
        token: Some(ctx.accounts.token_info.key),
        token_account: Some(&token),
        token_record_info: ctx.accounts.token_record_info,
        token_delegate_roles: vec![TokenDelegateRole::Utility],
        precedence: &[AuthorityType::Holder, AuthorityType::Delegate],
        ..Default::default()
    })?;

    match authority_type {
        AuthorityType::Holder => {
            // Checks:
            // * Metadata is owned by the token-metadata program
            // * Mint is owned by the spl-token program
            // * Token is owned by the spl-token program
            // * Token account is initialized
            // * Token account data owner is 'owner'
            // * Token account belongs to mint
            // * Token account has 1 or more tokens
            // * Mint matches metadata.mint
            assert_currently_holding(
                &crate::ID,
                ctx.accounts.authority_info,
                ctx.accounts.metadata_info,
                &metadata,
                ctx.accounts.mint_info,
                ctx.accounts.token_info,
            )?;
        }
        AuthorityType::Delegate => {
            if &token.mint != ctx.accounts.mint_info.key {
                return Err(MetadataError::MintMismatch.into());
            }

            if token.amount < amount {
                return Err(MetadataError::NotEnoughTokens.into());
            }

            if token.mint != metadata.mint {
                return Err(MetadataError::MintMismatch.into());
            }
        }
        _ => return Err(MetadataError::InvalidAuthorityType.into()),
    }

    let collection_metadata = ctx
        .accounts
        .collection_metadata_info
        .map(|r| Metadata::from_account_info(r))
        .transpose()?;

    if metadata.token_standard.is_none() {
        return Err(MetadataError::InvalidTokenStandard.into());
    }
    let token_standard = metadata.token_standard.unwrap();

    // NonFungible types can only burn one item and must have the edition
    // account present.
    if matches!(
        token_standard,
        TokenStandard::NonFungibleEdition
            | TokenStandard::NonFungible
            | TokenStandard::ProgrammableNonFungible
    ) {
        if amount != 1 {
            return Err(MetadataError::InvalidAmount.into());
        }

        if ctx.accounts.edition_info.is_none() {
            return Err(MetadataError::MissingEdition.into());
        }
    }

    match token_standard {
        TokenStandard::NonFungible => {
            let args = BurnNonFungibleArgs {
                collection_metadata,
                metadata,
            };

            burn_nonfungible(&ctx, args)?;
        }
        TokenStandard::NonFungibleEdition => {
            burn_nonfungible_edition(&ctx)?;
        }
        TokenStandard::ProgrammableNonFungible => {
            // All the checks are the same as burning a NonFungible token
            // except we also have to check the token state.
            let token_record = ctx
                .accounts
                .token_record_info
                .map(|r| TokenRecord::from_account_info(r))
                .transpose()?;

            if token_record.is_none() {
                return Err(MetadataError::MissingTokenRecord.into());
            }
            let token_record = token_record.unwrap();

            // Locked and Listed states cannot be burned.
            if token_record.state != TokenState::Unlocked {
                return Err(MetadataError::IncorrectTokenState.into());
            }

            thaw(
                ctx.accounts.mint_info.clone(),
                ctx.accounts.token_info.clone(),
                ctx.accounts.edition_info.unwrap().clone(),
                ctx.accounts.spl_token_program_info.clone(),
            )?;

            let args = BurnNonFungibleArgs {
                collection_metadata,
                metadata,
            };

            burn_nonfungible(&ctx, args)?;
        }
        TokenStandard::Fungible | TokenStandard::FungibleAsset => {
            burn_fungible(&ctx, amount)?;
        }
    }

    Ok(())
}
