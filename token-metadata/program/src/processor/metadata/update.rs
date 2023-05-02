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
    instruction::{
        CollectionDetailsToggle, CollectionToggle, Context, MetadataDelegateRole, Update,
        UpdateArgs, UsesToggle,
    },
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

    update_v1(program_id, context, args)
}

fn update_v1(program_id: &Pubkey, ctx: Context<Update>, args: UpdateArgs) -> ProgramResult {
    // Assert signers

    // Authority should always be a signer regardless of the authority type,
    // because at least one signer is required to update the metadata.
    assert_signer(ctx.accounts.authority_info)?;
    assert_signer(ctx.accounts.payer_info)?;

    // Assert program ownership

    if let Some(delegate_record_info) = ctx.accounts.delegate_record_info {
        assert_owned_by(delegate_record_info, &crate::ID)?;
    }

    if let Some(token_info) = ctx.accounts.token_info {
        assert_owned_by(token_info, &spl_token::ID)?;
    }

    assert_owned_by(ctx.accounts.mint_info, &spl_token::ID)?;
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;

    if let Some(edition) = ctx.accounts.edition_info {
        assert_owned_by(edition, program_id)?;
    }

    // Note that we do NOT check the ownership of authorization rules account here as this allows
    // `Update` to be used to correct a previously invalid `RuleSet`.  In practice the ownership of
    // authorization rules is checked by the Auth Rules program each time the program is invoked to
    // validate rules.

    // Check program IDs

    if ctx.accounts.system_program_info.key != &solana_program::system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if ctx.accounts.sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // If the current rule set is passed in, also require the mpl-token-auth-rules program
    // to be passed in (and check its program ID).
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

    // Token
    let (token_pubkey, token) = if let Some(token_info) = ctx.accounts.token_info {
        let token = Account::unpack(&token_info.try_borrow_data()?)?;

        // Token mint must match mint account key.  Token amount must be greater than 0.
        if token.mint != *ctx.accounts.mint_info.key {
            return Err(MetadataError::MintMismatch.into());
        } else if token.amount == 0 {
            return Err(MetadataError::AmountMustBeGreaterThanZero.into());
        }

        (Some(token_info.key), Some(token))
    } else {
        (None, None)
    };

    // Metadata
    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    // Metadata mint must match mint account key.
    if metadata.mint != *ctx.accounts.mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // Edition
    if let Some(edition) = ctx.accounts.edition_info {
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

    // Check authority.

    // There is a special case for collection-level delegates, where the
    // validation should use the collection key as the mint parameter.
    let existing_collection_mint = metadata
        .collection
        .as_ref()
        .map(|Collection { key, .. }| key);

    // Check if caller passed in a collection and if so use that.  Note that
    // `validate_update` checks that the authority has permission to pass in
    // a new collection value.
    let collection_mint = match &args {
        UpdateArgs::V1 { collection, .. }
        | UpdateArgs::AsUpdateAuthorityV2 { collection, .. }
        | UpdateArgs::AsCollectionDelegateV2 { collection, .. }
        | UpdateArgs::AsCollectionItemDelegateV2 { collection, .. } => match collection {
            CollectionToggle::Set(Collection { key, .. }) => Some(key),
            _ => existing_collection_mint,
        },
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
            MetadataDelegateRole::AuthorityItem,
            MetadataDelegateRole::Data,
            MetadataDelegateRole::DataItem,
            MetadataDelegateRole::Collection,
            MetadataDelegateRole::CollectionItem,
            MetadataDelegateRole::ProgrammableConfig,
            MetadataDelegateRole::ProgrammableConfigItem,
        ],
        collection_metadata_delegate_roles: vec![
            MetadataDelegateRole::Data,
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

    // Validate that authority has permission to use the update args that were provided.
    validate_update(&args, &authority_type, metadata_delegate_role)?;

    // See if caller passed in a desired token standard.
    let desired_token_standard = match args {
        UpdateArgs::AsUpdateAuthorityV2 { token_standard, .. }
        | UpdateArgs::AsAuthorityItemDelegateV2 { token_standard, .. } => token_standard,
        _ => None,
    };

    // Find existing token standard from metadata or infer it.
    let existing_or_inferred_token_std = if let Some(token_standard) = metadata.token_standard {
        token_standard
    } else {
        check_token_standard(ctx.accounts.mint_info, ctx.accounts.edition_info)?
    };

    // If there is a desired token standard, use it if it passes the check.  If there is not a
    // desired token standard, use the existing or inferred token standard.
    let token_standard = match desired_token_standard {
        Some(desired_token_standard) => {
            check_desired_token_standard(existing_or_inferred_token_std, desired_token_standard)?;
            desired_token_standard
        }
        None => existing_or_inferred_token_std,
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

    // validate the delegate role: this consist in checking that
    // the delegate is only updating fields that it has access to
    if let Some(metadata_delegate_role) = metadata_delegate_role {
        let valid_delegate_update = match (metadata_delegate_role, args) {
            (MetadataDelegateRole::AuthorityItem, UpdateArgs::AsAuthorityItemDelegateV2 { .. }) => {
                true
            }
            (MetadataDelegateRole::Data, UpdateArgs::AsDataDelegateV2 { .. }) => true,
            (MetadataDelegateRole::DataItem, UpdateArgs::AsDataItemDelegateV2 { .. }) => true,
            (MetadataDelegateRole::Collection, UpdateArgs::AsCollectionDelegateV2 { .. }) => true,
            (
                MetadataDelegateRole::CollectionItem,
                UpdateArgs::AsCollectionItemDelegateV2 { .. },
            ) => true,
            (
                // V1 supported Programmable config, leaving here for backwards
                // compatibility.
                MetadataDelegateRole::ProgrammableConfig,
                UpdateArgs::V1 {
                    new_update_authority: None,
                    data: None,
                    primary_sale_happened: None,
                    is_mutable: None,
                    collection: CollectionToggle::None,
                    collection_details: CollectionDetailsToggle::None,
                    uses: UsesToggle::None,
                    ..
                },
            ) => true,
            (
                MetadataDelegateRole::ProgrammableConfig,
                UpdateArgs::AsProgrammableConfigDelegateV2 { .. },
            ) => true,
            (
                MetadataDelegateRole::ProgrammableConfigItem,
                UpdateArgs::AsProgrammableConfigItemDelegateV2 { .. },
            ) => true,
            _ => false,
        };

        if !valid_delegate_update {
            return Err(MetadataError::InvalidUpdateArgs.into());
        }
    }

    Ok(())
}

fn check_desired_token_standard(
    existing_or_inferred_token_std: TokenStandard,
    desired_token_standard: TokenStandard,
) -> ProgramResult {
    match (existing_or_inferred_token_std, desired_token_standard) {
        (
            TokenStandard::Fungible | TokenStandard::FungibleAsset,
            TokenStandard::Fungible | TokenStandard::FungibleAsset,
        ) => Ok(()),
        (existing, desired) if existing == desired => Ok(()),
        _ => Err(MetadataError::InvalidTokenStandard.into()),
    }
}
