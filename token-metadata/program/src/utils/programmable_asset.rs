use mpl_token_auth_rules::instruction::InstructionBuilder;
use mpl_token_auth_rules::{
    instruction::{builders::ValidateBuilder, ValidateArgs},
    payload::{PayloadKey, PayloadType},
};
use mpl_utils::token::TokenTransferParams;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
};
use spl_token::instruction::{freeze_account, thaw_account};

use crate::{
    assertions::{assert_derivation, programmable::assert_valid_authorization},
    error::MetadataError,
    pda::{EDITION, PREFIX},
    processor::AuthorizationData,
    state::{Operation, ProgrammableConfig},
};

pub fn freeze<'a>(
    mint: AccountInfo<'a>,
    token: AccountInfo<'a>,
    edition: AccountInfo<'a>,
    spl_token_program: AccountInfo<'a>,
) -> ProgramResult {
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        mint.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let edition_info_path_bump_seed = &[assert_derivation(
        &crate::id(),
        &edition,
        &edition_info_path,
    )?];
    let mut edition_info_seeds = edition_info_path.clone();
    edition_info_seeds.push(edition_info_path_bump_seed);

    invoke_signed(
        &freeze_account(spl_token_program.key, token.key, mint.key, edition.key, &[]).unwrap(),
        &[token, mint, edition],
        &[&edition_info_seeds],
    )?;
    Ok(())
}

pub fn thaw<'a>(
    mint: AccountInfo<'a>,
    token: AccountInfo<'a>,
    edition: AccountInfo<'a>,
    spl_token_program: AccountInfo<'a>,
) -> ProgramResult {
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        mint.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let edition_info_path_bump_seed = &[assert_derivation(
        &crate::id(),
        &edition,
        &edition_info_path,
    )?];
    let mut edition_info_seeds = edition_info_path.clone();
    edition_info_seeds.push(edition_info_path_bump_seed);

    invoke_signed(
        &thaw_account(spl_token_program.key, token.key, mint.key, edition.key, &[]).unwrap(),
        &[token, mint, edition],
        &[&edition_info_seeds],
    )?;
    Ok(())
}

pub fn validate<'a>(
    ruleset: &'a AccountInfo<'a>,
    operation: Operation,
    mint_info: &'a AccountInfo<'a>,
    target: &'a AccountInfo<'a>,
    auth_data: &AuthorizationData,
) -> Result<(), ProgramError> {
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(*ruleset.key)
        .mint(*mint_info.key)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(*target.key, false)])
        .build(ValidateArgs::V1 {
            operation: operation.to_string(),
            payload: auth_data.payload.clone(),
            update_rule_state: false,
        })
        .map_err(|_error| MetadataError::InvalidAuthorizationRules)?
        .instruction();

    invoke_signed(
        &validate_ix,
        &[ruleset.clone(), mint_info.clone(), target.clone()],
        &[],
    )
}

#[derive(Debug, Clone)]
pub struct AuthRulesValidateParams<'a> {
    pub mint_info: &'a AccountInfo<'a>,
    pub destination_owner_info: &'a AccountInfo<'a>,
    pub programmable_config: Option<ProgrammableConfig>,
    pub amount: u64,
    pub auth_data: Option<AuthorizationData>,
    pub auth_rules_info: Option<&'a AccountInfo<'a>>,
}

pub fn auth_rules_validate(params: AuthRulesValidateParams) -> ProgramResult {
    let AuthRulesValidateParams {
        mint_info,
        destination_owner_info,
        programmable_config,
        amount,
        auth_data,
        auth_rules_info,
    } = params;

    if let Some(ref config) = programmable_config {
        msg!("Programmable config exists");
        let operation = Operation::Transfer;

        assert_valid_authorization(auth_rules_info, config)?;

        msg!("valid auth data. Adding rules...");
        // We can safely unwrap here because they were all checked for existence
        // in the assertion above.
        let auth_pda = auth_rules_info.unwrap();

        let mut auth_data = if let Some(auth_data) = auth_data {
            auth_data
        } else {
            AuthorizationData::new_empty()
        };

        // Insert auth rules for Transfer
        auth_data
            .payload
            .insert(PayloadKey::Amount, PayloadType::Number(amount));
        auth_data.payload.insert(
            PayloadKey::Target,
            PayloadType::Pubkey(*destination_owner_info.key),
        );

        validate(
            auth_pda,
            operation,
            mint_info,
            destination_owner_info,
            &auth_data,
        )?;
    }

    Ok(())
}

pub fn frozen_transfer<'a, 'b>(
    params: TokenTransferParams<'a, 'b>,
    edition_opt_info: Option<&'a AccountInfo<'a>>,
) -> ProgramResult {
    if edition_opt_info.is_none() {
        return Err(MetadataError::MissingEditionAccount.into());
    }
    let master_edition_info = edition_opt_info.unwrap();

    thaw(
        params.mint.clone(),
        params.source.clone(),
        master_edition_info.clone(),
        params.token_program.clone(),
    )?;

    let mint_info = params.mint.clone();
    let source_info = params.source.clone();
    let token_program_info = params.token_program.clone();

    mpl_utils::token::spl_token_transfer(params).unwrap();

    freeze(
        mint_info,
        source_info.clone(),
        master_edition_info.clone(),
        token_program_info.clone(),
    )?;

    Ok(())
}
