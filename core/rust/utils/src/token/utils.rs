use arrayref::{array_ref, array_refs};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_option::COption, pubkey::Pubkey,
};

/// Unpacks COption from a slice, taken from token program
fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 32];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}

/// Cheap method to just grab owner Pubkey from token account, instead of deserializing entire thing
pub fn get_owner_from_token_account(
    token_account_info: &AccountInfo,
) -> Result<Pubkey, ProgramError> {
    // TokenAccount layout:   mint(32), owner(32), ...
    let data = token_account_info.try_borrow_data()?;
    let owner_data = array_ref![data, 32, 32];
    Ok(Pubkey::new_from_array(*owner_data))
}

pub fn get_mint_authority(account_info: &AccountInfo) -> Result<COption<Pubkey>, ProgramError> {
    // In token program, 36, 8, 1, 1 is the layout, where the first 36 is mint_authority
    // so we start at 0.
    let data = account_info.try_borrow_data().unwrap();
    let authority_bytes = array_ref![data, 0, 36];

    unpack_coption_key(authority_bytes)
}

pub fn get_mint_freeze_authority(
    account_info: &AccountInfo,
) -> Result<COption<Pubkey>, ProgramError> {
    let data = account_info.try_borrow_data().unwrap();
    let authority_bytes = array_ref![data, 36 + 8 + 1 + 1, 36];

    unpack_coption_key(authority_bytes)
}

/// cheap method to just get supply off a mint without unpacking whole object
pub fn get_mint_supply(account_info: &AccountInfo) -> Result<u64, ProgramError> {
    // In token program, 36, 8, 1, 1 is the layout, where the first 8 is supply u64.
    // so we start at 36.
    let data = account_info.try_borrow_data()?;

    // If we don't check this and an empty account is passed in, we get a panic when
    // the array_ref! macro tries to index into the data.
    if data.is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    let bytes = array_ref![data, 36, 8];

    Ok(u64::from_le_bytes(*bytes))
}

/// cheap method to just get supply off a mint without unpacking whole object
pub fn get_mint_decimals(account_info: &AccountInfo) -> Result<u8, ProgramError> {
    // In token program, 36, 8, 1, 1, is the layout, where the first 1 is decimals u8.
    // so we start at 36.
    let data = account_info.try_borrow_data()?;

    // If we don't check this and an empty account is passed in, we get a panic when
    // we try to index into the data.
    if data.is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(data[44])
}
