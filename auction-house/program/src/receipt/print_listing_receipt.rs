//! Create PDAs to to track the status and results of various Auction House actions.
use crate::{
    constants::*,
    id,
    instruction::Sell,
    receipt::{ListingReceipt, LISTING_RECEIPT_SIZE},
    utils::*,
};
use anchor_lang::{prelude::*, AnchorDeserialize};
use solana_program::{sysvar, sysvar::instructions::get_instruction_relative};

/// Accounts for the [`print_listing_receipt` hanlder](fn.print_listing_receipt.html).
#[derive(Accounts)]
#[instruction(receipt_bump: u8)]
pub struct PrintListingReceipt<'info> {
    /// CHECK: Receipt seeds are checked in print_listing_receipt handler.
    #[account(mut)]
    pub receipt: UncheckedAccount<'info>,

    #[account(mut)]
    pub bookkeeper: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Validated by the address constraint.
    #[account(address = sysvar::instructions::id())]
    pub instruction: UncheckedAccount<'info>,
}

/// Create a Listing Receipt account at a PDA with the seeds:
/// "listing_receipt", <SELLER_TRADE_STATE_PUBKEY>.
///
/// The previous instruction is checked to ensure that it is a "Listing" type to
/// match the receipt type being created. Passing in an empty account results in the PDA
/// being created; an existing account will be written over.
pub fn print_listing_receipt<'info>(
    ctx: Context<'_, '_, '_, 'info, PrintListingReceipt<'info>>,
    receipt_bump: u8,
) -> Result<()> {
    let receipt_account = &ctx.accounts.receipt;
    let instruction_account = &ctx.accounts.instruction;
    let bookkeeper_account = &ctx.accounts.bookkeeper;

    let rent = &ctx.accounts.rent;
    let system_program = &ctx.accounts.system_program;
    let clock = Clock::get()?;

    let prev_instruction = get_instruction_relative(-1, instruction_account)?;
    let prev_instruction_accounts = prev_instruction.accounts;

    let wallet = &prev_instruction_accounts[0];
    let auction_house = &prev_instruction_accounts[4];
    let seller_trade_state = &prev_instruction_accounts[6];
    let metadata = &prev_instruction_accounts[2];

    assert_program_listing_instruction(&prev_instruction.data[..8])?;

    let mut buffer = &prev_instruction.data[8..];
    let sell_data = Sell::deserialize(&mut buffer)?;

    assert_keys_equal(prev_instruction.program_id, id())?;

    let receipt_info = receipt_account.to_account_info();

    assert_derivation(
        &id(),
        &receipt_info,
        &[
            LISTING_RECEIPT_PREFIX.as_ref(),
            seller_trade_state.pubkey.as_ref(),
        ],
    )?;

    if receipt_info.data_is_empty() {
        let receipt_seeds = [
            LISTING_RECEIPT_PREFIX.as_bytes(),
            seller_trade_state.pubkey.as_ref(),
            &[receipt_bump],
        ];

        create_or_allocate_account_raw(
            *ctx.program_id,
            &receipt_info,
            &rent.to_account_info(),
            system_program,
            bookkeeper_account,
            LISTING_RECEIPT_SIZE,
            &[],
            &receipt_seeds,
        )?;
    }

    let receipt = ListingReceipt {
        trade_state: seller_trade_state.pubkey,
        bookkeeper: bookkeeper_account.key(),
        auction_house: auction_house.pubkey,
        seller: wallet.pubkey,
        metadata: metadata.pubkey,
        purchase_receipt: None,
        price: sell_data.buyer_price,
        token_size: sell_data.token_size,
        bump: receipt_bump,
        trade_state_bump: sell_data.trade_state_bump,
        created_at: clock.unix_timestamp,
        canceled_at: None,
    };

    receipt.try_serialize(&mut *receipt_account.try_borrow_mut_data()?)?;

    Ok(())
}
