use std::io;

use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
};
use anchor_lang::*;
use mpl_auction_house::{
    pda::{
        find_auction_house_address, find_auction_house_fee_account_address,
        find_auction_house_treasury_address, find_auctioneer_pda,
        find_auctioneer_trade_state_address, find_escrow_payment_address,
        find_program_as_signer_address, find_trade_state_address,
    },
    AuctionHouse,
};
use mpl_auctioneer::pda::*;
use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use std::result::Result as StdResult;

use mpl_token_metadata::pda::find_metadata_account;
use solana_program_test::*;
use solana_sdk::{
    clock::UnixTimestamp, instruction::Instruction, transaction::Transaction,
    transport::TransportError,
};
use spl_associated_token_account::get_associated_token_address;

use crate::utils::helpers::default_scopes;

pub fn auctioneer_program_test<'a>() -> ProgramTest {
    let mut program = ProgramTest::new("mpl_auctioneer", mpl_auctioneer::id(), None);
    program.add_program("mpl_auction_house", mpl_auction_house::id(), None);
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
) -> StdResult<Pubkey, TransportError> {
    let create_accounts = mpl_auction_house::accounts::CreateAuctionHouse {
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

    let create_data = mpl_auction_house::instruction::CreateAuctionHouse {
        _bump: auction_house_key_bump,
        fee_payer_bump: auction_fee_account_key_bump,
        treasury_bump: auction_house_treasury_key_bump,
        seller_fee_basis_points,
        requires_sign_off,
        can_change_sale_price,
    }
    .data();

    let create_instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: create_data,
        accounts: create_accounts,
    };

    let (auctioneer_authority, _aa_bump) = find_auctioneer_authority_seeds(auction_house_key);
    let (auctioneer_pda, _) = find_auctioneer_pda(auction_house_key, &auctioneer_authority);

    let scopes = default_scopes();
    let delegate_accounts = mpl_auction_house::accounts::DelegateAuctioneer {
        auction_house: *auction_house_key,
        authority: payer_wallet.pubkey(),
        auctioneer_authority: auctioneer_authority,
        ah_auctioneer_pda: auctioneer_pda,
        system_program: system_program::id(),
    };

    let delegate_data = mpl_auction_house::instruction::DelegateAuctioneer {
        scopes: scopes.clone(),
    };

    let delegate_instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data: delegate_data.data(),
        accounts: delegate_accounts.to_account_metas(None),
    };

    let authorize_accounts = mpl_auctioneer::accounts::AuctioneerAuthorize {
        wallet: payer_wallet.pubkey(),
        auction_house: *auction_house_key,
        auctioneer_authority: auctioneer_authority,
        system_program: system_program::id(),
    };

    let authorize_data = mpl_auctioneer::instruction::Authorize {};

    let authorize_instruction = Instruction {
        program_id: mpl_auctioneer::id(),
        data: authorize_data.data(),
        accounts: authorize_accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[
            create_instruction,
            delegate_instruction,
            authorize_instruction,
        ],
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
) -> (mpl_auctioneer::accounts::AuctioneerDeposit, Transaction) {
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
    let (auctioneer_authority, aa_bump) = find_auctioneer_authority_seeds(ahkey);
    let (auctioneer_pda, _) = find_auctioneer_pda(ahkey, &auctioneer_authority);
    let accounts = mpl_auctioneer::accounts::AuctioneerDeposit {
        auction_house_program: mpl_auction_house::id(),
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
        escrow_payment_account: escrow,
        auctioneer_authority: auctioneer_authority,
        ah_auctioneer_pda: auctioneer_pda,
    };
    let account_metas = accounts.to_account_metas(None);

    let data = mpl_auctioneer::instruction::Deposit {
        amount: sale_price,
        escrow_payment_bump: escrow_bump,
        auctioneer_authority_bump: aa_bump,
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_auctioneer::id(),
        data: data,
        accounts: account_metas,
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&buyer.pubkey()),
            &[buyer],
            context.last_blockhash,
        ),
    )
}

