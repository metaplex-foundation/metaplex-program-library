use anchor_lang::{prelude::*, AnchorDeserialize, InstructionData};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};

use mpl_auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::AuctioneerWithdraw as AHWithdraw,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};

use solana_program::program::invoke_signed;

/// Accounts for the [`withdraw_with_auctioneer` handler](auction_house/fn.withdraw_with_auctioneer.html).
#[derive(Accounts, Clone)]
#[instruction(escrow_payment_bump: u8, auctioneer_authority_bump: u8)]
pub struct AuctioneerWithdraw<'info> {
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,

    /// CHECK: Verified through CPI
    /// User wallet account.
    pub wallet: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// SPL token account or native SOL account to transfer funds to. If the account is a native SOL account, this is the same as the wallet address.
    #[account(mut)]
    pub receipt_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account PDA.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), wallet.key().as_ref()], seeds::program=auction_house_program, bump=escrow_payment_bump)]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Auction House instance treasury mint account.
    pub treasury_mint: Box<Account<'info, Mint>>,

    /// CHECK: Verified through CPI
    /// Auction House instance authority account.
    pub authority: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], seeds::program=auction_house_program, bump=auction_house.bump, has_one=treasury_mint, has_one=auction_house_fee_account)]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], seeds::program=auction_house_program, bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// The auctioneer program PDA running this auction.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref()], bump=auctioneer_authority_bump)]
    pub auctioneer_authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(seeds = [AUCTIONEER.as_bytes(), auction_house.key().as_ref(), auctioneer_authority.key().as_ref()], seeds::program=auction_house_program, bump = auction_house.auctioneer_pda_bump)]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Withdraw but with an auctioneer.
pub fn auctioneer_withdraw<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerWithdraw<'info>>,
    escrow_payment_bump: u8,
    auctioneer_authority_bump: u8,
    amount: u64,
) -> Result<()> {
    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHWithdraw {
        wallet: ctx.accounts.wallet.to_account_info(),
        receipt_account: ctx.accounts.receipt_account.to_account_info(),
        escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
        treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        ata_program: ctx.accounts.ata_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let withdraw_data = mpl_auction_house::instruction::AuctioneerWithdraw {
        escrow_payment_bump,
        amount,
    };

    let ix = solana_program::instruction::Instruction {
        program_id: cpi_program.key(),
        accounts: cpi_accounts
            .to_account_metas(None)
            .into_iter()
            .zip(cpi_accounts.to_account_infos())
            .map(|mut pair| {
                pair.0.is_signer = pair.1.is_signer;
                if pair.0.pubkey == ctx.accounts.auctioneer_authority.key() {
                    pair.0.is_signer = true;
                }
                pair.0
            })
            .collect(),
        data: withdraw_data.data(),
    };

    let auction_house = &ctx.accounts.auction_house;
    let ah_key = auction_house.key();
    let auctioneer_authority = &ctx.accounts.auctioneer_authority;
    let _aa_key = auctioneer_authority.key();

    let auctioneer_seeds = [
        AUCTIONEER.as_bytes(),
        ah_key.as_ref(),
        &[auctioneer_authority_bump],
    ];

    invoke_signed(&ix, &cpi_accounts.to_account_infos(), &[&auctioneer_seeds])?;

    Ok(())
}
