
mod entrypoint;
pub mod processor;
pub mod state;
pub mod instruction;
pub mod error;
pub mod pod;

pub mod equality_proof;
pub mod transfer_proof;

// TODO: use spl-zk-token-sdk
#[macro_use]
pub(crate) mod macros;
pub mod errors;
pub mod encryption;
pub mod transcript;
pub mod zk_token_elgamal;

solana_program::declare_id!("5RQcngAmGWhNM3zZPQiegc4e11WKvsUzGCd3aVXUwjnx");
