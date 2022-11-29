use mpl_utils::{assert_signer, close_account_raw};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    system_program,
};

use crate::{
    assertions::{assert_derivation, assert_initialized, assert_owned_by},
    error::MetadataError,
    state::{
        EscrowAuthority, Metadata, TokenMetadataAccount, TokenOwnedEscrow, TokenStandard,
        ESCROW_POSTFIX, PREFIX,
    },
    utils::check_token_standard,
};

pub fn process_close_escrow_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_account_info = next_account_info(account_info_iter)?;
    let metadata_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    if *system_account_info.key != system_program::id() {
        return Err(MetadataError::InvalidSystemProgram.into());
    }

    assert_owned_by(metadata_account_info, program_id)?;
    assert_owned_by(mint_account_info, &spl_token::id())?;
    assert_owned_by(token_account_info, &spl_token::id())?;
    assert_signer(payer_account_info)?;

    let metadata: Metadata = Metadata::from_account_info(metadata_account_info)?;

    // Mint account passed in must be the mint of the metadata account passed in.
    if &metadata.mint != mint_account_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if check_token_standard(mint_account_info, Some(edition_account_info))?
        != TokenStandard::NonFungible
    {
        return Err(MetadataError::MustBeNonFungible.into());
    };

    let bump_seed = assert_derivation(
        program_id,
        escrow_account_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_account_info.key.as_ref(),
            ESCROW_POSTFIX.as_bytes(),
        ],
    )?;

    assert_owned_by(escrow_account_info, program_id)?;
    let token_account: spl_token::state::Account = assert_initialized(token_account_info)?;
    let toe = TokenOwnedEscrow::from_account_info(escrow_account_info)?;

    if bump_seed != toe.bump {
        return Err(MetadataError::InvalidEscrowBumpSeed.into());
    }

    match toe.authority {
        EscrowAuthority::TokenOwner => {
            if *payer_account_info.key != token_account.owner {
                return Err(MetadataError::MustBeEscrowAuthority.into());
            }
        }
        EscrowAuthority::Creator(authority) => {
            if *payer_account_info.key != authority {
                return Err(MetadataError::MustBeEscrowAuthority.into());
            }
        }
    }

    // Close the account.
    close_account_raw(payer_account_info, escrow_account_info)?;

    Ok(())
}
