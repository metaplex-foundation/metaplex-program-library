use std::{io};

use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
};
use anchor_lang::*;
use mpl_auction_house::{pda::{
    find_auction_house_address, find_auction_house_fee_account_address,
    find_auction_house_treasury_address, find_program_as_signer_address,
    find_trade_state_address,
}, AuctionHouse};
use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_program_test::*;
use solana_sdk::{instruction::Instruction, transaction::Transaction, transport::TransportError};
use spl_associated_token_account::get_associated_token_address;
use mpl_auction_house::pda::find_escrow_payment_address;

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
        authority: payer_wallet.pubkey(),
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

pub fn deposit(
    context: &mut ProgramTestContext,
    ahkey: &Pubkey,
    ah: &AuctionHouse,
    test_metadata: &Metadata,
    buyer: &Keypair,
    sale_price: u64,
)-> (mpl_auction_house::accounts::Deposit, Transaction) {
    let seller_token_account =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let (_buyer_trade_state, _sts_bump) = find_trade_state_address(
        &buyer.pubkey(),
        &ahkey,
        &seller_token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        sale_price,
        1,
    );
    let (escrow, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer.pubkey());
    let accounts = mpl_auction_house::accounts::Deposit {
        wallet: buyer.pubkey(),
        authority: ah.authority,
        auction_house: *ahkey,
        auction_house_fee_account: ah.auction_house_fee_account,
        token_program: spl_token::id(),
        treasury_mint: ah.treasury_mint,
        payment_account: buyer.pubkey(),
        transfer_authority: buyer.pubkey(),
        system_program: solana_program::system_program::id(),
        rent: sysvar::rent::id(),
        escrow_payment_account: escrow
    };
    let account_metas = accounts.to_account_metas(None);

    let data = mpl_auction_house::instruction::Deposit {
        amount: sale_price,
        escrow_payment_bump: escrow_bump,
    }.data();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data,
        accounts: account_metas,
    };

    (accounts,Transaction::new_signed_with_payer(
        &[instruction],
        Some(&buyer.pubkey()),
        &[buyer],
        context.last_blockhash,
    ))
}

pub fn buy(
    context: &mut ProgramTestContext,
    ahkey: &Pubkey,
    ah: &AuctionHouse,
    test_metadata: &Metadata,
    buyer: &Keypair,
    sale_price: u64,
)-> (mpl_auction_house::accounts::Buy, Transaction) {
    let seller_token_account =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let (buyer_trade_state, sts_bump) = find_trade_state_address(
        &buyer.pubkey(),
        &ahkey,
        &seller_token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        sale_price,
        1,
    );
    let (escrow, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer.pubkey());

    let accounts = mpl_auction_house::accounts::Buy {
        wallet: buyer.pubkey(),
        token_account: seller_token_account,
        metadata: test_metadata.pubkey,
        authority: ah.authority,
        auction_house: *ahkey,
        auction_house_fee_account: ah.auction_house_fee_account,
        buyer_trade_state,
        token_program: spl_token::id(),
        treasury_mint: ah.treasury_mint,
        payment_account: buyer.pubkey(),
        transfer_authority: buyer.pubkey(),
        system_program: solana_program::system_program::id(),
        rent: sysvar::rent::id(),
        escrow_payment_account: escrow
    };
    let account_metas = accounts.to_account_metas(None);

    let buy_ix = mpl_auction_house::instruction::Buy {
        trade_state_bump: sts_bump,
        escrow_payment_bump: escrow_bump,
        token_size: 1,
        buyer_price: sale_price,
    };
    let data = buy_ix.data();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data,
        accounts: account_metas,
    };

    (accounts, Transaction::new_signed_with_payer(
        &[instruction],
        Some(&buyer.pubkey()),
        &[buyer],
        context.last_blockhash,
    ))
}

pub fn sell(
    context: &mut ProgramTestContext,
    ahkey: &Pubkey,
    ah: &AuctionHouse,
    test_metadata: &Metadata,
    sale_price: u64,
) -> (mpl_auction_house::accounts::Sell,Transaction) {
    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let (seller_trade_state, sts_bump) = find_trade_state_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        sale_price,
        1,
    );
    let (free_seller_trade_state, free_sts_bump) = find_trade_state_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        0,
        1,
    );
    let (pas, pas_bump) = find_program_as_signer_address();

    let accounts = mpl_auction_house::accounts::Sell {
        wallet: test_metadata.token.pubkey(),
        token_account: token,
        metadata: test_metadata.pubkey,
        authority: ah.authority,
        auction_house: *ahkey,
        auction_house_fee_account: ah.auction_house_fee_account,
        seller_trade_state,
        free_seller_trade_state,
        token_program: spl_token::id(),
        system_program: solana_program::system_program::id(),
        program_as_signer: pas,
        rent: sysvar::rent::id(),
    };
    let account_metas = accounts.to_account_metas(None);

    let data = mpl_auction_house::instruction::Sell {
        trade_state_bump: sts_bump,
        _free_trade_state_bump: free_sts_bump,
        _program_as_signer_bump: pas_bump,
        token_size: 1,
        buyer_price: sale_price,
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data,
        accounts: account_metas,
    };

    (accounts, Transaction::new_signed_with_payer(
        &[instruction],
        Some(&test_metadata.token.pubkey()),
        &[&test_metadata.token],
        context.last_blockhash,
    ))
}

pub async fn existing_auction_house_test_context(
    context: &mut ProgramTestContext,
) -> Result<(AuctionHouse, Pubkey, Keypair), TransportError> {
    let twd_key = context.payer.pubkey().clone();
    let fwd_key = context.payer.pubkey().clone();
    let t_mint_key = spl_token::native_mint::id();
    let tdw_ata = twd_key;
    let seller_fee_basis_points: u16 = 100;
    let authority = Keypair::new();
    airdrop(context, &authority.pubkey(), 10_000_000_000).await?;
    // Derive Auction House Key
    let (auction_house_address, bump) =
        find_auction_house_address(&authority.pubkey(), &t_mint_key);
    let (auction_fee_account_key, fee_payer_bump) =
        find_auction_house_fee_account_address(&auction_house_address);
    // Derive Auction House Treasury Key
    let (auction_house_treasury_key, treasury_bump) =
        find_auction_house_treasury_address(&auction_house_address);
    let auction_house = create_auction_house(
        context,
        &authority,
        &twd_key,
        &fwd_key,
        &t_mint_key,
        &tdw_ata,
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
    return Ok((auction_house_data, auction_house_address,authority));
}
