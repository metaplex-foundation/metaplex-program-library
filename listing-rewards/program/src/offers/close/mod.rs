use anchor_lang::{prelude::*, InstructionData};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::{AuctioneerCancel, AuctioneerWithdraw},
    instruction::AuctioneerCancel as AuctioneerCancelParams,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};

use crate::{
    assertions::assert_belongs_to_rewardable_collection,
    constants::{OFFER, REWARDABLE_COLLECTION, REWARD_CENTER},
    errors::ListingRewardsError,
    state::{Offer, RewardCenter, RewardableCollection},
    MetadataAccount,
};
use solana_program::{instruction::Instruction, program::invoke_signed};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CloseOfferParams {
    pub trade_state_bump: u8,
    pub escrow_payment_bump: u8,
    pub buyer_price: u64,
    pub token_size: u64,
}

#[derive(Accounts, Clone)]
#[instruction(close_offer_params: CloseOfferParams)]
pub struct CloseOffer<'info> {
    /// User wallet account.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// The Offer config account used for bids
    #[account(
        mut,
        seeds = [
            OFFER.as_bytes(),
            wallet.key().as_ref(),
            metadata.key().as_ref(),
            rewardable_collection.key().as_ref()
        ],  
        bump = offer.bump
    )]
    pub offer: Box<Account<'info, Offer>>,

    /// The collection eligable for rewards
    #[account(
        seeds = [
            REWARDABLE_COLLECTION.as_bytes(),
            reward_center.key().as_ref(),
            metadata.collection.as_ref().ok_or(ListingRewardsError::NFTMissingCollection)?.key.as_ref()
        ],
        bump = rewardable_collection.bump
    )]
    pub rewardable_collection: Box<Account<'info, RewardableCollection>>,

    pub treasury_mint: Box<Account<'info, Mint>>,

    /// SPL token account containing the token of the sale to be canceled.
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Validated in auction house program withdraw_logic.
    /// SPL token account or native SOL account to transfer funds to. If the account is a native SOL account, this is the same as the wallet address.
    #[account(mut)]
    pub receipt_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = close_offer_params.escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: Box<Account<'info, MetadataAccount>>,

    /// Token mint account of SPL token.
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: Verified with has_one constraint on auction house account.
    /// Auction House authority account.
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Verified in ah_auctioneer_pda seeds and in bid logic.
    /// The auctioneer authority - typically a PDA of the Auctioneer program running this action.
    #[account(
        seeds = [
            REWARD_CENTER.as_bytes(), 
            auction_house.key().as_ref()
        ], 
        bump = reward_center.bump
    )]
    pub reward_center: Box<Account<'info, RewardCenter>>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump =auction_house.bump,
        has_one=authority,
        has_one=auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

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

    /// CHECK: Validated in auction house program cancel_logic.
    /// Trade state PDA account representing the bid or ask to be canceled.
    #[account(mut)]
    pub trade_state: UncheckedAccount<'info>,

    /// CHECK: Validated in auction house program cancel_logic.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            reward_center.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.auctioneer_pda_bump
    )]
    pub ah_auctioneer_pda: UncheckedAccount<'info>,

    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    pub ata_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CloseOffer>,
    CloseOfferParams {
        buyer_price,
        token_size,
        escrow_payment_bump,
        ..
    }: CloseOfferParams,
) -> Result<()> {
    let metadata = &ctx.accounts.metadata;
    let reward_center = &ctx.accounts.reward_center;
    let auction_house = &ctx.accounts.auction_house;
    let rewardable_collection = &ctx.accounts.rewardable_collection;
    let wallet = &ctx.accounts.wallet;
    let auction_house_key = auction_house.key();

    assert_belongs_to_rewardable_collection(metadata, rewardable_collection)?;

    let clock = Clock::get()?;
    let offer = &mut ctx.accounts.offer;

    offer.canceled_at = Some(clock.unix_timestamp);

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center.bump],
    ]];

    let withdraw_accounts_ctx = CpiContext::new_with_signer(
        ctx.accounts.auction_house_program.to_account_info(),
        AuctioneerWithdraw {
            wallet: wallet.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
            ata_program: ctx.accounts.ata_program.to_account_info(),
            auction_house: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
            auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
            receipt_account: ctx.accounts.receipt_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
        },
        reward_center_signer_seeds,
    );

    mpl_auction_house::cpi::auctioneer_withdraw(
        withdraw_accounts_ctx,
        escrow_payment_bump,
        buyer_price,
    )?;

    // Cancel (Close Offer) instruction via invoke_signed

    let auction_house_program = ctx.accounts.auction_house_program.to_account_info();
    let close_offer_ctx_accounts = AuctioneerCancel {
        wallet: ctx.accounts.wallet.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        trade_state: ctx.accounts.trade_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let close_offer_params = AuctioneerCancelParams {
        buyer_price,
        token_size,
    };

    let close_offer_ix = Instruction {
        program_id: auction_house_program.key(),
        data: close_offer_params.data(),
        accounts: close_offer_ctx_accounts
            .to_account_metas(None)
            .into_iter()
            .zip(close_offer_ctx_accounts.to_account_infos())
            .map(|mut pair| {
                pair.0.is_signer = pair.1.is_signer;
                if pair.0.pubkey == ctx.accounts.reward_center.key() {
                    pair.0.is_signer = true;
                }
                pair.0
            })
            .collect(),
    };

    invoke_signed(
        &close_offer_ix,
        &close_offer_ctx_accounts.to_account_infos(),
        &reward_center_signer_seeds,
    )?;

    Ok(())
}
