use mpl_utils::assert_signer;
use num_traits::FromPrimitive;
use solana_program::{account_info::next_account_info, rent::Rent, system_program, sysvar::Sysvar};

use crate::{
    state::{fee::FEE_AUTHORITY, MAX_METADATA_LEN},
    utils::fee::clear_fee_flag,
};

use super::*;

pub(crate) fn process_collect_fees(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let authority_info = next_account_info(account_info_iter)?;

    assert_signer(authority_info)?;

    if *authority_info.key != FEE_AUTHORITY {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }

    let recipient_info = next_account_info(account_info_iter)?;

    for account_info in account_info_iter {
        if account_info.owner != program_id {
            return Err(MetadataError::InvalidFeeAccount.into());
        }

        collect_fee_from_account(account_info, recipient_info)?;
    }

    Ok(())
}

fn collect_fee_from_account(account_info: &AccountInfo, dest_info: &AccountInfo) -> ProgramResult {
    // Scope refcell borrow
    let account_key = {
        let data = account_info.data.borrow();

        // Burned accounts with fees will have no data, so should be assigned the `Uninitialized` key.
        let key_byte = data.first().unwrap_or(&0);

        FromPrimitive::from_u8(*key_byte).ok_or(MetadataError::InvalidFeeAccount)?
    };

    let rent = Rent::get()?;
    let metadata_rent = rent.minimum_balance(MAX_METADATA_LEN);

    let (fee_amount, rent_amount) = match account_key {
        Key::Uninitialized => {
            account_info.assign(&system_program::ID);

            (account_info.lamports(), 0)
        }
        Key::MetadataV1 => {
            let fee_amount = account_info
                .lamports()
                .checked_sub(metadata_rent)
                .ok_or(MetadataError::NumericalOverflowError)?;

            (fee_amount, metadata_rent)
        }
        _ => return Err(MetadataError::InvalidFeeAccount.into()),
    };

    let dest_starting_lamports = dest_info.lamports();
    **dest_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(fee_amount)
        .ok_or(MetadataError::NumericalOverflowError)?;
    **account_info.lamports.borrow_mut() = rent_amount;

    // Clear fee flag.
    clear_fee_flag(account_info)?;

    Ok(())
}
