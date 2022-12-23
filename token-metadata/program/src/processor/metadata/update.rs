use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, sysvar,
};

use crate::{
    assertions::{assert_owned_by, metadata::assert_metadata_authority},
    error::MetadataError,
    instruction::{AuthorityType, UpdateArgs},
    processor::{try_get_account_info, try_get_optional_account_info, AuthorizationData},
    state::{Metadata, TokenMetadataAccount, TokenStandard},
};

const EXPECTED_ACCOUNTS_LEN: usize = 9;

pub fn update<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: UpdateArgs,
) -> ProgramResult {
    match args {
        UpdateArgs::V1 { .. } => update_v1(program_id, accounts, args),
    }
}

fn update_v1<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: UpdateArgs,
) -> ProgramResult {
    let UpdateAccounts::V1 {
        metadata_info,
        mint_info,
        master_edition_opt_info,
        authority_info,
        holder_token_account_opt_info,
        _delegate_record_opt_info,
        system_program_info,
        sysvar_instructions_info,
        authorization_rules_opt_info,
    } = args.get_accounts(accounts)?;

    //** Account Validation **/
    // Assert signers
    // This account should always be a signer regardless of the authority type,
    // because at least one signer is required to update the metadata.
    assert_signer(authority_info)?;

    // Assert program ownership
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::ID)?;

    if let Some(edition) = master_edition_opt_info {
        assert_owned_by(edition, program_id)?;
    }
    if let Some(authorization_rules) = authorization_rules_opt_info {
        assert_owned_by(authorization_rules, &mpl_token_auth_rules::ID)?;
    }
    if let Some(holder_token_account) = holder_token_account_opt_info {
        assert_owned_by(holder_token_account, &spl_token::ID)?;
    }

    // Check program IDs.
    if system_program_info.key != &solana_program::system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize metadata to determine its type
    let mut metadata = Metadata::from_account_info(metadata_info)?;

    // Validate relationships

    // Mint must match metadata mint
    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    let token_standard = metadata
        .token_standard
        .ok_or(MetadataError::InvalidTokenStandard)?;

    // We have to validate authorization rules if the token standard is a Programmable type
    // and the metadata has a programmable config set.
    let _auth_rules_apply = matches!(token_standard, TokenStandard::ProgrammableNonFungible)
        && metadata.programmable_config.is_some();

    let authority_type = args.get_authority_type();

    match authority_type {
        AuthorityType::Metadata => {
            // Check is signer and matches update authority on metadata.
            assert_metadata_authority(&metadata, authority_info)?;

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
    metadata.update(args, authority_info, metadata_info)?;

    Ok(())
}

enum UpdateAccounts<'a> {
    V1 {
        metadata_info: &'a AccountInfo<'a>,
        mint_info: &'a AccountInfo<'a>,
        system_program_info: &'a AccountInfo<'a>,
        sysvar_instructions_info: &'a AccountInfo<'a>,
        master_edition_opt_info: Option<&'a AccountInfo<'a>>,
        authority_info: &'a AccountInfo<'a>,
        holder_token_account_opt_info: Option<&'a AccountInfo<'a>>,
        _delegate_record_opt_info: Option<&'a AccountInfo<'a>>,
        authorization_rules_opt_info: Option<&'a AccountInfo<'a>>,
    },
}

impl UpdateArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<UpdateAccounts<'a>, ProgramError> {
        // validates that we got the correct number of accounts
        if accounts.len() < EXPECTED_ACCOUNTS_LEN {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        match self {
            UpdateArgs::V1 { .. } => {
                let metadata_info = try_get_account_info(accounts, 0)?;
                let mint_info = try_get_account_info(accounts, 1)?;
                let system_program_info = try_get_account_info(accounts, 2)?;
                let sysvar_instructions_info = try_get_account_info(accounts, 3)?;
                let master_edition_opt_info = try_get_optional_account_info(accounts, 4)?;
                let authority_info = try_get_account_info(accounts, 5)?;
                let holder_token_account_opt_info = try_get_optional_account_info(accounts, 6)?;
                let _delegate_record_opt_info = try_get_optional_account_info(accounts, 7)?;
                let _mpl_token_auth_rules_info = try_get_optional_account_info(accounts, 8)?;
                let authorization_rules_opt_info = try_get_optional_account_info(accounts, 9)?;

                Ok(UpdateAccounts::V1 {
                    metadata_info,
                    mint_info,
                    master_edition_opt_info,
                    system_program_info,
                    sysvar_instructions_info,
                    authority_info,
                    holder_token_account_opt_info,
                    _delegate_record_opt_info,
                    authorization_rules_opt_info,
                })
            }
        }
    }

    fn _get_auth_data(&self) -> Option<AuthorizationData> {
        match self {
            UpdateArgs::V1 {
                authorization_data, ..
            } => authorization_data.clone(),
        }
    }

    fn get_authority_type(&self) -> AuthorityType {
        match self {
            UpdateArgs::V1 { authority_type, .. } => authority_type.clone(),
        }
    }
}
