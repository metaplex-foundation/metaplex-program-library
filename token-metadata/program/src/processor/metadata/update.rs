use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, sysvar,
};

use crate::{
    assertions::{assert_owned_by, metadata::assert_metadata_authority},
    error::MetadataError,
    instruction::{AuthorityType, Context, Update, UpdateArgs},
    pda::{EDITION, PREFIX},
    state::{Metadata, TokenMetadataAccount, TokenStandard},
    utils::assert_derivation,
};

pub fn update<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: UpdateArgs,
) -> ProgramResult {
    let context = Update::to_context(accounts)?;

    match args {
        UpdateArgs::V1 { .. } => update_v1(program_id, context, args),
    }
}

fn update_v1(program_id: &Pubkey, ctx: Context<Update>, args: UpdateArgs) -> ProgramResult {
    //** Account Validation **/
    // Assert signers
    // This account should always be a signer regardless of the authority type,
    // because at least one signer is required to update the metadata.
    assert_signer(ctx.accounts.authority_info)?;

    // Assert program ownership
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;

    if let Some(edition) = ctx.accounts.master_edition_info {
        assert_owned_by(edition, program_id)?;
        // checks that we got the correct master account
        assert_derivation(
            program_id,
            edition,
            &[
                PREFIX.as_bytes(),
                program_id.as_ref(),
                ctx.accounts.mint_info.key.as_ref(),
                EDITION.as_bytes(),
            ],
        )?;
    }
    if let Some(authorization_rules) = ctx.accounts.authorization_rules_info {
        assert_owned_by(authorization_rules, &mpl_token_auth_rules::ID)?;
    }
    if let Some(holder_token_account) = ctx.accounts.token_info {
        assert_owned_by(holder_token_account, &spl_token::ID)?;
    }

    // Check program IDs.
    if ctx.accounts.system_program_info.key != &solana_program::system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if ctx.accounts.sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if ctx.accounts.authorization_rules_info.is_some() {
        if let Some(authorization_rules_program) = ctx.accounts.authorization_rules_program_info {
            if authorization_rules_program.key != &mpl_token_auth_rules::ID {
                return Err(ProgramError::IncorrectProgramId);
            }
        } else {
            return Err(MetadataError::MissingAuthorizationRulesProgram.into());
        }
    }

    // Deserialize metadata to determine its type
    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

    // Validate relationships

    // Mint must match metadata mint
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    let token_standard = metadata
        .token_standard
        .ok_or(MetadataError::InvalidTokenStandard)?;

    // We have to validate authorization rules if the token standard is a Programmable type
    // and the metadata has a programmable config set.
    let _auth_rules_apply = matches!(token_standard, TokenStandard::ProgrammableNonFungible)
        && metadata.programmable_config.is_some();

    match args.get_authority_type() {
        AuthorityType::Metadata => {
            // Check is signer and matches update authority on metadata.
            assert_metadata_authority(&metadata, ctx.accounts.authority_info)?;

            // Metadata authority is the paramount authority so is not subject to auth rules.
            // Exit the branch with no error and proceed to update.
        }
        AuthorityType::Delegate => {
            // If delegate_record_opt_info is None, then return an error.
            // Otherwise, validate delegate_record PDA derivation
            // seeds: mint, "update", authority as delegate, metadata.update_authority
            //
            // if auth_rules_apply:
            // we can unwrap programmable config
            // call the auth rules function to perform all auth rules checks
            //     - get authorization data
            //     - assert_valid_authorization
            //     - add required payload values
            //     - call validate function
            // if the auth rules function returns with no error, then we
            // exit this branch with no error
            //
            // else: it's a valid update delegate so we exit this branch with no error

            return Err(MetadataError::FeatureNotSupported.into());
        }
        AuthorityType::Holder => {
            // Rules TBD
            // Ensure that the holder token account is passed in
            // Ensure owner is currently holding the token and is the proper owner
            // Ensure mint matches metadata
            return Err(MetadataError::FeatureNotSupported.into());
        }
        AuthorityType::Other => {
            // Rules TBD, additional authority type for supporting PayloadKey::Target
            // and support flexible authorization rules.
            // This one should fail without authorization rules set as there is no valid
            // update without it.
            return Err(MetadataError::FeatureNotSupported.into());
        }
    }

    // If we reach here without errors we have validated that the authority is allowed to
    // perform an update.
    metadata.update_v1(
        args,
        ctx.accounts.authority_info,
        ctx.accounts.metadata_info,
    )?;

    Ok(())
}
