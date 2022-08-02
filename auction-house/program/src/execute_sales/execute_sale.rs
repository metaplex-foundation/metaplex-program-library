use crate::{constants::*, errors::*, AuctionHouse, AuthorityScope, *};
use anchor_lang::{prelude::*, AnchorDeserialize};

/// Accounts for the [`execute_sale` handler](auction_house/fn.execute_sale.html).
#[derive(Accounts)]
#[instruction(
    escrow_payment_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64
)]
pub struct ExecuteSale<'info> {
    /// CHECK: Validated in execute_sale_logic.
    /// Buyer user wallet account.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    /// Seller user wallet account.
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    // cannot mark these as real Accounts or else we blow stack size limit
    ///Token account where the SPL token is stored.
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    /// Token mint account for the SPL token.
    pub token_mint: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    // cannot mark these as real Accounts or else we blow stack size limit
    /// Auction House treasury mint account.
    pub treasury_mint: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            buyer.key().as_ref()
        ],
        bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    /// Seller SOL or SPL account to receive payment at.
    #[account(mut)]
    pub seller_payment_receipt_account: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    /// Buyer SPL token account to receive purchased item at.
    #[account(mut)]
    pub buyer_receipt_token_account: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    /// Auction House instance authority.
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
        has_one=auction_house_treasury,
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
    /// Auction House instance treasury account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            TREASURY.as_bytes()
        ],
        bump=auction_house.treasury_bump
    )]
    pub auction_house_treasury: UncheckedAccount<'info>,

    /// CHECK: Validated in execute_sale_logic.
    /// Buyer trade state PDA account encoding the buy order.
    #[account(mut)]
    pub buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Seller trade state PDA account encoding the sell order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_mint.key().as_ref(),
            &buyer_price.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump = seller_trade_state.to_account_info().data.borrow()[0]
    )]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_mint.key().as_ref(),
            &0u64.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump
    )]
    pub free_trade_state: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub ata_program: Program<'info, AssociatedToken>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump)]
    pub program_as_signer: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> From<AuctioneerExecuteSale<'info>> for ExecuteSale<'info> {
    fn from(a: AuctioneerExecuteSale<'info>) -> ExecuteSale<'info> {
        ExecuteSale {
            buyer: a.buyer,
            seller: a.seller,
            token_account: a.token_account,
            token_mint: a.token_mint,
            metadata: a.metadata,
            treasury_mint: a.treasury_mint,
            escrow_payment_account: a.escrow_payment_account,
            seller_payment_receipt_account: a.seller_payment_receipt_account,
            buyer_receipt_token_account: a.buyer_receipt_token_account,
            authority: a.authority,
            auction_house: a.auction_house,
            auction_house_fee_account: a.auction_house_fee_account,
            auction_house_treasury: a.auction_house_treasury,
            buyer_trade_state: a.buyer_trade_state,
            seller_trade_state: a.seller_trade_state,
            free_trade_state: a.free_trade_state,
            token_program: a.token_program,
            system_program: a.system_program,
            ata_program: a.ata_program,
            program_as_signer: a.program_as_signer,
            rent: a.rent,
        }
    }
}

pub fn execute_sale<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteSale<'info>>,
    escrow_payment_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    // If it has an auctioneer authority delegated must use auctioneer_* handler.
    if auction_house.has_auctioneer && auction_house.scopes[AuthorityScope::ExecuteSale as usize] {
        return Err(AuctionHouseError::MustUseAuctioneerHandler.into());
    }

    let escrow_canonical_bump = *ctx
        .bumps
        .get("escrow_payment_account")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;
    let free_trade_state_canonical_bump = *ctx
        .bumps
        .get("free_trade_state")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;
    let program_as_signer_canonical_bump = *ctx
        .bumps
        .get("program_as_signer")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;

    if (escrow_canonical_bump != escrow_payment_bump)
        || (free_trade_state_canonical_bump != free_trade_state_bump)
        || (program_as_signer_canonical_bump != program_as_signer_bump)
    {
        return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
    }

    execute_sale_logic(
        ctx.accounts,
        ctx.remaining_accounts,
        escrow_payment_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        buyer_price,
        token_size,
        None,
        None,
    )
}

impl<'info> From<ExecutePartialSale<'info>> for ExecuteSale<'info> {
    fn from(a: ExecutePartialSale<'info>) -> ExecuteSale<'info> {
        ExecuteSale {
            buyer: a.buyer,
            seller: a.seller,
            token_account: a.token_account,
            token_mint: a.token_mint,
            metadata: a.metadata,
            treasury_mint: a.treasury_mint,
            escrow_payment_account: a.escrow_payment_account,
            seller_payment_receipt_account: a.seller_payment_receipt_account,
            buyer_receipt_token_account: a.buyer_receipt_token_account,
            authority: a.authority,
            auction_house: a.auction_house,
            auction_house_fee_account: a.auction_house_fee_account,
            auction_house_treasury: a.auction_house_treasury,
            buyer_trade_state: a.buyer_trade_state,
            seller_trade_state: a.seller_trade_state,
            free_trade_state: a.free_trade_state,
            token_program: a.token_program,
            system_program: a.system_program,
            ata_program: a.ata_program,
            program_as_signer: a.program_as_signer,
            rent: a.rent,
        }
    }
}
