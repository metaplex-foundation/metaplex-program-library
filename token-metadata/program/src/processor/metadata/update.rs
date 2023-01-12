use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, sysvar,
};

use crate::{
    assertions::{assert_owned_by, programmable::assert_valid_authorization},
    error::MetadataError,
    instruction::{Context, MetadataDelegateRole, Update, UpdateArgs},
    pda::{EDITION, PREFIX},
    state::{
        AuthorityRequest, AuthorityType, Metadata, ProgrammableConfig, TokenMetadataAccount,
        TokenStandard,
    },
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

    if let Some(edition) = ctx.accounts.edition_info {
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
    // authorization rules
    if let Some(authorization_rules) = ctx.accounts.authorization_rules_info {
        assert_owned_by(authorization_rules, &mpl_token_auth_rules::ID)?;
    }
    // token owner
    if let Some(holder_token_account) = ctx.accounts.token_info {
        assert_owned_by(holder_token_account, &spl_token::ID)?;
    }
    // delegate
    if let Some(delegate_record_info) = ctx.accounts.delegate_record_info {
        assert_owned_by(delegate_record_info, &crate::ID)?;
    }

    // Check program IDs

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

    // Validate relationships

    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    // Mint must match metadata mint
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // Determines if we have a valid authority to perform the update. This must
    // be either the update authority, a delegate or the holder. This call fails
    // if no valid authority is present.
    let authority_type = AuthorityType::get_authority_type(AuthorityRequest {
        authority: ctx.accounts.authority_info.key,
        update_authority: &metadata.update_authority,
        mint: ctx.accounts.mint_info.key,
        token_info: ctx.accounts.token_info,
        metadata_delegate_record_info: ctx.accounts.delegate_record_info,
        metadata_delegate_role: Some(MetadataDelegateRole::Update),
        token_record_info: None,
        token_delegate_role: None,
    })?;

    let token_standard = metadata
        .token_standard
        .ok_or(MetadataError::InvalidTokenStandard)?;

    // for pNFTs, we need to validate the authorization rules
    if matches!(token_standard, TokenStandard::ProgrammableNonFungible) {
        if let Some(config) = &metadata.programmable_config {
            // if we have a programmable rule set
            if let ProgrammableConfig::V1 { rule_set: Some(_) } = config {
                assert_valid_authorization(ctx.accounts.authorization_rules_info, config)?;
            }
        }
    }

    match authority_type {
        AuthorityType::Metadata => {
            // Metadata authority is the paramount authority so is not subject to
            // auth rules. At this point we already checked that the authority is a
            // signer and that it matches the metadata's update authority.
            msg!("Authority type: Metadata");
        }
        AuthorityType::Delegate => {
            // Support for delegate update (for pNFTs this involves validating the
            // authoritzation rules)
            msg!("Authority type: Delegate");
            return Err(MetadataError::FeatureNotSupported.into());
        }
        AuthorityType::Holder => {
            // Support for holder update (for pNFTs this involves validating the
            // authoritzation rules)
            msg!("Authority type: Holder");
            return Err(MetadataError::FeatureNotSupported.into());
        }
        AuthorityType::None => {
            return Err(MetadataError::UpdateAuthorityIncorrect.into());
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
