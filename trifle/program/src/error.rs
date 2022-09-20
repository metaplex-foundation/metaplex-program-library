use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrifleError {
    #[error("Invalid account")]
    InvalidAccount,
}
