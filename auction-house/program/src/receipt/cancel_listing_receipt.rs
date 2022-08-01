//! Create PDAs to to track the status and results of various Auction House actions.
use crate::{constants::*, errors::AuctionHouseError, id, receipt::ListingReceipt, utils::*};
use anchor_lang::prelude::*;
use solana_program::{sysvar, sysvar::instructions::get_instruction_relative};

/// Accounts for the [`cancel_listing_receipt` handler](fn.cancel_listing_receipt.html).
#[derive(Accounts)]
pub struct CancelListingReceipt<'info> {
    /// CHECK: Receipt seeds are checked in the handler.
    #[account(mut)]
    pub receipt: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Validated by the address constraint.
    #[account(address = sysvar::instructions::id())]
    pub instruction: UncheckedAccount<'info>,
}

/// Add a cancelation time to a listing receipt.
pub fn cancel_listing_receipt<'info>(
    ctx: Context<'_, '_, '_, 'info, CancelListingReceipt<'info>>,
) -> Result<()> {
    let receipt_account = &ctx.accounts.receipt;
    let instruction_account = &ctx.accounts.instruction;
    let clock = Clock::get()?;

    let receipt_info = receipt_account.to_account_info();

    let prev_instruction = get_instruction_relative(-1, instruction_account)?;
    let prev_instruction_accounts = prev_instruction.accounts;

    let trade_state = &prev_instruction_accounts[6];

    assert_program_cancel_instruction(&prev_instruction.data[..8])?;

    if receipt_info.data_is_empty() {
        return Err(AuctionHouseError::ReceiptIsEmpty.into());
    }

    assert_derivation(
        &id(),
        &receipt_info,
        &[LISTING_RECEIPT_PREFIX.as_ref(), trade_state.pubkey.as_ref()],
    )?;

    let mut receipt_data = receipt_info.try_borrow_mut_data()?;
    let mut receipt_data_slice: &[u8] = &receipt_data;

    let mut receipt = ListingReceipt::try_deserialize(&mut receipt_data_slice)?;

    receipt.canceled_at = Some(clock.unix_timestamp);

    receipt.try_serialize(&mut *receipt_data)?;

    Ok(())
}
