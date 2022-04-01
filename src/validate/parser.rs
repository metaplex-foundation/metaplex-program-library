use crate::validate::errors::ValidateError;

pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_SYMBOL_LENGTH: usize = 10;
pub const MAX_URI_LENGTH: usize = 200;

pub fn check_name(name: &str) -> Result<(), ValidateError> {
    if name.chars().count() > 32 {
        return Err(ValidateError::NameTooLong);
    }
    Ok(())
}

pub fn check_symbol(symbol: &str) -> Result<(), ValidateError> {
    if symbol.chars().count() > 10 {
        return Err(ValidateError::SymbolTooLong);
    }
    Ok(())
}

pub fn check_url(url: &str) -> Result<(), ValidateError> {
    if url.chars().count() > 200 {
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
