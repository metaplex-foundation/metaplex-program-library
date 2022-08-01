//! Create PDAs to to track the status and results of various Auction House actions.
use crate::{
    constants::*,
    id,
    instruction::Buy,
    receipt::{BidReceipt, BID_RECEIPT_SIZE},
    utils::*,
};
use anchor_lang::{prelude::*, AnchorDeserialize};
use solana_program::{sysvar, sysvar::instructions::get_instruction_relative};

/// Accounts for the [`print_bid_receipt` handler](fn.print_bid_receipt.html).
#[derive(Accounts)]
#[instruction(receipt_bump: u8)]
pub struct PrintBidReceipt<'info> {
    /// CHECK: Receipt seeds are checked in the handler.
    #[account(mut)]
    receipt: UncheckedAccount<'info>,

    #[account(mut)]
    bookkeeper: Signer<'info>,

    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,

    /// CHECK: Validated by the address constraint.
    #[account(address = sysvar::instructions::id())]
    instruction: UncheckedAccount<'info>,
}

/// Create a Bid Receipt account at a PDA with the seeds:
/// "bid_receipt", <BUYER_TRADE_STATE_PUBKEY>.
///
/// The previous instruction is checked to ensure that it is a "Bid" type to
/// match the receipt type being created. Passing in an empty account results in the PDA
/// being created; an existing account will be written over.
pub fn print_bid_receipt<'info>(
    ctx: Context<'_, '_, '_, 'info, PrintBidReceipt<'info>>,
    receipt_bump: u8,
) -> Result<()> {
    let receipt_account = &ctx.accounts.receipt;
    let instruction_account = &ctx.accounts.instruction;
    let bookkeeper_account = &ctx.accounts.bookkeeper;

    let rent = &ctx.accounts.rent;
    let system_program = &ctx.accounts.system_program;
    let clock = Clock::get()?;

    let receipt_info = receipt_account.to_account_info();

    let prev_instruction = get_instruction_relative(-1, instruction_account)?;
    let prev_instruction_accounts = prev_instruction.accounts;

    let wallet = &prev_instruction_accounts[0];
    let token_account = &prev_instruction_accounts[4];
    let auction_house = &prev_instruction_accounts[8];
    let buyer_trade_state = &prev_instruction_accounts[10];
    let metadata = &prev_instruction_accounts[5];

    let mut buffer = &prev_instruction.data[8..];
    let buy_data = Buy::deserialize(&mut buffer)?;

    let bid_type = assert_program_bid_instruction(&prev_instruction.data[..8])?;

    let token_account = match bid_type {
        BidType::PrivateSale => Some(token_account.pubkey),
        BidType::AuctioneerPrivateSale => Some(token_account.pubkey),
        BidType::PublicSale => None,
        BidType::AuctioneerPublicSale => None,
    };

    assert_derivation(
        &id(),
        &receipt_info,
        &[
            BID_RECEIPT_PREFIX.as_ref(),
            buyer_trade_state.pubkey.as_ref(),
        ],
    )?;

    assert_keys_equal(prev_instruction.program_id, id())?;

    let receipt_info = receipt_account.to_account_info();

    if receipt_info.data_is_empty() {
        let receipt_seeds = [
            BID_RECEIPT_PREFIX.as_bytes(),
            buyer_trade_state.pubkey.as_ref(),
            &[receipt_bump],
        ];

        create_or_allocate_account_raw(
            *ctx.program_id,
            &receipt_info,
            &rent.to_account_info(),
            system_program,
            bookkeeper_account,
            BID_RECEIPT_SIZE,
            &[],
            &receipt_seeds,
        )?;
    }

    let receipt = BidReceipt {
        token_account,
        trade_state: buyer_trade_state.pubkey,
        bookkeeper: bookkeeper_account.key(),
        auction_house: auction_house.pubkey,
        buyer: wallet.pubkey,
        metadata: metadata.pubkey,
        purchase_receipt: None,
        price: buy_data.buyer_price,
        token_size: buy_data.token_size,
        bump: receipt_bump,
        trade_state_bump: buy_data.trade_state_bump,
        created_at: clock.unix_timestamp,
        canceled_at: None,
    };

    receipt.try_serialize(&mut *receipt_account.try_borrow_mut_data()?)?;

    Ok(())
}
