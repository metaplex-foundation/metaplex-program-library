use anchor_lang::{prelude::*, solana_program::program::invoke};

use crate::{constants::*, errors::*, utils::*, *};

pub mod auctioneer_deposit;
pub mod deposit;

pub use auctioneer_deposit::*;
pub use deposit::*;

#[allow(clippy::needless_lifetimes)]
/// Deposit `amount` into the escrow payment account for your specific wallet.
fn deposit_logic<'info>(
    accounts: &mut Deposit<'info>,
    escrow_payment_bump: u8,
    amount: u64,
) -> Result<()> {
    let wallet = &accounts.wallet;
    let payment_account = &accounts.payment_account;
    let transfer_authority = &accounts.transfer_authority;
    let escrow_payment_account = &accounts.escrow_payment_account;
    let authority = &accounts.authority;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let treasury_mint = &accounts.treasury_mint;
    let system_program = &accounts.system_program;
    let token_program = &accounts.token_program;
    let rent = &accounts.rent;

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];
    let wallet_key = wallet.key();

    let escrow_signer_seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        wallet_key.as_ref(),
        &[escrow_payment_bump],
    ];

    let (fee_payer, fee_seeds) = get_fee_payer(
        authority,
        auction_house,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;

    let is_native = treasury_mint.key() == spl_token::native_mint::id();

    create_program_token_account_if_not_present(
        escrow_payment_account,
        system_program,
        &fee_payer,
        token_program,
        treasury_mint,
        &auction_house.to_account_info(),
        rent,
        &escrow_signer_seeds,
        fee_seeds,
        is_native,
    )?;

    if !is_native {
        assert_is_ata(payment_account, &wallet.key(), &treasury_mint.key())?;
        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                &payment_account.key(),
                &escrow_payment_account.key(),
                &transfer_authority.key(),
                &[],
                amount,
            )?,
            &[
                escrow_payment_account.to_account_info(),
                payment_account.to_account_info(),
                token_program.to_account_info(),
                transfer_authority.to_account_info(),
            ],
        )?;
    } else {
        assert_keys_equal(payment_account.key(), wallet.key())?;

        // Get rental exemption shortfall and then add to deposit amount.
        let rent_shortfall = verify_deposit(escrow_payment_account.to_account_info(), 0)?;
        let checked_amount = amount
            .checked_add(rent_shortfall)
            .ok_or(AuctionHouseError::NumericalOverflow)?;

        invoke(
            &system_instruction::transfer(
                &payment_account.key(),
                &escrow_payment_account.key(),
                checked_amount,
            ),
            &[
                escrow_payment_account.to_account_info(),
                payment_account.to_account_info(),
                system_program.to_account_info(),
            ],
        )?;
    }

    Ok(())
}
