use std::{borrow::Borrow, env, io};

use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_program, sysvar,
};
use anchor_lang::*;

use mpl_auction_house::{
    pda::{
        find_auction_house_address, find_auction_house_fee_account_address,
        find_auction_house_treasury_address,
    },
    AuctionHouse,
};
use solana_program_test::*;
use solana_sdk::{instruction::Instruction, transaction::Transaction, transport::TransportError};

pub fn auction_house_program_test<'a>() -> ProgramTest {
    let mut program = ProgramTest::new("mpl_auction_house", mpl_auction_house::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    program
}

pub async fn create_auction_house(
    context: &mut ProgramTestContext,
    payer_wallet: &Keypair,
    twd_key: &Pubkey,
    fwd_key: &Pubkey,
    t_mint_key: &Pubkey,
    tdw_ata: &Pubkey,
    authority: &Pubkey,
    auction_house_key: &Pubkey,
    auction_house_key_bump: u8,
    auction_fee_account_key: &Pubkey,
    auction_fee_account_key_bump: u8,
    auction_house_treasury_key: &Pubkey,
    auction_house_treasury_key_bump: u8,
    seller_fee_basis_points: u16,
    requires_sign_off: bool,
    can_change_sale_price: bool,
) -> Result<Pubkey, TransportError> {
    let accounts = mpl_auction_house::accounts::CreateAuctionHouse {
        treasury_mint: *t_mint_key,
        payer: payer_wallet.pubkey(),
        authority: *authority,
        fee_withdrawal_destination: *fwd_key,
        treasury_withdrawal_destination: *tdw_ata,
        treasury_withdrawal_destination_owner: *twd_key,
        auction_house: *auction_house_key,
        auction_house_fee_account: *auction_fee_account_key,
        auction_house_treasury: *auction_house_treasury_key,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = mpl_auction_house::instruction::CreateAuctionHouse {
        bump: auction_house_key_bump,
        fee_payer_bump: auction_fee_account_key_bump,
        treasury_bump: auction_house_treasury_key_bump,
        seller_fee_basis_points,
        requires_sign_off,
        can_change_sale_price,
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer_wallet.pubkey()),
        &[payer_wallet],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .map(|_| auction_house_key.clone())
}

pub async fn sell(auction_house_key: &Pubkey, payer_wallet: &Keypair, mint: &Pubkey) {
    let accounts = mpl_auction_house::accounts::Sell {
        auction_house: *auction_house_key,
        payer: payer_wallet.pubkey(),
        mint: *mint,
        wallet: payer_wallet.pubkey(),
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: mpl_auction_house::instruction::Sell {
            bump: 0,
        }
        .data(),
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer_wallet.pubkey()),
        &[payer_wallet],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();
}

pub async fn existing_auction_house_test_context(
    context: &mut ProgramTestContext,
) -> Result<(AuctionHouse, Pubkey), TransportError> {
    let twd_key = context.payer.pubkey().clone();
    let fwd_key = context.payer.pubkey().clone();
    let t_mint_key = spl_token::native_mint::id();
    let tdw_ata = twd_key;
    let seller_fee_basis_points: u16 = 100;
    let authority = Keypair::new().pubkey();
    let payer = Keypair::from_bytes(&context.payer.to_bytes().clone())
        .map_err(|e| TransportError::IoError(io::Error::new(io::ErrorKind::InvalidData, e)))?;
    // Derive Auction House Key
    let (auction_house_address, bump) = find_auction_house_address(&authority, &t_mint_key);
    let (auction_fee_account_key, fee_payer_bump) =
        find_auction_house_fee_account_address(&auction_house_address);
    // Derive Auction House Treasury Key
    let (auction_house_treasury_key, treasury_bump) =
        find_auction_house_treasury_address(&auction_house_address);
    let auction_house = create_auction_house(
        context,
        &payer,
        &twd_key,
        &fwd_key,
        &t_mint_key,
        &tdw_ata,
        &authority,
        &auction_house_address,
        bump,
        &auction_fee_account_key,
        fee_payer_bump,
        &auction_house_treasury_key,
        treasury_bump,
        seller_fee_basis_points,
        false,
        false,
    );

    let auction_house_account = auction_house.await.unwrap();

    let auction_house_acc = context
        .banks_client
        .get_account(auction_house_account)
        .await?
        .expect("account empty");

    let auction_house_data = AuctionHouse::try_deserialize(&mut auction_house_acc.data.as_ref())
        .map_err(|e| TransportError::IoError(io::Error::new(io::ErrorKind::InvalidData, e)))?;
    return Ok((auction_house_data, auction_house_address));
}
