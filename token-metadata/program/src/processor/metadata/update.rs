use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, sysvar,
};

use crate::{
    assertions::assert_owned_by,
    error::MetadataError,
    instruction::UpdateArgs,
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
        update_authority_info,
        holder_token_account_opt_info,
        system_program_info,
        sysvar_instructions_info,
        authorization_rules_opt_info,
    } = args.get_accounts(accounts)?;

    msg!("account validation");
    //** Account Validation **/
    // Check signers
    assert_signer(update_authority_info)?;

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
    let mut metadata_data = Metadata::from_account_info(metadata_info)?;

    if metadata_data.token_standard.is_none() {
        return Err(MetadataError::CouldNotDetermineTokenStandard.into());
    }

    match metadata_data.token_standard.unwrap() {
        TokenStandard::ProgrammableNonFungible => {
            let authorization_data = args.get_auth_data();

            if authorization_rules_opt_info.is_none() || authorization_data.is_none() {
                return Err(MetadataError::MissingAuthorizationRules.into());
            }

            if metadata_data.programmable_config.is_none() {
                return Err(MetadataError::MissingProgrammableConfig.into());
            }

            if master_edition_opt_info.is_none() {
                return Err(MetadataError::MissingEditionAccount.into());
            }
            let _master_edition_info = master_edition_opt_info.unwrap();
        }
        TokenStandard::NonFungible
        | TokenStandard::NonFungibleEdition
        | TokenStandard::Fungible
        | TokenStandard::FungibleAsset => {
            metadata_data.update_data(args, update_authority_info, metadata_info)?;
        }
    }

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
                    authorization_rules_opt_info,
                    system_program_info,
                    sysvar_instructions_info,
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
