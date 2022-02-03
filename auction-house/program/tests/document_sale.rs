#![cfg(feature = "test-bpf")]
mod utils;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};

use mpl_auction_house::{pda::*, AuctionHouse};
use mpl_testing_utils::solana::{airdrop, create_associated_token_account, create_mint};
use mpl_testing_utils::utils::Metadata;
use solana_program_test::*;
use solana_sdk::{
    instruction::{Instruction, InstructionError},
    sysvar,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use spl_token;
use mpl_auction_house::{
  pda::{
      self, find_auction_house_address, find_auction_house_fee_account_address,
      find_auction_house_treasury_address
  },
};
use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn document_sale_success() {
  let mut context = auction_house_program_test().start_with_context().await;
  // Payer Wallet

  let (ah, ahkey) = existing_auction_house_test_context(&mut context)
      .await
      .unwrap();

  ()
}