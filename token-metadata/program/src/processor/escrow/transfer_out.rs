use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::is_initialized_account;

use super::find_escrow_seeds;
use crate::{
    assertions::{assert_derivation, assert_owned_by},
    error::MetadataError,
    instruction::TransferOutOfEscrowArgs,
    state::{EscrowAuthority, TokenMetadataAccount, TokenOwnedEscrow},
};

pub fn process_transfer_out_of_escrow(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: TransferOutOfEscrowArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_info = next_account_info(account_info_iter)?;
    assert_owned_by(escrow_info, &crate::ID)?;

    // Currently unused, if used in the future the mint of the metadata should
    // be verified against the escrow mint.
    let metadata_info = next_account_info(account_info_iter)?;
    assert_owned_by(metadata_info, &crate::ID)?;

    let payer_info = next_account_info(account_info_iter)?;
    assert_signer(payer_info)?;

    let attribute_mint_info = next_account_info(account_info_iter)?;
    assert_owned_by(attribute_mint_info, &spl_token::ID)?;

    let attribute_src_info = next_account_info(account_info_iter)?;
    assert_owned_by(attribute_src_info, &spl_token::ID)?;

    // We don't check attribute destination ownership because it may not be initialized yet.
    let attribute_dst_info = next_account_info(account_info_iter)?;

    let escrow_mint_info = next_account_info(account_info_iter)?;
    assert_owned_by(escrow_mint_info, &spl_token::ID)?;

    let escrow_account_info = next_account_info(account_info_iter)?;
    assert_owned_by(escrow_account_info, &spl_token::ID)?;

    let system_program_info = next_account_info(account_info_iter)?;
    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MetadataError::InvalidSystemProgram.into());
    }

    let ata_program_info = next_account_info(account_info_iter)?;
    if ata_program_info.key != &spl_associated_token_account::ID {
        return Err(MetadataError::InvalidAssociatedTokenAccountProgram.into());
    }

    let token_program_info = next_account_info(account_info_iter)?;
    if token_program_info.key != &spl_token::ID {
        return Err(MetadataError::InvalidTokenProgram.into());
    }

    let sysvar_ix_account_info = next_account_info(account_info_iter)?;
    if sysvar_ix_account_info.key != &solana_program::sysvar::instructions::ID {
        return Err(MetadataError::InvalidInstructionsSysvar.into());
    }

    // Allow the option to set a different authority than the payer.
    let is_using_authority = account_info_iter.len() == 1;
    let maybe_authority_info: Option<&AccountInfo> = if is_using_authority {
        let auth = next_account_info(account_info_iter)?;
        assert_signer(auth)?;
        Some(auth)
    } else {
        None
    };
    let authority = maybe_authority_info.unwrap_or(payer_info);

    let toe = TokenOwnedEscrow::from_account_info(escrow_info)?;

    // Derive the seeds for PDA signing.
    let escrow_seeds = find_escrow_seeds(escrow_mint_info.key, &toe.authority);

    let bump_seed = &[assert_derivation(&crate::ID, escrow_info, &escrow_seeds)?];
    let escrow_authority_seeds = [escrow_seeds, vec![bump_seed]].concat();

    // Allocate the target ATA if it doesn't exist.
    if !is_initialized_account(&attribute_dst_info.data.borrow()) {
        #[allow(deprecated)]
        let create_escrow_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                payer_info.key,
                payer_info.key,
                attribute_mint_info.key,
                &spl_token::ID,
            );

        invoke(
            &create_escrow_ata_ix,
            &[
                payer_info.clone(),
                attribute_dst_info.clone(),
                attribute_mint_info.clone(),
                system_program_info.clone(),
                token_program_info.clone(),
                ata_program_info.clone(),
            ],
        )?;
    }

    // Deserialize the token accounts and perform checks.
    let attribute_src = spl_token::state::Account::unpack(&attribute_src_info.data.borrow())?;
    if attribute_src.mint != *attribute_mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }
    if attribute_src.amount < args.amount {
        return Err(MetadataError::InsufficientTokens.into());
    }
    if attribute_src.delegated_amount != 0 {
        return Err(MetadataError::EscrowParentHasDelegate.into());
    }

    // Check that the authority matches based on the authority type.
    let escrow_account = spl_token::state::Account::unpack(&escrow_account_info.data.borrow())?;
    if escrow_account.mint != *escrow_mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }
    if escrow_account.amount != 1 {
        return Err(MetadataError::AmountMustBeGreaterThanZero.into());
    }

    match toe.authority {
        EscrowAuthority::TokenOwner => {
            if escrow_account.owner != *authority.key {
                return Err(MetadataError::MustBeEscrowAuthority.into());
            }
        }
        EscrowAuthority::Creator(creator) => {
            if creator != *authority.key {
                return Err(MetadataError::MustBeEscrowAuthority.into());
            }
        }
    }

    let attribute_dst = spl_token::state::Account::unpack(&attribute_dst_info.data.borrow())?;
    if attribute_dst.mint != *attribute_mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    // Transfer the token out of the escrow to the destination ATA.
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        attribute_src_info.key,
        attribute_dst_info.key,
        escrow_info.key,
        &[escrow_info.key],
        args.amount,
    )?;

    invoke_signed(
        &transfer_ix,
        &[
            attribute_src_info.clone(),
            attribute_dst_info.clone(),
            escrow_info.clone(),
            token_program_info.clone(),
        ],
        &[&escrow_authority_seeds],
    )?;

    let attribute_src = spl_token::state::Account::unpack(&attribute_src_info.data.borrow())?;

    // Close the source ATA and return funds to the user.
    if attribute_src.amount == 0 {
        let close_ix = spl_token::instruction::close_account(
            &spl_token::ID,
            attribute_src_info.key,
            payer_info.key,
            escrow_info.key,
            &[escrow_info.key],
        )?;

        invoke_signed(
            &close_ix,
            &[
                attribute_src_info.clone(),
                payer_info.clone(),
                escrow_info.clone(),
                token_program_info.clone(),
            ],
            &[&escrow_authority_seeds],
        )?;
    }

    Ok(())
}
