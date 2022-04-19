pub use mpl_token_metadata::state::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH};

use crate::validate::errors::ValidateError;

pub fn check_name(name: &str) -> Result<(), ValidateError> {
    if name.len() > MAX_NAME_LENGTH {
        return Err(ValidateError::NameTooLong);
    }
    Ok(())
}

pub fn check_symbol(symbol: &str) -> Result<(), ValidateError> {
    if symbol.len() > MAX_SYMBOL_LENGTH {
        return Err(ValidateError::SymbolTooLong);
    }
    Ok(())
}

pub fn check_url(url: &str) -> Result<(), ValidateError> {
    if url.len() > MAX_URI_LENGTH {
        return Err(ValidateError::UrlTooLong);
    }
    Ok(())
}

pub fn check_seller_fee_basis_points(seller_fee_basis_points: u16) -> Result<(), ValidateError> {
    if seller_fee_basis_points > 10000 {
        return Err(ValidateError::InvalidCreatorShare);
    }
    Ok(())
}
