use super::*;
use crate::processor::burn::nonfungible_edition::burn_nonfungible_edition;

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
    let BurnArgs::V1 {
        amount,
        authorization_data,
    } = args;

    // Validate accounts

    // Assert signer
    assert_signer(ctx.accounts.owner_info)?;

    // Assert program ownership.
    if let Some(collection_metadata_info) = ctx.accounts.collection_metadata_info {
        assert_owned_by(collection_metadata_info, program_id)?;
    }
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.edition_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;
    assert_owned_by(ctx.accounts.token_info, &spl_token::ID)?;

    if let Some(parent_edition) = ctx.accounts.parent_edition_info {
        assert_owned_by(parent_edition, program_id)?;
    }
    if let Some(parent_mint) = ctx.accounts.parent_mint_info {
        assert_owned_by(parent_mint, &spl_token::ID)?;
    }
    if let Some(parent_token) = ctx.accounts.parent_token_info {
        assert_owned_by(parent_token, &spl_token::ID)?;
    }

    if let Some(authorization_rules) = ctx.accounts.authorization_rules_info {
        assert_owned_by(authorization_rules, &mpl_token_auth_rules::ID)?;
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

    if let Some(auth_rules_program) = ctx.accounts.authorization_rules_program_info {
        if auth_rules_program.key != &mpl_token_auth_rules::ID {
            return Err(ProgramError::IncorrectProgramId);
        }
    }

    // Deserialize accounts.
    let metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

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
        ctx.accounts.owner_info,
        ctx.accounts.metadata_info,
        &metadata,
        ctx.accounts.mint_info,
        ctx.accounts.token_info,
    )?;

    let collection_metadata =
        if let Some(collection_metadata_info) = ctx.accounts.collection_metadata_info {
            Some(Metadata::from_account_info(collection_metadata_info)?)
        } else {
            None
        };

    if metadata.token_standard.is_none() {
        return Err(MetadataError::InvalidTokenStandard.into());
    }
    let token_standard = metadata.token_standard.unwrap();

    if matches!(
        token_standard,
        TokenStandard::NonFungibleEdition
            | TokenStandard::NonFungible
            | TokenStandard::ProgrammableNonFungible
    ) {
        if amount != 1 {
            return Err(MetadataError::InvalidAmount.into());
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

        _ => todo!(),
    }

    Ok(())
}
