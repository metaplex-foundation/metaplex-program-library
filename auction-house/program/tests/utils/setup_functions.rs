use std::{env, error};

use super::{
    constants::{AUCTION_HOUSE, FEE_PAYER, TREASURY},
    helpers::{
        derive_auction_house_fee_account_key, derive_auction_house_key,
        derive_auction_house_treasury_key,
    },
};
use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{read_keypair_file, Keypair, Signature},
        system_program, sysvar,
    },
    Client, ClientError, Cluster, Program,
};
use anchor_lang::prelude::Pubkey;
use mpl_auction_house::{
    accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction,
    AuctionHouse,
};

pub fn setup_payer_wallet() -> Keypair {
    let wallet_path = match env::var("LOCALNET_PAYER_WALLET") {
        Ok(val) => val,
        Err(_) => shellexpand::tilde("~/.config/solana/id.json").to_string(),
    };

    read_keypair_file(wallet_path).unwrap()
}

pub fn setup_client(payer_wallet: Keypair) -> Client {
    Client::new_with_options(
        Cluster::Localnet,
        payer_wallet,
        CommitmentConfig::processed(),
    )
}

pub fn setup_auction_house(
    program: &Program,
    authority: &Pubkey,
    mint_key: &Pubkey,
) -> Result<Pubkey, ClientError> {
    let seller_fee_basis_points: u16 = 100;

    let twd_key = program.payer();
    let fwd_key = program.payer();
    let tdw_ata = twd_key;

    let (auction_house_key, bump) = derive_auction_house_key(authority, mint_key);

    let (auction_fee_account_key, fee_payer_bump) =
        derive_auction_house_fee_account_key(&auction_house_key);

    let (auction_house_treasury_key, treasury_bump) =
        derive_auction_house_treasury_key(&auction_house_key);

    program
        .request()
        .accounts(mpl_auction_house_accounts::CreateAuctionHouse {
            treasury_mint: *mint_key,
            payer: program.payer(),
            authority: *authority,
            fee_withdrawal_destination: fwd_key,
            treasury_withdrawal_destination: tdw_ata,
            treasury_withdrawal_destination_owner: twd_key,
            auction_house: auction_house_key,
            auction_house_fee_account: auction_fee_account_key,
            auction_house_treasury: auction_house_treasury_key,
            token_program: spl_token::id(),
            system_program: system_program::id(),
            ata_program: spl_associated_token_account::id(),
            rent: sysvar::rent::id(),
        })
        .args(mpl_auction_house_instruction::CreateAuctionHouse {
            bump,
            fee_payer_bump,
            treasury_bump,
            seller_fee_basis_points,
            requires_sign_off: false,
            can_change_sale_price: true,
        })
        .send()
        .unwrap();

    Ok(auction_house_key)
}