pub fn buy(
    context: &mut ProgramTestContext,
    ahkey: &Pubkey,
    ah: &AuctionHouse,
    test_metadata: &Metadata,
    owner: &Pubkey,
    buyer: &Keypair,
    seller: &Pubkey,
    listing_config: &Pubkey,
    sale_price: u64,
) -> (mpl_auctioneer::accounts::AuctioneerBuy, Transaction) {
    let seller_token_account = get_associated_token_address(&owner, &test_metadata.mint.pubkey());
    let trade_state = find_trade_state_address(
        &buyer.pubkey(),
        &ahkey,
        &seller_token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        sale_price,
        1,
    );
    let (auctioneer_authority, aa_bump) = find_auctioneer_authority_seeds(ahkey);
    let (escrow, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer.pubkey());
    let (auctioneer_pda, _) = find_auctioneer_pda(ahkey, &auctioneer_authority);
    let (bts, bts_bump) = trade_state;
    let accounts = mpl_auctioneer::accounts::AuctioneerBuy {
        auction_house_program: mpl_auction_house::id(),
        listing_config: *listing_config,
        seller: *seller,
        wallet: buyer.pubkey(),
        token_account: seller_token_account,
        metadata: test_metadata.pubkey,
        authority: ah.authority,
        auction_house: *ahkey,
        auction_house_fee_account: ah.auction_house_fee_account,
        buyer_trade_state: bts,
        token_program: spl_token::id(),
        treasury_mint: ah.treasury_mint,
        payment_account: buyer.pubkey(),
        transfer_authority: buyer.pubkey(),
        system_program: solana_program::system_program::id(),
        rent: sysvar::rent::id(),
        escrow_payment_account: escrow,
        auctioneer_authority: auctioneer_authority,
        ah_auctioneer_pda: auctioneer_pda,
    };

    let account_metas = accounts.to_account_metas(None);

    let buy_ix = mpl_auctioneer::instruction::Buy {
        trade_state_bump: bts_bump,
        escrow_payment_bump: escrow_bump,
        auctioneer_authority_bump: aa_bump,
        token_size: 1,
        buyer_price: sale_price,
    };
    let data = buy_ix.data();

    let instruction = Instruction {
        program_id: mpl_auctioneer::id(),
        data,
        accounts: account_metas,
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&buyer.pubkey()),
            &[buyer],
            context.last_blockhash,
        ),
    )
}

pub fn execute_sale(
    context: &mut ProgramTestContext,
    listing_config: &Pubkey,
    ahkey: &Pubkey,
    ah: &AuctionHouse,
    authority: &Keypair,
    test_metadata: &Metadata,
    buyer: &Pubkey,
    seller: &Pubkey,
    token_account: &Pubkey,
    seller_trade_state: &Pubkey,
    buyer_trade_state: &Pubkey,
    token_size: u64,
    buyer_price: u64,
) -> (mpl_auctioneer::accounts::AuctioneerExecuteSale, Transaction) {
    let buyer_token_account = get_associated_token_address(&buyer, &test_metadata.mint.pubkey());

    let (program_as_signer, pas_bump) = find_program_as_signer_address();

    let (free_trade_state, free_sts_bump) = find_trade_state_address(
        &seller,
        &ahkey,
        &token_account,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        0,
        token_size,
    );

    let (escrow_payment_account, escrow_bump) = find_escrow_payment_address(&ahkey, &buyer);
    let (auctioneer_authority, aa_bump) = find_auctioneer_authority_seeds(ahkey);
    let (auctioneer_pda, _) = find_auctioneer_pda(ahkey, &auctioneer_authority);
    let execute_sale_accounts = mpl_auctioneer::accounts::AuctioneerExecuteSale {
        auction_house_program: mpl_auction_house::id(),
        listing_config: *listing_config,
        buyer: *buyer,
        seller: *seller,
        auction_house: *ahkey,
        token_account: *token_account,
        token_mint: test_metadata.mint.pubkey(),
        treasury_mint: ah.treasury_mint,
        metadata: test_metadata.pubkey,
        seller_trade_state: *seller_trade_state,
        buyer_trade_state: *buyer_trade_state,
        free_trade_state: free_trade_state,
        seller_payment_receipt_account: *seller,
        buyer_receipt_token_account: buyer_token_account,
        escrow_payment_account: escrow_payment_account,
        auction_house_fee_account: ah.auction_house_fee_account,
        auction_house_treasury: ah.auction_house_treasury,
        program_as_signer: program_as_signer,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
        authority: authority.pubkey(),
        auctioneer_authority: auctioneer_authority,
        ah_auctioneer_pda: auctioneer_pda,
    };

    let execute_sale_account_metas = execute_sale_accounts.to_account_metas(None);

    let execute_sale_instruction = Instruction {
        program_id: mpl_auctioneer::id(),
        data: mpl_auctioneer::instruction::ExecuteSale {
            escrow_payment_bump: escrow_bump,
            free_trade_state_bump: free_sts_bump,
            program_as_signer_bump: pas_bump,
            auctioneer_authority_bump: aa_bump,
            token_size,
            buyer_price,
        }
        .data(),
        accounts: execute_sale_account_metas,
    };

    let tx = Transaction::new_signed_with_payer(
        &[execute_sale_instruction],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    );

    (execute_sale_accounts, tx)
}

