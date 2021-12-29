//! Module provide utilities for testing.

#![allow(unused)]

pub mod constants;
pub mod helpers;
pub mod setup_functions;

use std::{env, str::FromStr};

use anchor_client::{
    solana_client::{client_error::ClientError, rpc_client::RpcClient},
    solana_sdk::{
        program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
        system_program, sysvar, transaction::Transaction,
    },
    Program,
};
use constants::{AUCTION_HOUSE, FEE_PAYER, SIGNER, TREASURY};

// const PREFIX: &str = "auction_house";
// const FEE_PAYER: &str = "fee_payer";
// const TREASURY: &str = "treasury";
// const SIGNER: &str = "signer";

/// Return `spl_token` token account.
pub fn get_token_account(
    connection: &RpcClient,
    token_account: &Pubkey,
) -> Result<spl_token::state::Account, ClientError> {
    let data = connection.get_account_data(token_account)?;
    Ok(spl_token::state::Account::unpack(&data).unwrap())
}

/// Perform native lamports transfer.
pub fn transfer_lamports(
    connection: &RpcClient,
    wallet: &Keypair,
    to: &Pubkey,
    amount: u64,
) -> Result<(), ClientError> {
    let (recent_blockhash, _) = connection.get_recent_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(&wallet.pubkey(), to, amount)],
        Some(&wallet.pubkey()),
        &[wallet],
        recent_blockhash,
    );

    connection.send_and_confirm_transaction(&tx)?;

    Ok(())
}

/// Create new `TokenMetadata` using `RpcClient`.
pub fn create_token_metadata(
    connection: &RpcClient,
    wallet: &Keypair,
    mint: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
) -> Result<Pubkey, ClientError> {
    let pid = match env::var("TOKEN_METADATA_PID") {
        Ok(val) => val,
        Err(_) => mpl_token_metadata::id().to_string(),
    };

    let program_id = Pubkey::from_str(&pid).unwrap();

    let (recent_blockhash, _) = connection.get_recent_blockhash()?;

    let (metadata_account, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            program_id.as_ref(),
            mint.as_ref(),
        ],
        &program_id,
    );

    let tx = Transaction::new_signed_with_payer(
        &[mpl_token_metadata::instruction::create_metadata_accounts(
            program_id,
            metadata_account,
            *mint,
            wallet.pubkey(),
            wallet.pubkey(),
            wallet.pubkey(),
            name,
            symbol,
            uri,
            None,
            seller_fee_basis_points,
            false,
            false,
        )],
        Some(&wallet.pubkey()),
        &[wallet],
        recent_blockhash,
    );

    connection.send_and_confirm_transaction(&tx)?;
    Ok(metadata_account)
}

/// Mint tokens.
pub fn mint_to(
    connection: &RpcClient,
    wallet: &Keypair,
    mint: &Pubkey,
    to: &Pubkey,
    amount: u64,
) -> Result<(), ClientError> {
    let (recent_blockhash, _) = connection.get_recent_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            to,
            &wallet.pubkey(),
            &[&wallet.pubkey()],
            amount,
        )
        .unwrap()],
        Some(&wallet.pubkey()),
        &[wallet],
        recent_blockhash,
    );

    connection.send_and_confirm_transaction(&tx)?;

    Ok(())
}

/// Create new `AuctionHouse` using `Program`.
pub fn create_auction_house(
    program: &Program,
    treasury_mint: &Pubkey,
    wallet: &Pubkey,
    fee_withdrawal_destination: &Pubkey,
    treasury_withdrawal_destination: &Pubkey,
    can_change_sale_price: bool,
    requires_sign_off: bool,
    seller_fee_basis_points: u16,
) -> Result<(), ClientError> {
    let (auction_house, bump) = find_auction_house_address(wallet, treasury_mint);
    let (auction_house_fee_account, fee_payer_bump) =
        find_auction_house_fee_account_address(&auction_house);
    let (auction_house_treasury, treasury_bump) =
        find_auction_house_treasury_address(&auction_house);

    program
        .request()
        .accounts(mpl_auction_house::accounts::CreateAuctionHouse {
            treasury_mint: *treasury_mint,
            payer: *wallet,
            authority: *wallet,
            fee_withdrawal_destination: *fee_withdrawal_destination,
            treasury_withdrawal_destination: *treasury_withdrawal_destination,
            treasury_withdrawal_destination_owner: *wallet,
            auction_house,
            auction_house_fee_account,
            auction_house_treasury,
            token_program: spl_token::id(),
            ata_program: spl_associated_token_account::id(),
            rent: sysvar::rent::id(),
            system_program: system_program::id(),
        })
        .args(mpl_auction_house::instruction::CreateAuctionHouse {
            bump,
            can_change_sale_price,
            fee_payer_bump,
            requires_sign_off,
            seller_fee_basis_points,
            treasury_bump,
        })
        .send()
        .unwrap();

    Ok(())
}

