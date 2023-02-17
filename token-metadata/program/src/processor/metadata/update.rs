use std::fmt::{Display, Formatter};

use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey, sysvar,
};
use spl_token::state::Account;

use crate::{
    assertions::{assert_owned_by, programmable::assert_valid_authorization},
    error::MetadataError,
    instruction::{Context, MetadataDelegateRole, Update, UpdateArgs},
    pda::{EDITION, PREFIX},
    state::{
        AuthorityRequest, AuthorityResponse, AuthorityType, Collection, Metadata,
        ProgrammableConfig, TokenMetadataAccount, TokenStandard,
    },
    utils::{assert_derivation, check_token_standard},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateScenario {
    MetadataAuth,
    Delegate,
    Proxy,
}

impl Display for UpdateScenario {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateScenario::MetadataAuth => write!(f, "MetadataAuth"),
            UpdateScenario::Delegate => write!(f, "Delegate"),
            UpdateScenario::Proxy => write!(f, "Proxy"),
        }
    }
}

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

    // If the current rule set is passed in, also require the mpl-token-auth-rules program
    // to be passed in.
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

    let token_standard = if let Some(token_standard) = metadata.token_standard {
        token_standard
    } else {
        check_token_standard(ctx.accounts.mint_info, ctx.accounts.edition_info)?
    };

    let (token_pubkey, token) = if let Some(token_info) = ctx.accounts.token_info {
        (
            Some(token_info.key),
            Some(Account::unpack(&token_info.try_borrow_data()?)?),
        )
    } else {
        (None, None)
    };

    // there is a special case for collection-level delegates, where the
    // validation should use the collection key as the mint parameter
    let collection_mint = if let Some(Collection { key, .. }) = &metadata.collection {
        Some(key)
    } else {
        None
    };

    // Determines if we have a valid authority to perform the update. This must
    // be either the update authority, a delegate or the holder. This call fails
    // if no valid authority is present.
    let AuthorityResponse {
        authority_type,
        metadata_delegate_role,
        ..
    } = AuthorityType::get_authority_type(AuthorityRequest {
        authority: ctx.accounts.authority_info.key,
        update_authority: &metadata.update_authority,
        mint: ctx.accounts.mint_info.key,
        collection_mint,
        token: token_pubkey,
        token_account: token.as_ref(),
        metadata_delegate_record_info: ctx.accounts.delegate_record_info,
        metadata_delegate_roles: vec![MetadataDelegateRole::ProgrammableConfig],
        precedence: &[
            AuthorityType::Metadata,
            AuthorityType::MetadataDelegate,
            AuthorityType::Holder,
        ],
        ..Default::default()
    })?;

    // For pNFTs, we need to validate the authorization rules.
    if matches!(token_standard, TokenStandard::ProgrammableNonFungible) {
        // If the metadata account has a current rule set, we validate that
        // the current rule set account is passed in and matches value on the
        // metadata.
        if let Some(config) = &metadata.programmable_config {
            // if we have a programmable rule set
            if let ProgrammableConfig::V1 { rule_set: Some(_) } = config {
                assert_valid_authorization(ctx.accounts.authorization_rules_info, config)?;
            }
        }
    }

    validate_update(&args, &authority_type, metadata_delegate_role)?;

    // If we reach here without errors we have validated that the authority is allowed to
    // perform an update.
    metadata.update_v1(
        args,
        ctx.accounts.authority_info,
        ctx.accounts.metadata_info,
        token,
        Some(token_standard),
        authority_type,
        metadata_delegate_role,
    )?;

    Ok(())
}

/// Validates that the authority is only updating metadata fields
/// that it has access to.
fn validate_update(
    args: &UpdateArgs,
    authority_type: &AuthorityType,
    metadata_delegate_role: Option<MetadataDelegateRole>,
) -> ProgramResult {
    // validate the authority type

    match authority_type {
        AuthorityType::Metadata => {
            // metadata authority is the paramount (upadte) authority
            msg!("Auth type: Metadata");
        }
        AuthorityType::MetadataDelegate => {
            // support for delegate update
            msg!("Auth type: Delegate");
        }
        AuthorityType::Holder => {
            // support for holder update
            msg!("Auth type: Holder");
            return Err(MetadataError::FeatureNotSupported.into());
        }
        _ => {
            return Err(MetadataError::InvalidAuthorityType.into());
        }
    }

    let UpdateArgs::V1 {
        data,
        primary_sale_happened,
        is_mutable,
        collection,
        uses,
        new_update_authority,
        collection_details,
        ..
    } = args;

    // validate the delegate role: this consist in checking that
    // the delegate is only updating fields that it has access to
    match metadata_delegate_role {
        Some(MetadataDelegateRole::ProgrammableConfig) => {
            // can only update the programmable config
            if data.is_some()
                || primary_sale_happened.is_some()
                || is_mutable.is_some()
                || collection.is_some()
                || uses.is_some()
                || new_update_authority.is_some()
                || collection_details.is_some()
            {
                return Err(MetadataError::InvalidUpdateArgs.into());
            }
        }
        Some(_) => {
            return Err(MetadataError::InvalidAuthorityType.into());
        }
        None => { /* no delegate role to check */ }
    }

    Ok(())
}
