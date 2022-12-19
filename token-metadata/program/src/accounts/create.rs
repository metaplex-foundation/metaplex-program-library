use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
};

use crate::processor::next_optional_account_info;

use super::Context;

pub struct Create<'a> {
    pub metadata_info: &'a AccountInfo<'a>,
    pub mint_info: &'a AccountInfo<'a>,
    pub mint_authority_info: &'a AccountInfo<'a>,
    pub payer_info: &'a AccountInfo<'a>,
    pub update_authority_info: &'a AccountInfo<'a>,
    pub system_program_info: &'a AccountInfo<'a>,
    pub sysvar_instructions_info: &'a AccountInfo<'a>,
    pub spl_token_program_info: &'a AccountInfo<'a>,
    pub master_edition_info: Option<&'a AccountInfo<'a>>,
    pub authorization_rules_info: Option<&'a AccountInfo<'a>>,
}

impl<'a> Create<'a> {
    pub fn as_context(
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<Context<'a, Self>, ProgramError> {
        let account_info_iter = &mut accounts.iter();

        let metadata_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let mint_authority_info = next_account_info(account_info_iter)?;
        let payer_info = next_account_info(account_info_iter)?;
        let update_authority_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let sysvar_instructions_info = next_account_info(account_info_iter)?;
        let spl_token_program_info = next_account_info(account_info_iter)?;
        let master_edition_info = next_optional_account_info(account_info_iter)?;
        let authorization_rules_info = next_optional_account_info(account_info_iter)?;

        let accounts = Self {
            authorization_rules_info,
            master_edition_info,
            metadata_info,
            mint_info,
            mint_authority_info,
            payer_info,
            spl_token_program_info,
            system_program_info,
            sysvar_instructions_info,
            update_authority_info,
        };

        Ok(Context {
            accounts,
            remaining: Vec::<&'a AccountInfo<'a>>::from_iter(account_info_iter),
        })
    }
}
