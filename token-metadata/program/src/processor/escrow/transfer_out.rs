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
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: TransferOutOfEscrowArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_info = next_account_info(account_info_iter)?;
    let _metadata_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let attribute_mint_info = next_account_info(account_info_iter)?;
    let attribute_src_info = next_account_info(account_info_iter)?;
    let attribute_dst_info = next_account_info(account_info_iter)?;
    let escrow_mint_info = next_account_info(account_info_iter)?;
    let escrow_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let ata_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let _sysvar_ix_account_info = next_account_info(account_info_iter)?;

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

    assert_owned_by(escrow_info, program_id)?;
    let toe = TokenOwnedEscrow::from_account_info(escrow_info)?;

    // Derive the seeds for PDA signing.
    let escrow_seeds = find_escrow_seeds(escrow_mint_info.key, &toe.authority);

    let bump_seed = &[assert_derivation(&crate::id(), escrow_info, &escrow_seeds)?];
    let escrow_authority_seeds = [escrow_seeds, vec![bump_seed]].concat();

    assert_signer(payer_info)?;

    // Allocate the target ATA if it doesn't exist.
    if !is_initialized_account(*attribute_dst_info.data.borrow()) {
        #[allow(deprecated)]
        let create_escrow_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                payer_info.key,
                payer_info.key,
                attribute_mint_info.key,
            );

        invoke(
            &create_escrow_ata_ix,
            &[
                payer_info.clone(),
                attribute_dst_info.clone(),
                attribute_mint_info.clone(),
                system_account_info.clone(),
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

    // Check that the authority matches based on the authority type.
    let escrow_account = spl_token::state::Account::unpack(&escrow_account_info.data.borrow())?;
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
        &spl_token::id(),
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

    // Close the source ATA and return funds to the user.
    let close_ix = spl_token::instruction::close_account(
        &spl_token::id(),
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

    Ok(())
}
