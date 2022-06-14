use anchor_lang::{prelude::*, solana_program::program::invoke, AnchorDeserialize};
use spl_token::instruction::approve;

use crate::{constants::*, errors::*, utils::*, AuctionHouse, AuthorityScope, *};

/// Accounts for the [`sell` handler](auction_house/fn.sell.html).
#[derive(Accounts)]
#[instruction(
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64
)]
pub struct Sell<'info> {
    /// CHECK: Verified through CPI
    /// User wallet account.
    pub wallet: UncheckedAccount<'info>,

    /// SPL token account containing token for sale.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Verified through CPI
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Auction House authority account.
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
        has_one=auction_house_fee_account
    )]
    pub auction_house: Account<'info, AuctionHouse>,

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
    /// Seller trade state PDA account encoding the sell order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &buyer_price.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump=trade_state_bump
    )]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &0u64.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump=free_trade_state_bump
    )]
    pub free_seller_trade_state: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump=program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> From<AuctioneerSell<'info>> for Sell<'info> {
    fn from(a: AuctioneerSell<'info>) -> Sell<'info> {
        Sell {
            wallet: a.wallet,
            token_account: a.token_account,
            metadata: a.metadata,
            authority: a.authority,
            auction_house: *a.auction_house,
            auction_house_fee_account: a.auction_house_fee_account,
            seller_trade_state: a.seller_trade_state,
            free_seller_trade_state: a.free_seller_trade_state,
            token_program: a.token_program,
            system_program: a.system_program,
            program_as_signer: a.program_as_signer,
            rent: a.rent,
        }
    }
}

/// Accounts for the [`auctioneer_sell` handler](auction_house/fn.auctioneer_sell.html).
#[derive(Accounts, Clone)]
#[instruction(
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    token_size: u64
)]
pub struct AuctioneerSell<'info> {
    /// CHECK: Wallet is validated as a signer in sell_logic.
    /// User wallet account.
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,

    /// SPL token account containing token for sale.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Validated by assert_metadata_valid.
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Auction House authority account.
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Validated in ah_auctioneer_pda seeds and as a signer in sell_logic.
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
    /// Seller trade state PDA account encoding the sell order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &u64::MAX.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump=trade_state_bump
    )]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &0u64.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump=free_trade_state_bump
    )]
    pub free_seller_trade_state: UncheckedAccount<'info>,

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

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump=program_as_signer_bump)]
    pub program_as_signer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn sell<'info>(
    ctx: Context<'_, '_, '_, 'info, Sell<'info>>,
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    // If it has an auctioneer authority delegated must use auctioneer_* handler.
    if auction_house.has_auctioneer {
        return Err(AuctionHouseError::MustUseAuctioneerHandler.into());
    }

    sell_logic(
        ctx.accounts,
        ctx.program_id,
        trade_state_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        buyer_price,
        token_size,
    )
}

/// Create a sell bid by creating a `seller_trade_state` account and approving the program as the token delegate.
pub fn auctioneer_sell<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerSell<'info>>,
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    token_size: u64,
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
        AuthorityScope::Sell,
    )?;

    let mut accounts: Sell<'info> = (*ctx.accounts).clone().into();

    sell_logic(
        &mut accounts,
        ctx.program_id,
        trade_state_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        u64::MAX,
        token_size,
    )
}

/// Create a sell bid by creating a `seller_trade_state` account and approving the program as the token delegate.
fn sell_logic<'info>(
    accounts: &mut Sell<'info>,
    program_id: &Pubkey,
    trade_state_bump: u8,
    _free_trade_state_bump: u8,
    _program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let wallet = &accounts.wallet;
    let token_account = &accounts.token_account;
    let metadata = &accounts.metadata;
    let authority = &accounts.authority;
    let seller_trade_state = &accounts.seller_trade_state;
    let free_seller_trade_state = &accounts.free_seller_trade_state;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let token_program = &accounts.token_program;
    let system_program = &accounts.system_program;
    let program_as_signer = &accounts.program_as_signer;
    let rent = &accounts.rent;

    // 1. The wallet being a signer is the only condition in which an NFT can sell at a price of 0.
    //    If the user does list at 0 then auction house can change the sale price if the 'can_change_sale_price' option is true.
    // 2. If the trade is not priced at 0, the wallet holder has to be a signer since auction house cannot sign if listing over 0.
    // 3. There must be one and only one signer; there can never be zero signers or two signers.
    // 4. Auction house should be the signer for changing the price instead of user wallet for cases when seller lists at 0.
    if !wallet.to_account_info().is_signer
        && (buyer_price == 0
            || free_seller_trade_state.data_is_empty()
            || !authority.to_account_info().is_signer
            || !auction_house.can_change_sale_price)
    {
        return Err(AuctionHouseError::SaleRequiresSigner.into());
    }

    let auction_house_key = auction_house.key();

    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let (fee_payer, fee_seeds) = get_fee_payer(
        authority,
        auction_house,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;
    assert_is_ata(
        &token_account.to_account_info(),
        &wallet.key(),
        &token_account.mint,
    )?;

    assert_metadata_valid(metadata, token_account)?;

    if token_size > token_account.amount {
        return Err(AuctionHouseError::InvalidTokenAmount.into());
    }

    if wallet.is_signer {
        invoke(
            &approve(
                &token_program.key(),
                &token_account.key(),
                &program_as_signer.key(),
                &wallet.key(),
                &[],
                token_size,
            )
            .unwrap(),
            &[
                token_program.to_account_info(),
                token_account.to_account_info(),
                program_as_signer.to_account_info(),
                wallet.to_account_info(),
            ],
        )?;
    }

    let ts_info = seller_trade_state.to_account_info();
    if ts_info.data_is_empty() {
        let token_account_key = token_account.key();
        let wallet_key = wallet.key();
        let ts_seeds = [
            PREFIX.as_bytes(),
            wallet_key.as_ref(),
            auction_house_key.as_ref(),
            token_account_key.as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &buyer_price.to_le_bytes(),
            &token_size.to_le_bytes(),
            &[trade_state_bump],
        ];
        create_or_allocate_account_raw(
            *program_id,
            &ts_info,
            &rent.to_account_info(),
            system_program,
            &fee_payer,
            TRADE_STATE_SIZE,
            fee_seeds,
            &ts_seeds,
        )?;
    }

    let data = &mut ts_info.data.borrow_mut();
    data[0] = trade_state_bump;

    Ok(())
}