pub fn sell_mint(
    context: &mut ProgramTestContext,
    ahkey: &Pubkey,
    ah: &AuctionHouse,
    test_metadata_mint: &Pubkey,
    seller: &Keypair,
    start_time: UnixTimestamp,
    end_time: UnixTimestamp,
    reserve_price: Option<u64>,
    min_bid_increment: Option<u64>,
    time_ext_period: Option<u32>,
    time_ext_delta: Option<u32>,
    allow_high_bid_cancel: Option<bool>,
) -> (
    (mpl_auctioneer::accounts::AuctioneerSell, Pubkey),
    Transaction,
) {
    let token = get_associated_token_address(&seller.pubkey(), &test_metadata_mint);
    let (metadata, _) = find_metadata_account(test_metadata_mint);
    let (seller_trade_state, sts_bump) = find_auctioneer_trade_state_address(
        &seller.pubkey(),
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata_mint,
        1,
    );
    let (free_seller_trade_state, free_sts_bump) = find_trade_state_address(
        &seller.pubkey(),
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata_mint,
        0,
        1,
    );

    let (listing_config_address, _list_bump) = find_listing_config_address(
        &seller.pubkey(),
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata_mint,
        1,
    );

    let (pas, pas_bump) = find_program_as_signer_address();
    let (auctioneer_authority, aa_bump) = find_auctioneer_authority_seeds(ahkey);
    let (auctioneer_pda, _) = find_auctioneer_pda(ahkey, &auctioneer_authority);

    let accounts = mpl_auctioneer::accounts::AuctioneerSell {
        auction_house_program: mpl_auction_house::id(),
        listing_config: listing_config_address,
        wallet: seller.pubkey(),
        token_account: token,
        metadata,
        authority: ah.authority,
        auction_house: *ahkey,
        auction_house_fee_account: ah.auction_house_fee_account,
        seller_trade_state,
        free_seller_trade_state,
        token_program: spl_token::id(),
        system_program: solana_program::system_program::id(),
        program_as_signer: pas,
        rent: sysvar::rent::id(),
        auctioneer_authority: auctioneer_authority,
        ah_auctioneer_pda: auctioneer_pda,
    };
    let account_metas = accounts.to_account_metas(None);

    let data = mpl_auctioneer::instruction::Sell {
        trade_state_bump: sts_bump,
        free_trade_state_bump: free_sts_bump,
        program_as_signer_bump: pas_bump,
        auctioneer_authority_bump: aa_bump,
        token_size: 1,
        start_time,
        end_time,
        reserve_price,
        min_bid_increment,
        time_ext_period,
        time_ext_delta,
        allow_high_bid_cancel,
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_auctioneer::id(),
        data,
        accounts: account_metas,
    };

    (
        (accounts, listing_config_address),
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&seller.pubkey()),
            &[seller],
            context.last_blockhash,
        ),
    )
}

