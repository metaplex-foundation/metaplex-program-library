//! Module provide tests for `Buy` instruction.

mod utils;

use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        signer::{keypair::read_keypair_file, Signer},
        system_program, sysvar,
    },
    Client, Cluster,
};
use std::{env, error};

#[test]
fn success() -> Result<(), Box<dyn error::Error>> {
    const BUYER_PRICE: u64 = 1;
    const TOKEN_SIZE: u64 = 1;

    // Load `Localnet` keypair
    let wallet = read_keypair_file(env::var("LOCALNET_PAYER_WALLET")?)?;
    let wallet_pubkey = wallet.pubkey();

    // Initialize anchor RPC `Client`
    let client = Client::new(Cluster::Localnet, utils::clone_keypair(&wallet));

    // Initialize vanilla `RpcClient`
    let connection = RpcClient::new(Cluster::Localnet.url().to_string());

    // Initialize `Program` handle
    let program = client.program(mpl_auction_house::id());

    // Derive native(wrapped) sol mint
    let treasury_mint = spl_token::native_mint::id();

    // Token mint for `TokenMetadata`.
    let token_mint = utils::create_mint(&connection, &wallet)?;

    // Derive / Create associated token account
    let token_account =
        utils::create_associated_token_account(&connection, &wallet, &token_mint.pubkey())?;

    // Mint tokens
    utils::mint_to(
        &connection,
        &wallet,
        &token_mint.pubkey(),
        &token_account,
        1,
    )?;

    // Derive `AuctionHouse` address
    let (auction_house, _) = utils::find_auction_house_address(&wallet_pubkey, &treasury_mint);

    // Derive `AuctionHouse` fee account
    let (auction_house_fee_account, _) =
        utils::find_auction_house_fee_account_address(&auction_house);

    // Derive buyer trade state address
    let (buyer_trade_state, buyer_trade_state_bump) = utils::find_trade_state_address(
        &wallet_pubkey,
        &auction_house,
        &token_account,
        &treasury_mint,
        &token_mint.pubkey(),
        BUYER_PRICE,
        TOKEN_SIZE,
    );

    // Derive escrow payment address
    let (escrow_payment_account, escrow_payment_bump) =
        utils::find_escrow_payment_address(&auction_house, &wallet_pubkey);

    // Create `TokenMetadata`
    let metadata = utils::create_token_metadata(
        &connection,
        &wallet,
        &token_mint.pubkey(),
        String::from("TEST"),
        String::from("TST"),
        String::from("https://github.com"),
        5000,
    )?;

    // Transfer enough lamports to create seller trade state
    utils::transfer_lamports(&connection, &wallet, &auction_house_fee_account, 10000000)?;

    // Perform RPC instruction request
    program
        .request()
        .accounts(mpl_auction_house::accounts::Buy {
            auction_house,
            auction_house_fee_account,
            authority: wallet_pubkey,
            buyer_trade_state,
            escrow_payment_account,
            metadata,
            payment_account: wallet.pubkey(),
            rent: sysvar::rent::id(),
            system_program: system_program::id(),
            token_account,
            token_program: spl_token::id(),
            transfer_authority: wallet_pubkey,
            treasury_mint,
            wallet: wallet_pubkey,
        })
        .args(mpl_auction_house::instruction::Buy {
            buyer_price: BUYER_PRICE,
            escrow_payment_bump,
            token_size: TOKEN_SIZE,
            trade_state_bump: buyer_trade_state_bump,
        })
        .send()?;

    assert_eq!(
        connection.get_account_data(&buyer_trade_state)?[0],
        buyer_trade_state_bump
    );

    Ok(())
}
