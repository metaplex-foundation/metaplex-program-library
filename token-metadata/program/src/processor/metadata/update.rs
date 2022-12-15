use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

use crate::{
    assertions::assert_owned_by,
    error::MetadataError,
    instruction::UpdateArgs,
    pda::find_master_edition_account,
    state::{Metadata, TokenMetadataAccount, TokenStandard},
};

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
        metadata,
        mint,
        master_edition,
        update_authority,
        holder_token_account,
        system_program,
        sysvar_instructions,
        authorization_rules,
    } = args.get_accounts(accounts)?;

    //** Account Validation **/
    // Check signers
    assert_signer(update_authority)?;

    // Assert program ownership
    assert_owned_by(metadata, program_id)?;
    assert_owned_by(mint, &spl_token::id())?;

    if let Some(edition) = master_edition {
        assert_owned_by(edition, program_id)?;
    }
    if let Some(authorization_rules) = authorization_rules {
        assert_owned_by(authorization_rules, &mpl_token_auth_rules::ID)?;
    }
    if let Some(holder_token_account) = holder_token_account {
        assert_owned_by(holder_token_account, &spl_token::id())?;
    }

    // Check program IDs.
    if system_program.key != &solana_program::system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    if sysvar_instructions.key != &sysvar::instructions::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize metadata to determine its type
    let mut metadata_data = Metadata::from_account_info(metadata)?;

    if metadata_data.token_standard.is_none() {
        return Err(MetadataError::CouldNotDetermineTokenStandard.into());
    }

    match metadata_data.token_standard.unwrap() {
        TokenStandard::ProgrammableNonFungible => {
            todo!()
        }
        TokenStandard::NonFungible
        | TokenStandard::NonFungibleEdition
        | TokenStandard::Fungible
        | TokenStandard::FungibleAsset => {
            metadata_data.update_data(args, update_authority, metadata)?;
        }
    }

    Ok(())
}

enum UpdateAccounts<'a> {
    V1 {
        metadata: &'a AccountInfo<'a>,
        mint: &'a AccountInfo<'a>,
        master_edition: Option<&'a AccountInfo<'a>>,
        update_authority: &'a AccountInfo<'a>,
        holder_token_account: Option<&'a AccountInfo<'a>>,
        system_program: &'a AccountInfo<'a>,
        sysvar_instructions: &'a AccountInfo<'a>,
        authorization_rules: Option<&'a AccountInfo<'a>>,
    },
}

impl UpdateArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<UpdateAccounts<'a>, ProgramError> {
        let account_info_iter = &mut accounts.iter().peekable();

        match self {
            UpdateArgs::V1 { .. } => {
                let metadata = next_account_info(account_info_iter)?;
                let mint = next_account_info(account_info_iter)?;

                let system_program = next_account_info(account_info_iter)?;
                let sysvar_instructions = next_account_info(account_info_iter)?;

                let (edition_pda, _) = find_master_edition_account(mint.key);
                let master_edition = account_info_iter.next_if(|a| a.key == &edition_pda);

                let update_authority = next_account_info(account_info_iter)?;
                let holder_token_account = account_info_iter
                    .next_if(|a| a.owner == &spl_token::id() && !a.data_is_empty());

                // If the next account is the mpl_token_auth_rules ID, then we consume it
                // and read the next account which will be the authorization rules account.
                let authorization_rules = if account_info_iter
                    .next_if(|a| a.key == &mpl_token_auth_rules::ID)
                    .is_some()
                {
                    // Auth rules account
                    Some(next_account_info(account_info_iter)?)
                } else {
                    None
                };

                Ok(UpdateAccounts::V1 {
                    metadata,
                    mint,
                    master_edition,
                    update_authority,
                    holder_token_account,
                    authorization_rules,
                    system_program,
                    sysvar_instructions,
                })
            }
        }
    }
}
