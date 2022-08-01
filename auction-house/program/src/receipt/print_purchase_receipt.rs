//! Create PDAs to to track the status and results of various Auction House actions.
use crate::{
    constants::*,
    errors::AuctionHouseError,
    id,
    instruction::ExecuteSale,
    receipt::{BidReceipt, ListingReceipt, PurchaseReceipt, PURCHASE_RECEIPT_SIZE},
    utils::*,
};
use anchor_lang::{prelude::*, AnchorDeserialize};
use solana_program::{sysvar, sysvar::instructions::get_instruction_relative};

/// Accounts for the [`print_purchase_receipt` handler](fn.print_purchase_receipt.html).
#[derive(Accounts)]
#[instruction(receipt_bump: u8)]
pub struct PrintPurchaseReceipt<'info> {
    /// CHECK: Receipt seeds are checked in the handler.
    #[account(mut)]
    purchase_receipt: UncheckedAccount<'info>,

    /// CHECK: Receipt seeds are checked in the handler.
    #[account(mut)]
    listing_receipt: UncheckedAccount<'info>,

    /// CHECK: Receipt seeds are checked in the handler.
    #[account(mut)]
    bid_receipt: UncheckedAccount<'info>,

    #[account(mut)]
    bookkeeper: Signer<'info>,

    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,

    /// CHECK: Validated by the address constraint.
    #[account(address = sysvar::instructions::id())]
    instruction: UncheckedAccount<'info>,
}

/// Create a Purchase Receipt account at a PDA with the seeds:
/// "listing_receipt", <SELLER_TRADE_STATE_PUBKEY>, <BUYER_TRADE_STATE_PUBKEY>.
///
/// The previous instruction is checked to ensure that it is a "Purchase" type to
/// match the receipt type being created. Passing in an empty account results in the PDA
/// being created; an existing account will be written over.
pub fn print_purchase_receipt<'info>(
    ctx: Context<'_, '_, '_, 'info, PrintPurchaseReceipt<'info>>,
    purchase_receipt_bump: u8,
) -> Result<()> {
    let purchase_receipt_account = &ctx.accounts.purchase_receipt;
    let listing_receipt_account = &ctx.accounts.listing_receipt;
    let bid_receipt_account = &ctx.accounts.bid_receipt;
    let instruction_account = &ctx.accounts.instruction;
    let bookkeeper = &ctx.accounts.bookkeeper;
    let rent = &ctx.accounts.rent;
    let system_program = &ctx.accounts.system_program;
    let clock = Clock::get()?;

    let prev_instruction = get_instruction_relative(-1, instruction_account)?;
    let prev_instruction_accounts = prev_instruction.accounts;

    let mut buffer = &prev_instruction.data[8..];
    let execute_sale_data = ExecuteSale::deserialize(&mut buffer)?;

    assert_program_purchase_instruction(&prev_instruction.data[..8])?;

    assert_keys_equal(prev_instruction.program_id, id())?;

    let buyer = &prev_instruction_accounts[0];
    let seller = &prev_instruction_accounts[1];
    let metadata = &prev_instruction_accounts[4];
    let auction_house = &prev_instruction_accounts[10];
    let buyer_trade_state = &prev_instruction_accounts[13];
    let seller_trade_state = &prev_instruction_accounts[14];

    let timestamp = clock.unix_timestamp;

    let purchase_receipt_info = purchase_receipt_account.to_account_info();
    let listing_receipt_info = listing_receipt_account.to_account_info();
    let bid_receipt_info = bid_receipt_account.to_account_info();

    assert_derivation(
        &id(),
        &listing_receipt_info,
        &[
            LISTING_RECEIPT_PREFIX.as_ref(),
            seller_trade_state.pubkey.as_ref(),
        ],
    )?;
    assert_derivation(
        &id(),
        purchase_receipt_account,
        &[
            PURCHASE_RECEIPT_PREFIX.as_ref(),
            seller_trade_state.pubkey.as_ref(),
            buyer_trade_state.pubkey.as_ref(),
        ],
    )?;
    assert_derivation(
        &id(),
        &bid_receipt_info,
        &[
            BID_RECEIPT_PREFIX.as_ref(),
            buyer_trade_state.pubkey.as_ref(),
        ],
    )?;

    if listing_receipt_info.data_is_empty() || bid_receipt_info.data_is_empty() {
        return Err(AuctionHouseError::ReceiptIsEmpty.into());
    }

    if purchase_receipt_info.data_is_empty() {
        let purchase_receipt_seeds = [
            PURCHASE_RECEIPT_PREFIX.as_bytes(),
            seller_trade_state.pubkey.as_ref(),
            buyer_trade_state.pubkey.as_ref(),
            &[purchase_receipt_bump],
        ];

        create_or_allocate_account_raw(
            *ctx.program_id,
            &purchase_receipt_info,
            &rent.to_account_info(),
            system_program,
            bookkeeper,
            PURCHASE_RECEIPT_SIZE,
            &[],
            &purchase_receipt_seeds,
        )?;
    }

    let purchase = PurchaseReceipt {
        buyer: buyer.pubkey,
        seller: seller.pubkey,
        auction_house: auction_house.pubkey,
        metadata: metadata.pubkey,
        bookkeeper: bookkeeper.key(),
        bump: purchase_receipt_bump,
        price: execute_sale_data.buyer_price,
        token_size: execute_sale_data.token_size,
        created_at: timestamp,
    };

    purchase.try_serialize(&mut *purchase_receipt_account.try_borrow_mut_data()?)?;

    let mut listing_receipt_data = listing_receipt_info.try_borrow_mut_data()?;
    let mut listing_receipt_data_slice: &[u8] = &listing_receipt_data;

    let mut listing_receipt = ListingReceipt::try_deserialize(&mut listing_receipt_data_slice)?;

    listing_receipt.purchase_receipt = Some(purchase_receipt_account.key());

    listing_receipt.try_serialize(&mut *listing_receipt_data)?;

    let mut bid_receipt_data = bid_receipt_account.try_borrow_mut_data()?;
    let mut bid_receipt_slice: &[u8] = &bid_receipt_data;

    let mut bid_receipt = BidReceipt::try_deserialize(&mut bid_receipt_slice)?;

    bid_receipt.purchase_receipt = Some(purchase_receipt_account.key());

    bid_receipt.try_serialize(&mut *bid_receipt_data)?;

    Ok(())
}
