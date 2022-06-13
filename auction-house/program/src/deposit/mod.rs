use anchor_lang::{prelude::*, solana_program::program::invoke, AnchorDeserialize};

use crate::{constants::*, errors::*, utils::*, AuctionHouse, AuthorityScope, *};

/// Accounts for the [`deposit` handler](auction_house/fn.deposit.html).
#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8)]
pub struct Deposit<'info> {
    /// User wallet account.
    pub wallet: Signer<'info>,

    /// CHECK: Validated in deposit_logic.
    /// User SOL or SPL account to transfer funds from.
    #[account(mut)]
    pub payment_account: UncheckedAccount<'info>,

    /// CHECK: Validated in deposit_logic.
    /// SPL token account transfer authority.
    pub transfer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account PDA.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        bump=escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Auction House instance treasury mint account.
    pub treasury_mint: Box<Account<'info, Mint>>,

    /// CHECK: Validated in deposit_logic.
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump=auction_house.bump,
        has_one=authority,
        has_one=treasury_mint,
        has_one=auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        bump=auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> From<AuctioneerDeposit<'info>> for Deposit<'info> {
    fn from(a: AuctioneerDeposit<'info>) -> Deposit<'info> {
        Deposit {
            wallet: a.wallet,
            payment_account: a.payment_account,
            transfer_authority: a.transfer_authority,
            escrow_payment_account: a.escrow_payment_account,
            treasury_mint: a.treasury_mint,
            authority: a.authority,
            auction_house: a.auction_house,
            auction_house_fee_account: a.auction_house_fee_account,
            token_program: a.token_program,
            system_program: a.system_program,
            rent: a.rent,
        }
    }
}

pub fn deposit<'info>(
    ctx: Context<'_, '_, '_, 'info, Deposit<'info>>,
    escrow_payment_bump: u8,
    amount: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    // If it has an auctioneer authority delegated must use auctioneer_* handler.
    if auction_house.has_auctioneer {
        return Err(AuctionHouseError::MustUseAuctioneerHandler.into());
    }

    deposit_logic(ctx.accounts, escrow_payment_bump, amount)
}

/// Accounts for the [`deposit` handler](auction_house/fn.deposit.html).
#[derive(Accounts, Clone)]
#[instruction(escrow_payment_bump: u8)]
pub struct AuctioneerDeposit<'info> {
    /// User wallet account.
    pub wallet: Signer<'info>,

    /// CHECK: Validated in deposit_logic.
    /// User SOL or SPL account to transfer funds from.
    #[account(mut)]
    pub payment_account: UncheckedAccount<'info>,

    /// CHECK: Validated in deposit_logic.
    /// SPL token account transfer authority.
    pub transfer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account PDA.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        bump=escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Auction House instance treasury mint account.
    pub treasury_mint: Box<Account<'info, Mint>>,

    /// CHECK: Validated in deposit_logic.
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Validated in ah_auctioneer_pda seeds and deposit_logic.
    /// The auctioneer authority - typically a PDA of the Auctioneer program running this action.
    pub auctioneer_authority: Signer<'info>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump=auction_house.bump,
        has_one=authority,
        has_one=treasury_mint,
        has_one=auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        bump=auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        bump = auction_house.auctioneer_pda_bump
    )]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn auctioneer_deposit<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerDeposit<'info>>,
    escrow_payment_bump: u8,
    amount: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;
    let auctioneer_authority = &ctx.accounts.auctioneer_authority;
    let ah_auctioneer_pda = &ctx.accounts.ah_auctioneer_pda;

    if !auction_house.has_auctioneer {
        return Err(AuctionHouseError::NoAuctioneerProgramSet.into());
    }

    assert_valid_auctioneer_and_scope(
        &auction_house.key(),
        &auctioneer_authority.key(),
        ah_auctioneer_pda,
        AuthorityScope::Deposit,
    )?;

    let mut accounts: Deposit<'info> = (*ctx.accounts).clone().into();

    deposit_logic(&mut accounts, escrow_payment_bump, amount)
}

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

        // Reach rental exemption and then add deposit amount.
        let checked_amount = rent_checked_add(escrow_payment_account.to_account_info(), 0)?
            .checked_add(amount)
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
