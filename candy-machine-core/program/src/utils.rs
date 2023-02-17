use anchor_lang::prelude::*;
use arrayref::array_ref;
use mpl_token_metadata::{
    error::MetadataError,
    instruction::{
        approve_collection_authority,
        builders::{DelegateBuilder, RevokeBuilder},
        revoke_collection_authority, DelegateArgs, InstructionBuilder, RevokeArgs,
    },
    state::{Metadata, TokenMetadataAccount, EDITION, PREFIX},
    utils::assert_derivation,
};
use solana_program::{
    account_info::AccountInfo,
    program::{invoke, invoke_signed},
    program_memory::sol_memcmp,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, PUBKEY_BYTES},
    system_instruction,
};
use std::result::Result as StdResult;

use crate::{
    constants::{
        AUTHORITY_SEED, HIDDEN_SECTION, NULL_STRING, REPLACEMENT_INDEX, REPLACEMENT_INDEX_INCREMENT,
    },
    CandyError,
};

pub struct ApproveCollectionAuthorityHelperAccounts<'info> {
    /// CHECK: account checked in CPI
    pub payer: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub authority_pda: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_update_authority: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_mint: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_metadata: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_authority_record: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub token_metadata_program: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub system_program: AccountInfo<'info>,
}

pub struct RevokeCollectionAuthorityHelperAccounts<'info> {
    /// CHECK: account checked in CPI
    pub authority_pda: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_mint: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_metadata: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_authority_record: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub token_metadata_program: AccountInfo<'info>,
}

pub struct ApproveMetadataDelegateHelperAccounts<'info> {
    /// CHECK: account checked in CPI
    pub delegate_record: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub authority_pda: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_metadata: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_mint: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_update_authority: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub payer: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub system_program: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub sysvar_instructions: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub authorization_rules_program: Option<AccountInfo<'info>>,
    /// CHECK: account checked in CPI
    pub authorization_rules: Option<AccountInfo<'info>>,
}

pub struct RevokeMetadataDelegateHelperAccounts<'info> {
    /// CHECK: account checked in CPI
    pub delegate_record: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub authority_pda: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_metadata: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_mint: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_update_authority: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub payer: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub system_program: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub sysvar_instructions: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub authorization_rules_program: Option<AccountInfo<'info>>,
    /// CHECK: account checked in CPI
    pub authorization_rules: Option<AccountInfo<'info>>,
}

pub fn assert_initialized<T: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<T> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(CandyError::Uninitialized.into())
    } else {
        Ok(account)
    }
}

/// Return the current number of lines written to the account.
pub fn get_config_count(data: &[u8]) -> Result<usize> {
    Ok(u32::from_le_bytes(*array_ref![data, HIDDEN_SECTION, 4]) as usize)
}

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

/// Return a padded string up to the specified length. If the specified
/// string `value` is longer than the allowed `length`, return an error.
pub fn fixed_length_string(value: String, length: usize) -> Result<String> {
    if length < value.len() {
        // the value is larger than the allowed length
        return err!(CandyError::ExceededLengthError);
    }

    let padding = NULL_STRING.repeat(length - value.len());
    Ok(value + &padding)
}

pub fn punish_bots<'a>(
    error: CandyError,
    bot_account: AccountInfo<'a>,
    payment_account: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    fee: u64,
) -> Result<()> {
    msg!(
        "{}, Candy Machine Botting is taxed at {:?} lamports",
        error.to_string(),
        fee
    );

    let final_fee = fee.min(bot_account.lamports());
    invoke(
        &system_instruction::transfer(bot_account.key, payment_account.key, final_fee),
        &[bot_account, payment_account, system_program],
    )?;
    Ok(())
}

/// Replace the index pattern variables on the specified string.
pub fn replace_patterns(value: String, index: usize) -> String {
    let mut mutable = value;
    // check for pattern $ID+1$
    if mutable.contains(REPLACEMENT_INDEX_INCREMENT) {
        mutable = mutable.replace(REPLACEMENT_INDEX_INCREMENT, &(index + 1).to_string());
    }
    // check for pattern $ID$
    if mutable.contains(REPLACEMENT_INDEX) {
        mutable = mutable.replace(REPLACEMENT_INDEX, &index.to_string());
    }

    mutable
}

pub fn assert_edition_from_mint(
    edition_account: &AccountInfo,
    mint_account: &AccountInfo,
) -> StdResult<(), ProgramError> {
    assert_derivation(
        &mpl_token_metadata::id(),
        edition_account,
        &[
            PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint_account.key().as_ref(),
            EDITION.as_bytes(),
        ],
    )
    .map_err(|_| MetadataError::CollectionMasterEditionAccountInvalid)?;
    Ok(())
}

