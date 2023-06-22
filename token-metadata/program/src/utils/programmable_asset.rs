use mpl_token_auth_rules::{
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::PayloadType,
};
use mpl_utils::{create_or_allocate_account_raw, token::TokenTransferParams};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, program_option::COption, pubkey::Pubkey,
};
use spl_token::{
    instruction::{freeze_account, thaw_account, AuthorityType as SplAuthorityType},
    state::Account,
};

use crate::{
    assertions::{assert_derivation, programmable::assert_valid_authorization},
    edition_seeds,
    error::MetadataError,
    pda::{EDITION, PREFIX},
    processor::{AuthorizationData, TransferScenario},
    state::{
        Operation, PayloadKey, ProgrammableConfig, Resizable, ToAccountMeta, TokenMetadataAccount,
        TokenRecord, TOKEN_RECORD_SEED,
    },
};

pub fn create_token_record_account<'a>(
    program_id: &Pubkey,
    token_record_info: &'a AccountInfo<'a>,
    mint_info: &'a AccountInfo<'a>,
    token_info: &'a AccountInfo<'a>,
    payer_info: &'a AccountInfo<'a>,
    system_program_info: &'a AccountInfo<'a>,
) -> ProgramResult {
    if !token_record_info.data_is_empty() {
        return Err(MetadataError::DelegateAlreadyExists.into());
    }

    let mut signer_seeds = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        mint_info.key.as_ref(),
        TOKEN_RECORD_SEED.as_bytes(),
        token_info.key.as_ref(),
    ]);

    let bump = &[assert_derivation(
        program_id,
        token_record_info,
        &signer_seeds,
    )?];
    signer_seeds.push(bump);

    // allocate the delegate account

    create_or_allocate_account_raw(
        *program_id,
        token_record_info,
        system_program_info,
        payer_info,
        TokenRecord::size(),
        &signer_seeds,
    )?;

    let token_record = TokenRecord {
        bump: bump[0],
        ..Default::default()
    };

    token_record.save(token_record_info, payer_info, system_program_info)
}

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
    let edition_info_path_bump_seed =
        &[assert_derivation(&crate::ID, &edition, &edition_info_path)?];
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
    mint_info: AccountInfo<'a>,
    token_info: AccountInfo<'a>,
    edition_info: AccountInfo<'a>,
    spl_token_program: AccountInfo<'a>,
) -> ProgramResult {
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let edition_info_path_bump_seed = &[assert_derivation(
        &crate::ID,
        &edition_info,
        &edition_info_path,
    )?];
    let mut edition_info_seeds = edition_info_path.clone();
    edition_info_seeds.push(edition_info_path_bump_seed);

    invoke_signed(
        &thaw_account(
            spl_token_program.key,
            token_info.key,
            mint_info.key,
            edition_info.key,
            &[],
        )
        .unwrap(),
        &[token_info, mint_info, edition_info],
        &[&edition_info_seeds],
    )?;
    Ok(())
}

pub fn validate<'a>(
    ruleset: &'a AccountInfo<'a>,
    operation: Operation,
    mint_info: &'a AccountInfo<'a>,
    additional_rule_accounts: Vec<&'a AccountInfo<'a>>,
    auth_data: &AuthorizationData,
    rule_set_revision: Option<usize>,
) -> Result<(), ProgramError> {
    let account_metas = additional_rule_accounts
        .iter()
        .map(|account| account.to_account_meta())
        .collect();

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(*ruleset.key)
        .mint(*mint_info.key)
        .additional_rule_accounts(account_metas)
        .build(ValidateArgs::V1 {
            operation: operation.to_string(),
            payload: auth_data.payload.clone(),
            update_rule_state: false,
            rule_set_revision,
        })
        .map_err(|_error| MetadataError::InvalidAuthorizationRules)?
        .instruction();

    let mut account_infos = vec![ruleset.clone(), mint_info.clone()];
    account_infos.extend(additional_rule_accounts.into_iter().cloned());
    invoke_signed(&validate_ix, account_infos.as_slice(), &[])
}

#[derive(Debug, Clone)]
pub struct AuthRulesValidateParams<'a> {
    pub mint_info: &'a AccountInfo<'a>,
    pub source_info: Option<&'a AccountInfo<'a>>,
    pub destination_info: Option<&'a AccountInfo<'a>>,
    pub authority_info: Option<&'a AccountInfo<'a>>,
    pub owner_info: Option<&'a AccountInfo<'a>>,
    pub programmable_config: Option<ProgrammableConfig>,
    pub amount: u64,
    pub auth_data: Option<AuthorizationData>,
    pub auth_rules_info: Option<&'a AccountInfo<'a>>,
    pub operation: Operation,
    pub is_wallet_to_wallet: bool,
    pub rule_set_revision: Option<usize>,
}

