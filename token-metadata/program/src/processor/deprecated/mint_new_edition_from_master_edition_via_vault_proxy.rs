use arrayref::array_ref;
use borsh::BorshSerialize;
use mpl_token_vault::{error::VaultError, state::VaultState};
use mpl_utils::{assert_signer, token::get_owner_from_token_account};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::{
    assertions::assert_owned_by,
    error::MetadataError,
    instruction::MintNewEditionFromMasterEditionViaTokenArgs,
    instruction_old::MetadataInstruction,
    utils::{
        process_mint_new_edition_from_master_edition_via_token_logic,
        MintNewEditionFromMasterEditionViaTokenLogicArgs,
    },
};

pub(crate) mod instruction {
    use super::*;

    /// creates a mint_edition_proxy instruction
    #[deprecated(since = "1.4.0")]
    #[allow(clippy::too_many_arguments)]
    pub fn mint_edition_from_master_edition_via_vault_proxy(
        program_id: Pubkey,
        new_metadata: Pubkey,
        new_edition: Pubkey,
        master_edition: Pubkey,
        new_mint: Pubkey,
        edition_mark_pda: Pubkey,
        new_mint_authority: Pubkey,
        payer: Pubkey,
        vault_authority: Pubkey,
        safety_deposit_store: Pubkey,
        safety_deposit_box: Pubkey,
        vault: Pubkey,
        new_metadata_update_authority: Pubkey,
        metadata: Pubkey,
        token_program: Pubkey,
        token_vault_program_info: Pubkey,
        edition: u64,
    ) -> Instruction {
        let accounts = vec![
            AccountMeta::new(new_metadata, false),
            AccountMeta::new(new_edition, false),
            AccountMeta::new(master_edition, false),
            AccountMeta::new(new_mint, false),
            AccountMeta::new(edition_mark_pda, false),
            AccountMeta::new_readonly(new_mint_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(vault_authority, true),
            AccountMeta::new_readonly(safety_deposit_store, false),
            AccountMeta::new_readonly(safety_deposit_box, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(new_metadata_update_authority, false),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(token_vault_program_info, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ];

        Instruction {
            program_id,
            accounts,
            data: MetadataInstruction::MintNewEditionFromMasterEditionViaVaultProxy(
                MintNewEditionFromMasterEditionViaTokenArgs { edition },
            )
            .try_to_vec()
            .unwrap(),
        }
    }
}

pub fn process_deprecated_mint_new_edition_from_master_edition_via_vault_proxy<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    edition: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let new_metadata_account_info = next_account_info(account_info_iter)?;
    let new_edition_account_info = next_account_info(account_info_iter)?;
    let master_edition_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let edition_marker_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let vault_authority_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let safety_deposit_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let master_metadata_account_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    // we cant do much here to prove that this is the right token vault program except to prove that it matches
    // the global one right now. We dont want to force people to use one vault program,
    // so there is a bit of trust involved, but the attack vector here is someone provides
    // an entirely fake vault program that claims to own token account X via it's pda but in order to spoof X's owner
    // and get a free edition. However, we check that the owner of account X is the vault account's pda, so
    // not sure how they would get away with it - they'd need to actually own that account! - J.
    let token_vault_program_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    let vault_data = vault_info.data.borrow();
    let safety_deposit_data = safety_deposit_info.data.borrow();

    // Since we're crunching out borsh for CPU units, do type checks this way
    if vault_data[0] != mpl_token_vault::state::Key::VaultV1 as u8 {
        return Err(VaultError::DataTypeMismatch.into());
    }

    if safety_deposit_data[0] != mpl_token_vault::state::Key::SafetyDepositBoxV1 as u8 {
        return Err(VaultError::DataTypeMismatch.into());
    }

    // skip deserialization to keep things cheap on CPU
    let token_program = Pubkey::new_from_array(*array_ref![vault_data, 1, 32]);
    let vault_authority = Pubkey::new_from_array(*array_ref![vault_data, 65, 32]);
    let store_on_sd = Pubkey::new_from_array(*array_ref![safety_deposit_data, 65, 32]);
    let vault_on_sd = Pubkey::new_from_array(*array_ref![safety_deposit_data, 1, 32]);

    let owner = get_owner_from_token_account(store_info)?;

    let seeds = &[
        mpl_token_vault::state::PREFIX.as_bytes(),
        token_vault_program_info.key.as_ref(),
        vault_info.key.as_ref(),
    ];
    let (authority, _) = Pubkey::find_program_address(seeds, token_vault_program_info.key);

    if owner != authority {
        return Err(MetadataError::InvalidOwner.into());
    }

    assert_signer(vault_authority_info)?;

    // Since most checks happen next level down in token program, we only need to verify
    // that the vault authority signer matches what's expected on vault to authorize
    // use of our pda authority, and that the token store is right for the safety deposit.
    // Then pass it through.
    assert_owned_by(vault_info, token_vault_program_info.key)?;
    assert_owned_by(safety_deposit_info, token_vault_program_info.key)?;
    assert_owned_by(store_info, token_program_account_info.key)?;

    if &token_program != token_program_account_info.key {
        return Err(VaultError::TokenProgramProvidedDoesNotMatchVault.into());
    }

    if !vault_authority_info.is_signer {
        return Err(VaultError::AuthorityIsNotSigner.into());
    }
    if *vault_authority_info.key != vault_authority {
        return Err(VaultError::AuthorityDoesNotMatch.into());
    }

    if vault_data[195] != VaultState::Combined as u8 {
        return Err(VaultError::VaultShouldBeCombined.into());
    }

    if vault_on_sd != *vault_info.key {
        return Err(VaultError::SafetyDepositBoxVaultMismatch.into());
    }

    if *store_info.key != store_on_sd {
        return Err(VaultError::StoreDoesNotMatchSafetyDepositBox.into());
    }

    let args = MintNewEditionFromMasterEditionViaTokenLogicArgs {
        new_metadata_account_info,
        new_edition_account_info,
        master_edition_account_info,
        mint_info,
        edition_marker_info,
        mint_authority_info,
        payer_account_info: payer_info,
        owner_account_info: vault_authority_info,
        token_account_info: store_info,
        update_authority_info,
        master_metadata_account_info,
        token_program_account_info,
        system_account_info,
    };

    process_mint_new_edition_from_master_edition_via_token_logic(program_id, args, edition, true)
}
