use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::{constants::*, errors::*, utils::*, AuctionHouse, AuthorityScope, *};

/// Accounts for the [`withdraw` handler](auction_house/fn.withdraw.html).
#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8)]
pub struct InstantWithdraw<'info> {
    /// User wallet account.
    pub wallet: UncheckedAccount<'info>,

    /// SPL token account or native SOL account to transfer funds to. If the account is a native SOL account, this is the same as the wallet address.
    #[account(mut)]
    pub receipt_account: UncheckedAccount<'info>,

    /// Buyer escrow payment account PDA.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Auction House instance treasury mint account.
    pub treasury_mint: Account<'info, Mint>,

    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=auction_house_fee_account)]
    pub auction_house: Account<'info, AuctionHouse>,

    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> From<WithdrawWithAuctioneer<'info>> for InstantWithdraw<'info> {
    fn from(a: WithdrawWithAuctioneer<'info>) -> InstantWithdraw<'info> {
        InstantWithdraw {
            wallet: a.wallet,
            receipt_account: a.receipt_account,
            escrow_payment_account: a.escrow_payment_account,
            treasury_mint: a.treasury_mint,
            authority: a.authority,
            auction_house: a.auction_house,
            auction_house_fee_account: a.auction_house_fee_account,
            token_program: a.token_program,
            system_program: a.system_program,
            ata_program: a.ata_program,
            rent: a.rent,
        }
    }
}

/// Accounts for the [`withdraw` handler](auction_house/fn.withdraw.html).
#[derive(Accounts, Clone)]
#[instruction(escrow_payment_bump: u8)]
pub struct WithdrawWithAuctioneer<'info> {
    /// User wallet account.
    pub wallet: UncheckedAccount<'info>,

    /// SPL token account or native SOL account to transfer funds to. If the account is a native SOL account, this is the same as the wallet address.
    #[account(mut)]
    pub receipt_account: UncheckedAccount<'info>,

    /// Buyer escrow payment account PDA.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Auction House instance treasury mint account.
    pub treasury_mint: Account<'info, Mint>,

    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=auction_house_fee_account)]
    pub auction_house: Account<'info, AuctionHouse>,

    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// The auctioneer program PDA running this auction.
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], bump = auction_house.bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn instant_withdraw<'info>(
    ctx: Context<'_, '_, '_, 'info, InstantWithdraw<'info>>,
    escrow_payment_bump: u8,
    amount: u64,
) -> ProgramResult {
    let auction_house = &ctx.accounts.auction_house;

    // If it has an auctioneer authority delegated must use *_with_auctioneer handler.
    if auction_house.has_auctioneer {
        return Err(ErrorCode::MustUseAuctioneerHandler.into());
    }

    withdraw(ctx.accounts, escrow_payment_bump, amount)
}

pub fn withdraw_with_auctioneer<'info>(
    ctx: Context<'_, '_, '_, 'info, WithdrawWithAuctioneer<'info>>,
    escrow_payment_bump: u8,
    amount: u64,
) -> ProgramResult {
    let auction_house = &ctx.accounts.auction_house;
    let auctioneer_authority = &ctx.accounts.auctioneer_authority;
    let ah_auctioneer_pda = &ctx.accounts.ah_auctioneer_pda;

    if !auction_house.has_auctioneer {
        return Err(ErrorCode::NoAuctioneerProgramSet.into());
    }

    assert_valid_auctioneer_and_scope(
        &auction_house.key(),
        &auctioneer_authority.key(),
        ah_auctioneer_pda,
        AuthorityScope::Sell,
    )?;

    let mut accounts: InstantWithdraw<'info> = (*ctx.accounts).clone().into();

    withdraw(&mut accounts, escrow_payment_bump, amount)
}

fn withdraw<'info>(
    accounts: &mut InstantWithdraw<'info>,
    escrow_payment_bump: u8,
    amount: u64,
) -> ProgramResult {
    let wallet = &accounts.wallet;
    let receipt_account = &accounts.receipt_account;
    let escrow_payment_account = &accounts.escrow_payment_account;
    let authority = &accounts.authority;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let treasury_mint = &accounts.treasury_mint;
    let system_program = &accounts.system_program;
    let token_program = &accounts.token_program;
    let ata_program = &accounts.ata_program;
    let rent = &accounts.rent;

    // If it has an auctioneer authority delegated must use *_with_auctioneer handler.
    if auction_house.has_auctioneer {
        return Err(ErrorCode::MustUseAuctioneerHandler.into());
    }

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let ah_seeds = [
        PREFIX.as_bytes(),
        auction_house.creator.as_ref(),
        auction_house.treasury_mint.as_ref(),
        &[auction_house.bump],
    ];

    let auction_house_key = auction_house.key();
    let wallet_key = wallet.key();

    if !wallet.to_account_info().is_signer && !authority.to_account_info().is_signer {
        return Err(ErrorCode::NoValidSignerPresent.into());
    }

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

    if !is_native {
        if receipt_account.data_is_empty() {
            make_ata(
                receipt_account.to_account_info(),
                wallet.to_account_info(),
                treasury_mint.to_account_info(),
                fee_payer.to_account_info(),
                ata_program.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
                rent.to_account_info(),
                &fee_seeds,
            )?;
        }

        let rec_acct = assert_is_ata(
            &receipt_account.to_account_info(),
            &wallet.key(),
            &treasury_mint.key(),
        )?;

        // make sure you cant get rugged
        if rec_acct.delegate.is_some() {
            return Err(ErrorCode::BuyerATACannotHaveDelegate.into());
        }

        assert_is_ata(receipt_account, &wallet.key(), &treasury_mint.key())?;
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                &escrow_payment_account.key(),
                &receipt_account.key(),
                &auction_house.key(),
                &[],
                amount,
            )?,
            &[
                escrow_payment_account.to_account_info(),
                receipt_account.to_account_info(),
                token_program.to_account_info(),
                auction_house.to_account_info(),
            ],
            &[&ah_seeds],
        )?;
    } else {
        assert_keys_equal(receipt_account.key(), wallet.key())?;
        invoke_signed(
            &system_instruction::transfer(
                &escrow_payment_account.key(),
                &receipt_account.key(),
                amount,
            ),
            &[
                escrow_payment_account.to_account_info(),
                receipt_account.to_account_info(),
                system_program.to_account_info(),
            ],
            &[&escrow_signer_seeds],
        )?;
    }

    Ok(())
}
