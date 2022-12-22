use mpl_token_auth_rules::payload::{PayloadKey, PayloadType};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, sysvar,
};

use crate::{
    assertions::{assert_owned_by, programmable::assert_valid_authorization},
    error::MetadataError,
    instruction::UpdateArgs,
    processor::{try_get_account_info, try_get_optional_account_info, AuthorizationData},
    state::{Metadata, Operation, TokenMetadataAccount, TokenStandard},
    utils::{assert_update_authority_is_correct, validate},
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
        update_authority_info,
        holder_token_account_opt_info,
        system_program_info,
        sysvar_instructions_info,
        authorization_rules_opt_info,
    } = args.get_accounts(accounts)?;

    //** Account Validation **/
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

    // Deserialize metadata to determine its type
    let mut metadata = Metadata::from_account_info(metadata_info)?;

    // Check signers
    // Check update authority
    assert_update_authority_is_correct(&metadata, update_authority_info)?;

    // Check program IDs.
    if system_program_info.key != &solana_program::system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if metadata.token_standard.is_none() {
        return Err(MetadataError::CouldNotDetermineTokenStandard.into());
    }

    let token_standard = metadata.token_standard.unwrap();

    // The token being frozen has no effect on updating the metadata account,
    // so we do not have to thaw and re-freeze here.
    //
    // If the NFT has a programmable config set, we have to get the authorization
    // rules and validate if this action is allowed.
    //
    // Authorization rules only apply to the ProgrammableNonFungible asset type
    // currently.
    if matches!(token_standard, TokenStandard::ProgrammableNonFungible) {
        if let Some(ref config) = metadata.programmable_config {
            let authorization_data = args.get_auth_data();

            assert_valid_authorization(authorization_rules_opt_info, config)?;

            // We can safely unwrap here because they were all checked for existence
            // in the assertion above.
            let auth_pda = authorization_rules_opt_info.unwrap();
            let mut auth_data = authorization_data.unwrap();

            /*
            Insert auth rules for Update.
            Generic target for update_authority to allow for different rules
            for who can update.
             */
            auth_data.payload.insert(
                PayloadKey::Target,
                PayloadType::Pubkey(*update_authority_info.key),
            );

            // This panics if the CPI into the auth rules program fails, so unwrapping is ok.
            validate(
                auth_pda,
                Operation::Update,
                update_authority_info,
                &auth_data,
            )
            .unwrap();
        }
    }
    // For other token types we can simply update the metadata.
    metadata.update(args, update_authority_info, metadata_info)?;

    Ok(())
}

enum UpdateAccounts<'a> {
    V1 {
        metadata_info: &'a AccountInfo<'a>,
        mint_info: &'a AccountInfo<'a>,
        master_edition_opt_info: Option<&'a AccountInfo<'a>>,
        update_authority_info: &'a AccountInfo<'a>,
        holder_token_account_opt_info: Option<&'a AccountInfo<'a>>,
        system_program_info: &'a AccountInfo<'a>,
        sysvar_instructions_info: &'a AccountInfo<'a>,
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
                let update_authority_info = try_get_account_info(accounts, 5)?;
                let holder_token_account_opt_info = try_get_optional_account_info(accounts, 6)?;
                let _mpl_token_auth_rules_info = try_get_optional_account_info(accounts, 7)?;
                let authorization_rules_opt_info = try_get_optional_account_info(accounts, 8)?;

                Ok(UpdateAccounts::V1 {
                    metadata_info,
                    mint_info,
                    master_edition_opt_info,
                    update_authority_info,
                    holder_token_account_opt_info,
                    system_program_info,
                    sysvar_instructions_info,
                    authorization_rules_opt_info,
                })
            }
        }
    }

    fn get_auth_data(&self) -> Option<AuthorizationData> {
        match self {
            UpdateArgs::V1 {
                authorization_data, ..
            } => authorization_data.clone(),
        }
    }
}