/// Return `Clone` of provided `Keypair`.
pub fn clone_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}

/// Create and return new mint.
pub fn create_mint(connection: &RpcClient, wallet: &Keypair) -> Result<Keypair, ClientError> {
    let (recent_blockhash, _) = connection.get_recent_blockhash()?;
    let mint = Keypair::new();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &wallet.pubkey(),
                &mint.pubkey(),
                connection.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &wallet.pubkey(),
                None,
                9,
            )
            .unwrap(),
        ],
        Some(&wallet.pubkey()),
        &[wallet, &mint],
        recent_blockhash,
    );

    connection.send_and_confirm_transaction(&tx)?;

    Ok(mint)
}

/// Create and return new associated token account.
pub fn create_associated_token_account(
    connection: &RpcClient,
    wallet: &Keypair,
    token_mint: &Pubkey,
) -> Result<Pubkey, ClientError> {
    let (recent_blockhash, _) = connection.get_recent_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[
            spl_associated_token_account::create_associated_token_account(
                &wallet.pubkey(),
                &wallet.pubkey(),
                token_mint,
            ),
        ],
        Some(&wallet.pubkey()),
        &[wallet],
        recent_blockhash,
    );

    connection.send_and_confirm_transaction(&tx)?;

    Ok(spl_associated_token_account::get_associated_token_address(
        &wallet.pubkey(),
        token_mint,
    ))
}

/// Create new token account.
pub fn create_token_account(
    connection: &RpcClient,
    wallet: &Keypair,
    token_account: &Keypair,
    token_mint: &Pubkey,
    owner: &Pubkey,
) -> Result<(), ClientError> {
    let (recent_blockhash, _) = connection.get_recent_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &wallet.pubkey(),
                &token_account.pubkey(),
                spl_token::state::Account::LEN as u64,
                connection
                    .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)?,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &token_account.pubkey(),
                token_mint,
                owner,
            )
            .unwrap(),
        ],
        Some(&wallet.pubkey()),
        &[wallet, token_account],
        recent_blockhash,
    );

    connection.send_and_confirm_transaction(&tx)?;

    Ok(())
}

/// Return escrow payment `Pubkey` address and bump seed.
pub fn find_escrow_payment_address(auction_house: &Pubkey, wallet: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            AUCTION_HOUSE.as_bytes(),
            auction_house.as_ref(),
            wallet.as_ref(),
        ],
        &mpl_auction_house::id(),
    )
}

/// Return `AuctionHouse` `Pubkey` address and bump seed.
pub fn find_auction_house_address(authority: &Pubkey, treasury_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            AUCTION_HOUSE.as_bytes(),
            authority.as_ref(),
            treasury_mint.as_ref(),
        ],
        &mpl_auction_house::id(),
    )
}

/// Return `AuctionHouse` fee account `Pubkey` address and bump seed.
pub fn find_auction_house_fee_account_address(auction_house: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            AUCTION_HOUSE.as_bytes(),
            auction_house.as_ref(),
            FEE_PAYER.as_bytes(),
        ],
        &mpl_auction_house::id(),
    )
}

/// Return trade state `Pubkey` address and bump seed.
pub fn find_trade_state_address(
    wallet: &Pubkey,
    auction_house: &Pubkey,
    token_account: &Pubkey,
    treasury_mint: &Pubkey,
    token_mint: &Pubkey,
    price: u64,
    token_size: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            AUCTION_HOUSE.as_bytes(),
            wallet.as_ref(),
            auction_house.as_ref(),
            token_account.as_ref(),
            treasury_mint.as_ref(),
            token_mint.as_ref(),
            &price.to_le_bytes(),
            &token_size.to_le_bytes(),
        ],
        &mpl_auction_house::id(),
    )
}

/// Return program as signer `Pubkey` address and bump seed.
pub fn find_program_as_signer_address() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[AUCTION_HOUSE.as_bytes(), SIGNER.as_bytes()],
        &mpl_auction_house::id(),
    )
}

/// Return `AuctionHouse` treasury `Pubkey` address and bump seed.
pub fn find_auction_house_treasury_address(auction_house: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            AUCTION_HOUSE.as_bytes(),
            auction_house.as_ref(),
            TREASURY.as_bytes(),
        ],
        &mpl_auction_house::id(),
    )
}
