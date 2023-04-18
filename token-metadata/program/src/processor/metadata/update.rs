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
    get_update_args_fields,
    instruction::{CollectionToggle, Context, MetadataDelegateRole, Update, UpdateArgs},
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
        UpdateArgs::V2 { .. } => update_v1(program_id, context, args),
    }
}

fn update_v1(program_id: &Pubkey, ctx: Context<Update>, args: UpdateArgs) -> ProgramResult {
    //** Account Validation **/
    // Assert signers

    // This account should always be a signer regardless of the authority type,
    // because at least one signer is required to update the metadata.
    assert_signer(ctx.accounts.authority_info)?;
    // Note that payer is not checked because it is not used.

    // Assert program ownership

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;
    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;

    if let Some(edition) = ctx.accounts.edition_info {
        assert_owned_by(edition, program_id)?;
        // checks that we got the correct edition account
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
    // to be passed in.  Note that we do NOT check the ownership of authorization rules
    // here as this allows `Update` to be used to correct a previously invalid `RuleSet`.
    if ctx.accounts.authorization_rules_info.is_some() {
        let authorization_rules_program = ctx
            .accounts
            .authorization_rules_program_info
            .ok_or(MetadataError::MissingAuthorizationRulesProgram)?;
        if authorization_rules_program.key != &mpl_token_auth_rules::ID {
            return Err(ProgramError::IncorrectProgramId);
        }
    }

    // Validate relationships

    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    // Mint must match metadata mint
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

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
    let existing_collection_mint = metadata
        .collection
        .as_ref()
        .map(|Collection { key, .. }| key);

    // Check if caller passed in a collection and if so use that.  Note that if the
    // delegate role from `get_authority_type` comes back as something other than
    // `MetadataDelegateRole::Collection` or `MetadataDelegateRole::CollectionItem`,
    // then it will fail in `validate_update` because those are the only roles that
    // can change collection.
    let collection_mint = match get_update_args_fields!(&args, collection).0 {
        CollectionToggle::Set(Collection { key, .. }) => Some(key),
        _ => existing_collection_mint,
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
        metadata_delegate_roles: vec![
            MetadataDelegateRole::Authority,
            MetadataDelegateRole::Data,
            MetadataDelegateRole::Collection,
            MetadataDelegateRole::CollectionItem,
            MetadataDelegateRole::ProgrammableConfig,
            MetadataDelegateRole::ProgrammableConfigItem,
        ],
        collection_metadata_delegate_roles: vec![
            MetadataDelegateRole::Collection,
            MetadataDelegateRole::ProgrammableConfig,
        ],
        precedence: &[
            AuthorityType::Metadata,
            AuthorityType::MetadataDelegate,
            AuthorityType::Holder,
        ],
        ..Default::default()
    })?;

    // Check if caller passed in a desired token standard.
    let desired_token_standard = match args {
        UpdateArgs::V1 { .. } => None,
        UpdateArgs::V2 { token_standard, .. } => token_standard,
    };

    // Validate that authority has permission to update the fields that have been specified in the
    // update args.
    validate_update(
        &args,
        &authority_type,
        metadata_delegate_role,
        desired_token_standard,
    )?;

    // Find existing token standard from metadata or infer it.
    let existing_or_inferred_token_standard = if let Some(token_standard) = metadata.token_standard
    {
        token_standard
    } else {
        check_token_standard(ctx.accounts.mint_info, ctx.accounts.edition_info)?
    };

    // If there is a desired token standard, use it if it passes the check.  If there is not a
    // desired token standard, use the existing or inferred token standard.
    let token_standard = match desired_token_standard {
        Some(desired_token_standard) => {
            check_desired_token_standard(
                existing_or_inferred_token_standard,
                desired_token_standard,
            )?;
            desired_token_standard
        }
        None => existing_or_inferred_token_standard,
    };

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

    // If we reach here without errors we have validated that the authority is allowed to
    // perform an update.
    metadata.update_v1(
        args,
        ctx.accounts.authority_info,
        ctx.accounts.metadata_info,
        token,
        token_standard,
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
    desired_token_standard: Option<TokenStandard>,
) -> ProgramResult {
    // validate the authority type
    match authority_type {
        AuthorityType::Metadata => {
            // metadata authority is the paramount (update) authority
            msg!("Auth type: Metadata");
        }
        AuthorityType::Holder => {
            // support for holder update
            msg!("Auth type: Holder");
            return Err(MetadataError::FeatureNotSupported.into());
        }
        AuthorityType::MetadataDelegate => {
            // support for delegate update
            msg!("Auth type: Delegate");
        }
        _ => return Err(MetadataError::InvalidAuthorityType.into()),
    }

    // Destructure args.
    let (
        new_update_authority,
        data,
        primary_sale_happened,
        is_mutable,
        collection,
        collection_details,
        uses,
        rule_set,
    ) = get_update_args_fields!(
        args,
        new_update_authority,
        data,
        primary_sale_happened,
        is_mutable,
        collection,
        collection_details,
        uses,
        rule_set
    );

    // validate the delegate role: this consist in checking that
    // the delegate is only updating fields that it has access to
    if let Some(metadata_delegate_role) = metadata_delegate_role {
        match metadata_delegate_role {
            MetadataDelegateRole::Authority => {
                // Fields allowed for `Authority`:
                // `new_update_authority`
                // `primary_sale_happened`
                // `is_mutable`
                // `token_standard`
                if data.is_some()
                    || collection.is_some()
                    || collection_details.is_some()
                    || uses.is_some()
                    || rule_set.is_some()
                {
                    return Err(MetadataError::InvalidUpdateArgs.into());
                }
            }
            MetadataDelegateRole::Collection | MetadataDelegateRole::CollectionItem => {
                // Fields allowed for `Collection` and `CollectionItem`:
                // `collection`
                if new_update_authority.is_some()
                    || data.is_some()
                    || primary_sale_happened.is_some()
                    || is_mutable.is_some()
                    || collection_details.is_some()
                    || uses.is_some()
                    || rule_set.is_some()
                    || desired_token_standard.is_some()
                {
                    return Err(MetadataError::InvalidUpdateArgs.into());
                }
            }
            MetadataDelegateRole::Data => {
                // Fields allowed for `Data`:
                // `data`
                if new_update_authority.is_some()
                    || primary_sale_happened.is_some()
                    || is_mutable.is_some()
                    || collection.is_some()
                    || collection_details.is_some()
                    || uses.is_some()
                    || rule_set.is_some()
                    || desired_token_standard.is_some()
                {
                    return Err(MetadataError::InvalidUpdateArgs.into());
                }
            }
            MetadataDelegateRole::ProgrammableConfig
            | MetadataDelegateRole::ProgrammableConfigItem => {
                // Fields allowed for `ProgrammableConfig` and `ProgrammableConfigItem`:
                // `rule_set`
                if new_update_authority.is_some()
                    || data.is_some()
                    || primary_sale_happened.is_some()
                    || is_mutable.is_some()
                    || collection.is_some()
                    || collection_details.is_some()
                    || uses.is_some()
                    || desired_token_standard.is_some()
                {
                    return Err(MetadataError::InvalidUpdateArgs.into());
                }
            }
            _ => return Err(MetadataError::InvalidAuthorityType.into()),
        }
    }

    Ok(())
}

fn check_desired_token_standard(
    existing_or_inferred_token_standard: TokenStandard,
    desired_token_standard: TokenStandard,
) -> ProgramResult {
    // This function only allows switching between Fungible and FungibleAsset.  Mint decimals must
    // be zero.
    match existing_or_inferred_token_standard {
        TokenStandard::Fungible | TokenStandard::FungibleAsset => match desired_token_standard {
            TokenStandard::Fungible | TokenStandard::FungibleAsset => Ok(()),
            _ => Err(MetadataError::InvalidTokenStandard.into()),
        },
        _ => Err(MetadataError::InvalidTokenStandard.into()),
    }
}
