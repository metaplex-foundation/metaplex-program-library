use anchor_lang::{prelude::*, InstructionData};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::{AuctioneerDeposit, AuctioneerWithdraw},
    instruction::{
        AuctioneerDeposit as AuctioneerDepositData, AuctioneerWithdraw as AuctioneerWithdrawData,
    },
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse, Auctioneer,
};
use solana_program::program::invoke_signed;

use crate::{
    constants::{OFFER, REWARD_CENTER},
    metaplex_cpi::auction_house::{make_auctioneer_instruction, AuctioneerInstructionArgs},
    errors::ListingRewardsError,
    state::{
        Offer, RewardCenter,
        metaplex_anchor::TokenMetadata,
    },
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateOfferParams {
    pub new_buyer_price: u64,
    pub escrow_payment_bump: u8,
}

#[derive(Accounts, Clone)]
#[instruction(update_offer_params: UpdateOfferParams)]
pub struct UpdateOffer<'info> {
    /// The wallet who created the offer bid
    #[account(mut, address = offer.buyer)]
    pub wallet: Signer<'info>,

    /// The Offer config account used for bids
    #[account(
        mut,
        has_one = reward_center,
        has_one = metadata,
        constraint = offer.canceled_at.is_none() @ ListingRewardsError::OfferAlreadyCancelled,
        constraint = offer.purchase_ticket.is_none() @ ListingRewardsError::OfferAlreadyPurchased,
        seeds = [
            OFFER.as_bytes(),
            wallet.key().as_ref(),
            metadata.key().as_ref(),
            reward_center.key().as_ref()
        ],  
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        has_one = auction_house,
        seeds = [
            REWARD_CENTER.as_bytes(), 
            auction_house.key().as_ref()
        ], 
        bump = reward_center.bump
    )]
    pub reward_center: Box<Account<'info, RewardCenter>>,

    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.bump,
        has_one = authority,
        has_one = treasury_mint,
        has_one = auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Verified with has_one constraint on auction house account.
    /// Auction House authority account.
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Validated in public_bid_logic.
    #[account(mut)]
    pub buyer_token_account: UncheckedAccount<'info>,

    /// CHECK: Validated in public_bid_logic.
    pub transfer_authority: UncheckedAccount<'info>,

    pub treasury_mint: Box<Account<'info, Mint>>,

    /// SPL token account containing token for sale.
    #[account(
        constraint = token_account.amount == 1
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// Metaplex metadata account decorating SPL mint account.
    #[account(
        constraint = metadata.mint.eq(&token_account.mint)
    )]
    pub metadata: Box<Account<'info, TokenMetadata>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = update_offer_params.escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            reward_center.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = ah_auctioneer_pda.bump
    )]
    pub ah_auctioneer_pda: Box<Account<'info, Auctioneer>>,

    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<UpdateOffer>,
    UpdateOfferParams {
        new_buyer_price,
        escrow_payment_bump,
    }: UpdateOfferParams,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;
    let reward_center = &ctx.accounts.reward_center;
    let offer = &mut ctx.accounts.offer;

    let auction_house_key = auction_house.key();
    let old_buyer_price = offer.price;

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center.bump],
    ]];

    if new_buyer_price != old_buyer_price {
        let (ix, account_infos) = if new_buyer_price > old_buyer_price {
            let amount_to_deposit = new_buyer_price.saturating_sub(old_buyer_price);
            msg!("Depositing {} tokens", amount_to_deposit);

            let deposit_cpi_accounts = AuctioneerDeposit {
                wallet: ctx.accounts.wallet.to_account_info(),
                transfer_authority: ctx.accounts.transfer_authority.to_account_info(),
                treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
                ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
                auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
                auction_house: ctx.accounts.auction_house.to_account_info(),
                auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
                payment_account: ctx.accounts.buyer_token_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            };

            make_auctioneer_instruction(AuctioneerInstructionArgs {
                accounts: deposit_cpi_accounts,
                instruction_data: AuctioneerDepositData {
                    escrow_payment_bump,
                    amount: amount_to_deposit,
                }
                .data(),
                auctioneer_authority: ctx.accounts.reward_center.key(),
            })
        } else {
            let amount_to_withdraw = old_buyer_price.saturating_sub(new_buyer_price);
            msg!("Withdrawing {} tokens", amount_to_withdraw);

            let withdraw_cpi_accounts = AuctioneerWithdraw {
                wallet: ctx.accounts.wallet.to_account_info(),
                treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
                ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
                auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
                auction_house: ctx.accounts.auction_house.to_account_info(),
                auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
                receipt_account: ctx.accounts.buyer_token_account.to_account_info(),
                ata_program: ctx.accounts.ata_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            };

            make_auctioneer_instruction(AuctioneerInstructionArgs {
                accounts: withdraw_cpi_accounts,
                instruction_data: AuctioneerWithdrawData {
                    escrow_payment_bump,
                    amount: amount_to_withdraw,
                }
                .data(),
                auctioneer_authority: ctx.accounts.reward_center.key(),
            })
        };

        invoke_signed(&ix, &account_infos, reward_center_signer_seeds)?;

        offer.price = new_buyer_price;
    }

    Ok(())
}
