#![cfg(feature = "test-bpf")]
mod utils;

use mpl_token_metadata::state::{UseMethod, Uses};

use mpl_token_metadata::{
    error::MetadataError,
    id, instruction,
    state::{Key, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use num_traits::FromPrimitive;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use utils::*;
mod uses {

    use super::*;
    #[tokio::test]
    async fn success() {}

    #[tokio::test]
    async fn success_and_burn() {}

    #[tokio::test]
    async fn fail_out_of_uses() {}
}