pub fn sell(
    context: &mut ProgramTestContext,
    ahkey: &Pubkey,
    ah: &AuctionHouse,
    test_metadata: &Metadata,
    start_time: UnixTimestamp,
    end_time: UnixTimestamp,
    reserve_price: Option<u64>,
    min_bid_increment: Option<u64>,
    time_ext_period: Option<u32>,
    time_ext_delta: Option<u32>,
    allow_high_bid_cancel: Option<bool>,
) -> (
    (mpl_auctioneer::accounts::AuctioneerSell, Pubkey),
    Transaction,
) {
    let token =
        get_associated_token_address(&test_metadata.token.pubkey(), &test_metadata.mint.pubkey());
    let (seller_trade_state, sts_bump) = find_auctioneer_trade_state_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
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

    let (listing_config_address, _list_bump) = find_listing_config_address(
        &test_metadata.token.pubkey(),
        &ahkey,
        &token,
        &ah.treasury_mint,
        &test_metadata.mint.pubkey(),
        1,
    );

    let (pas, pas_bump) = find_program_as_signer_address();
    let (auctioneer_authority, aa_bump) = find_auctioneer_authority_seeds(ahkey);
    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority);

    let accounts = mpl_auctioneer::accounts::AuctioneerSell {
        auction_house_program: mpl_auction_house::id(),
        listing_config: listing_config_address,
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
        auctioneer_authority: auctioneer_authority,
        ah_auctioneer_pda: auctioneer_pda,
    };
    let account_metas = accounts.to_account_metas(None);

    let data = mpl_auctioneer::instruction::Sell {
        trade_state_bump: sts_bump,
        free_trade_state_bump: free_sts_bump,
        program_as_signer_bump: pas_bump,
        auctioneer_authority_bump: aa_bump,
        token_size: 1,
        start_time,
        end_time,
        reserve_price,
        min_bid_increment,
        time_ext_period,
        time_ext_delta,
        allow_high_bid_cancel,
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_auctioneer::id(),
        data,
        accounts: account_metas,
    };

    (
        (accounts, listing_config_address),
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&test_metadata.token.pubkey()),
            &[&test_metadata.token],
            context.last_blockhash,
        ),
    )
}

pub fn withdraw(
    context: &mut ProgramTestContext,
    buyer: &Keypair,
    ahkey: &Pubkey,
    ah: &AuctionHouse,
    test_metadata: &Metadata,
    sale_price: u64,
    withdraw_amount: u64,
) -> ((mpl_auctioneer::accounts::AuctioneerWithdraw,), Transaction) {
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
    let (escrow_payment_account, escrow_bump) =
        find_escrow_payment_address(&ahkey, &buyer.pubkey());
    let (auctioneer_authority, aa_bump) = find_auctioneer_authority_seeds(ahkey);
    let (auctioneer_pda, _) = find_auctioneer_pda(&ahkey, &auctioneer_authority);

    let accounts = mpl_auctioneer::accounts::AuctioneerWithdraw {
        auction_house_program: mpl_auction_house::id(),
        wallet: buyer.pubkey(),
        escrow_payment_account,
        receipt_account: buyer.pubkey(),
        treasury_mint: ah.treasury_mint,
        authority: ah.authority,
        auction_house: *ahkey,
        auction_house_fee_account: ah.auction_house_fee_account,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
        auctioneer_authority: auctioneer_authority,
        ah_auctioneer_pda: auctioneer_pda,
    };

    let accounts_metas = accounts.to_account_metas(None);

    let data = mpl_auctioneer::instruction::Withdraw {
        escrow_payment_bump: escrow_bump,
        auctioneer_authority_bump: aa_bump,
        amount: withdraw_amount,
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_auctioneer::id(),
        data,
        accounts: accounts_metas,
    };
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&buyer.pubkey()),
        &[buyer],
        context.last_blockhash,
    );

    ((accounts,), tx)
}

pub async fn existing_auction_house_test_context(
    context: &mut ProgramTestContext,
) -> StdResult<(AuctionHouse, Pubkey, Keypair), TransportError> {
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
    return Ok((auction_house_data, auction_house_address, authority));
}