pub fn auth_rules_validate(params: AuthRulesValidateParams) -> ProgramResult {
    let AuthRulesValidateParams {
        mint_info,
        owner_info,
        source_info,
        destination_info,
        authority_info,
        programmable_config,
        amount,
        auth_data,
        auth_rules_info,
        operation,
        is_wallet_to_wallet,
        rule_set_revision,
    } = params;

    if is_wallet_to_wallet {
        return Ok(());
    }

    if let Operation::Transfer { scenario } = &operation {
        // Migration delegate is allowed to skip auth rules to guarantee that
        // it can transfer the asset.
        if matches!(scenario, TransferScenario::MigrationDelegate) {
            return Ok(());
        }
    }

    if let Some(ref config) = programmable_config {
        if let ProgrammableConfig::V1 { rule_set: Some(_) } = config {
            assert_valid_authorization(auth_rules_info, config)?;

            // We can safely unwrap here because they were all checked for existence
            // in the assertion above.
            let auth_pda = auth_rules_info.unwrap();

            let mut auth_data = if let Some(auth_data) = auth_data {
                auth_data
            } else {
                AuthorizationData::new_empty()
            };

            let mut additional_rule_accounts = vec![];
            if let Some(source_info) = source_info {
                additional_rule_accounts.push(source_info);
            }
            if let Some(destination_info) = destination_info {
                additional_rule_accounts.push(destination_info);
            }
            if let Some(authority_info) = authority_info {
                additional_rule_accounts.push(authority_info);
            }
            if let Some(owner_info) = owner_info {
                additional_rule_accounts.push(owner_info);
            }

            // Insert auth rules for the operation type.
            match operation {
                Operation::Transfer { scenario: _ } => {
                    // Get account infos
                    let authority_info = authority_info.ok_or(MetadataError::InvalidOperation)?;
                    let source_info = source_info.ok_or(MetadataError::InvalidOperation)?;
                    let destination_info =
                        destination_info.ok_or(MetadataError::InvalidOperation)?;

                    // Transfer Amount
                    auth_data
                        .payload
                        .insert(PayloadKey::Amount.to_string(), PayloadType::Number(amount));

                    // Transfer Authority
                    auth_data.payload.insert(
                        PayloadKey::Authority.to_string(),
                        PayloadType::Pubkey(*authority_info.key),
                    );

                    // Transfer Source
                    auth_data.payload.insert(
                        PayloadKey::Source.to_string(),
                        PayloadType::Pubkey(*source_info.key),
                    );

                    // Transfer Destination
                    auth_data.payload.insert(
                        PayloadKey::Destination.to_string(),
                        PayloadType::Pubkey(*destination_info.key),
                    );
                }
                Operation::Delegate { scenario: _ } => {
                    // get account infos
                    let destination_info =
                        destination_info.ok_or(MetadataError::InvalidOperation)?;

                    // delegate amount
                    auth_data
                        .payload
                        .insert(PayloadKey::Amount.to_string(), PayloadType::Number(amount));

                    // delegate authority
                    auth_data.payload.insert(
                        PayloadKey::Delegate.to_string(),
                        PayloadType::Pubkey(*destination_info.key),
                    );
                }
                _ => {
                    return Err(MetadataError::InvalidOperation.into());
                }
            }

            validate(
                auth_pda,
                operation,
                mint_info,
                additional_rule_accounts,
                &auth_data,
                rule_set_revision,
            )?;
        }
    }
    Ok(())
}

pub fn frozen_transfer<'a>(
    params: TokenTransferParams<'a, '_>,
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
    let dest_info = params.destination.clone();
    let token_program_info = params.token_program.clone();

    mpl_utils::token::spl_token_transfer(params).unwrap();

    freeze(
        mint_info,
        dest_info.clone(),
        master_edition_info.clone(),
        token_program_info.clone(),
    )?;

    Ok(())
}

pub(crate) struct ClearCloseAuthorityParams<'a> {
    pub token: Account,
    pub mint_info: &'a AccountInfo<'a>,
    pub token_info: &'a AccountInfo<'a>,
    pub master_edition_info: &'a AccountInfo<'a>,
    pub authority_info: &'a AccountInfo<'a>,
    pub spl_token_program_info: &'a AccountInfo<'a>,
}

pub(crate) fn clear_close_authority(params: ClearCloseAuthorityParams) -> ProgramResult {
    let ClearCloseAuthorityParams {
        token,
        mint_info,
        token_info,
        master_edition_info,
        authority_info,
        spl_token_program_info,
    } = params;

    // If there's an existing close authority that is not the metadata account,
    // it will need to be revoked by the original UtilityDelegate.
    if let COption::Some(close_authority) = token.close_authority {
        if &close_authority != master_edition_info.key {
            return Err(MetadataError::InvalidCloseAuthority.into());
        }
        let seeds = edition_seeds!(mint_info.key);

        invoke_signed(
            &spl_token::instruction::set_authority(
                spl_token_program_info.key,
                token_info.key,
                None,
                SplAuthorityType::CloseAccount,
                authority_info.key,
                &[],
            )?,
            &[token_info.clone(), authority_info.clone()],
            &[seeds.as_slice()],
        )?;
    }

    Ok(())
}
