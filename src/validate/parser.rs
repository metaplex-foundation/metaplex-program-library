use anchor_client::solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::validate::errors::ValidateError;
use crate::validate::format::Creator;

pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_SYMBOL_LENGTH: usize = 10;
pub const MAX_URI_LENGTH: usize = 200;

pub fn check_name(name: &String) -> Result<(), ValidateError> {
    if name.chars().count() > 32 {
        return Err(ValidateError::NameTooLong);
    }
    Ok(())
}

pub fn check_symbol(symbol: &String) -> Result<(), ValidateError> {
    if symbol.chars().count() > 10 {
        return Err(ValidateError::SymbolTooLong);
    }
    Ok(())
}

pub fn check_url(url: &String) -> Result<(), ValidateError> {
    if url.chars().count() > 200 {
        return Err(ValidateError::UrlTooLong);
    }
    Ok(())
}

pub fn check_seller_fee_basis_points(seller_fee_basis_points: u32) -> Result<(), ValidateError> {
    if seller_fee_basis_points > 10000 {
        return Err(ValidateError::InvalidCreatorShare);
    }
    Ok(())
}

pub fn check_creators(creators: &Vec<Creator>) -> Result<(), ValidateError> {
    let mut sum = 0;
    for creator in creators {
        match Pubkey::from_str(&creator.address) {
            Ok(_) => (),
            Err(_) => {
                return Err(ValidateError::InvalidCreatorAddress(
                    creator.address.clone(),
                ))
            }
        }
        sum += creator.share;
    }
    if sum != 100 {
        return Err(ValidateError::InvalidCreatorShare);
    }
    Ok(())
}