pub fn approve_collection_authority_helper(
    accounts: ApproveCollectionAuthorityHelperAccounts,
) -> Result<()> {
    let ApproveCollectionAuthorityHelperAccounts {
        payer,
        authority_pda,
        collection_update_authority,
        collection_mint,
        collection_metadata,
        collection_authority_record,
        token_metadata_program,
        system_program,
    } = accounts;

    let collection_data: Metadata = Metadata::from_account_info(&collection_metadata)?;

    if !cmp_pubkeys(
        &collection_data.update_authority,
        &collection_update_authority.key(),
    ) {
        return err!(CandyError::IncorrectCollectionAuthority);
    }

    if !cmp_pubkeys(&collection_data.mint, &collection_mint.key()) {
        return err!(CandyError::MintMismatch);
    }

    let approve_collection_authority_ix = approve_collection_authority(
        token_metadata_program.key(),
        collection_authority_record.key(),
        authority_pda.key(),
        collection_update_authority.key(),
        payer.key(),
        collection_metadata.key(),
        collection_mint.key(),
    );

    if collection_authority_record.data_is_empty() {
        let approve_collection_infos = vec![
            collection_authority_record,
            authority_pda,
            collection_update_authority,
            payer,
            collection_metadata,
            collection_mint,
            system_program,
        ];

        invoke(
            &approve_collection_authority_ix,
            approve_collection_infos.as_slice(),
        )?;
    }

    Ok(())
}

pub fn revoke_collection_authority_helper(
    accounts: RevokeCollectionAuthorityHelperAccounts,
    candy_machine: Pubkey,
    signer_bump: u8,
) -> Result<()> {
    let revoke_collection_infos = vec![
        accounts.collection_authority_record.to_account_info(),
        accounts.authority_pda.to_account_info(),
        accounts.collection_metadata.to_account_info(),
        accounts.collection_mint.to_account_info(),
    ];

    let authority_seeds = [
        AUTHORITY_SEED.as_bytes(),
        candy_machine.as_ref(),
        &[signer_bump],
    ];

    invoke_signed(
        &revoke_collection_authority(
            accounts.token_metadata_program.key(),
            accounts.collection_authority_record.key(),
            accounts.authority_pda.key(),
            accounts.authority_pda.key(),
            accounts.collection_metadata.key(),
            accounts.collection_mint.key(),
        ),
        revoke_collection_infos.as_slice(),
        &[&authority_seeds],
    )
    .map_err(|error| error.into())
}

pub fn approve_metadata_delegate(accounts: ApproveMetadataDelegateHelperAccounts) -> Result<()> {
    let mut delegate_builder = DelegateBuilder::new();
    delegate_builder
        .delegate_record(accounts.delegate_record.key())
        .delegate(accounts.authority_pda.key())
        .mint(accounts.collection_mint.key())
        .metadata(accounts.collection_metadata.key())
        .payer(accounts.payer.key())
        .authority(accounts.collection_update_authority.key());

    let mut delegate_infos = vec![
        accounts.delegate_record.to_account_info(),
        accounts.authority_pda.to_account_info(),
        accounts.collection_metadata.to_account_info(),
        accounts.collection_mint.to_account_info(),
        accounts.collection_update_authority.to_account_info(),
        accounts.payer.to_account_info(),
        accounts.system_program.to_account_info(),
        accounts.sysvar_instructions.to_account_info(),
    ];

    if let Some(authorization_rules_program) = &accounts.authorization_rules_program {
        delegate_builder.authorization_rules_program(authorization_rules_program.key());
        delegate_infos.push(authorization_rules_program.to_account_info());
    }

    if let Some(authorization_rules) = &accounts.authorization_rules {
        delegate_builder.authorization_rules(authorization_rules.key());
        delegate_infos.push(authorization_rules.to_account_info());
    }

    let delegate_ix = delegate_builder
        .build(DelegateArgs::CollectionV1 {
            authorization_data: None,
        })
        .map_err(|_| CandyError::InstructionBuilderFailed)?
        .instruction();

    invoke(&delegate_ix, &delegate_infos).map_err(|error| error.into())
}

pub fn revoke_metadata_delegate(accounts: RevokeMetadataDelegateHelperAccounts) -> Result<()> {
    let mut revoke_builder = RevokeBuilder::new();
    revoke_builder
        .delegate_record(accounts.delegate_record.key())
        .delegate(accounts.authority_pda.key())
        .mint(accounts.collection_mint.key())
        .metadata(accounts.collection_metadata.key())
        .payer(accounts.payer.key())
        .authority(accounts.collection_update_authority.key());

    let mut revoke_infos = vec![
        accounts.delegate_record.to_account_info(),
        accounts.authority_pda.to_account_info(),
        accounts.collection_metadata.to_account_info(),
        accounts.collection_mint.to_account_info(),
        accounts.collection_update_authority.to_account_info(),
        accounts.payer.to_account_info(),
        accounts.system_program.to_account_info(),
        accounts.sysvar_instructions.to_account_info(),
    ];

    if let Some(authorization_rules_program) = &accounts.authorization_rules_program {
        revoke_builder.authorization_rules_program(authorization_rules_program.key());
        revoke_infos.push(authorization_rules_program.to_account_info());
    }

    if let Some(authorization_rules) = &accounts.authorization_rules {
        revoke_builder.authorization_rules(authorization_rules.key());
        revoke_infos.push(authorization_rules.to_account_info());
    }

    let revoke_ix = revoke_builder
        .build(RevokeArgs::CollectionV1)
        .map_err(|_| CandyError::InstructionBuilderFailed)?
        .instruction();

    invoke(&revoke_ix, &revoke_infos).map_err(|error| error.into())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn check_keys_equal() {
        let key1 = Pubkey::new_unique();
        assert!(cmp_pubkeys(&key1, &key1));
    }

    #[test]
    fn check_keys_not_equal() {
        let key1 = Pubkey::new_unique();
        let key2 = Pubkey::new_unique();
        assert!(!cmp_pubkeys(&key1, &key2));
    }
}
