
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token},
};
use solana_program::{
    program::{invoke, invoke_signed},
    system_instruction,
};
use std::convert::TryInto;

declare_id!("BNJ3tosyYaVoShvznwM5cSvCDf91WDtt3957UegPQvko");

#[program]
pub mod stealth_escorw {
    use super::*;

    pub fn init_escrow(
        ctx: Context<InitEscrow>,
        collateral: u64,
        slots: u64,
    ) -> ProgramResult {
        let escrow = &mut ctx.accounts.escrow;

        escrow.bidder = ctx.accounts.bidder.key();
        escrow.mint = ctx.accounts.mint.key();
        escrow.collateral = collateral;
        // TODO: max?
        escrow.slots = slots;
        escrow.accept_slot = None;

        Ok(())
    }

    pub fn close_escrow<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseEscrow<'info>>,
    ) -> ProgramResult {
        let escrow = &ctx.accounts.escrow;

        if let Some(accept_slot) = escrow.accept_slot {
            let unlocked_slot = accept_slot
                .checked_add(escrow.slots)
                .ok_or(ProgramError::InvalidArgument)?;
            let clock = Clock::get()?;
            if unlocked_slot < clock.slot {
                return Err(ProgramError::InvalidArgument);
            }

            // return the NFT
            let remaining_accounts = &mut ctx.remaining_accounts.iter();
            let escrow_token_account = next_account_info(remaining_accounts)?;
            let acceptor_token_account = next_account_info(remaining_accounts)?;
            let token_program = next_account_info(remaining_accounts)?;

            let bidder_key = ctx.accounts.bidder.key();
            let mint_key = ctx.accounts.mint.key();
            let escrow_signer_seeds: &[&[&[u8]]] = &[
                &[
                    b"BidEscrow".as_ref(),
                    bidder_key.as_ref(),
                    mint_key.as_ref(),
                ],
            ];

            token::transfer(
                CpiContext::new_with_signer(
                    token_program.clone(),
                    token::Transfer {
                        from: escrow_token_account.clone(),
                        to: acceptor_token_account.clone(),
                        authority: ctx.accounts.escrow.to_account_info(),
                    },
                    escrow_signer_seeds,
                ),
                1,
            )?;

            token::close_account(
                CpiContext::new_with_signer(
                    token_program.clone(),
                    token::CloseAccount {
                        account: escrow_token_account.clone(),
                        destination: ctx.accounts.bidder.to_account_info(),
                        authority: ctx.accounts.escrow.to_account_info(),
                    },
                    escrow_signer_seeds,
                ),
            )?;
        }

        Ok(())
    }

    pub fn accept_escrow(
        ctx: Context<AcceptEscrow>,
    ) -> ProgramResult {

        // init transfer before escrowing NFT
        invoke(
            &stealth::instruction::init_transfer(
                &ctx.accounts.acceptor.key(),
                &ctx.accounts.mint.key(),
                &ctx.accounts.bidder.key(),
            ),
            &[
                ctx.accounts.acceptor.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.acceptor_token_account.to_account_info(),
                ctx.accounts.stealth.to_account_info(),
                ctx.accounts.bidder.to_account_info(),
                ctx.accounts.bidder_elgamal_info.to_account_info(),
                ctx.accounts.transfer_buffer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
                ctx.accounts.stealth_program.to_account_info(),
            ],
        )?;


        // transfer NFT to escrow. can be reclaimed after unlocked if fini_transfer fails
        associated_token::create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: ctx.accounts.acceptor.to_account_info(),
                    associated_token: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
        )?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.acceptor_token_account.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.acceptor.to_account_info(),
                },
            ),
            1,
        )?;


        // set `accept_slot`
        let escrow = &mut ctx.accounts.escrow;
        let clock = Clock::get()?;

        // TODO: named errors. check that we don't overflow...
        let _unlocked_slot = clock.slot
            .checked_add(escrow.slots)
            .ok_or(ProgramError::InvalidArgument)?;
        escrow.accept_slot = Some(clock.slot);

        let escrow_collateral = escrow.collateral;
        drop(escrow);


        // post collateral
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.acceptor.key(),
                &ctx.accounts.escrow.key(),
                escrow_collateral,
            ),
            &[
                ctx.accounts.acceptor.to_account_info(),
                ctx.accounts.escrow.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn complete_escrow<'info>(
        ctx: Context<'_, '_, '_, 'info, CompleteEscrow<'info>>,
    ) -> ProgramResult {

        // finalize secret transfer
        invoke(
            &stealth::instruction::fini_transfer(
                ctx.accounts.acceptor.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.transfer_buffer.key(),
            ),
            &[
                ctx.accounts.acceptor.to_account_info(),
                ctx.accounts.stealth.to_account_info(),
                ctx.accounts.transfer_buffer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.stealth_program.to_account_info(),
            ],
        )?;


        let bidder_key = ctx.accounts.bidder.key();
        let mint_key = ctx.accounts.mint.key();
        let escrow_signer_seeds: &[&[&[u8]]] = &[
            &[
                b"BidEscrow".as_ref(),
                bidder_key.as_ref(),
                mint_key.as_ref(),
            ],
        ];

        // send NFT to bidder
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.bidder_token_account.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                escrow_signer_seeds,
            ),
            1,
        )?;

        token::close_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::CloseAccount {
                    account: ctx.accounts.escrow_token_account.to_account_info(),
                    destination: ctx.accounts.acceptor.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                escrow_signer_seeds,
            ),
        )?;


        // royalties
        let total_paid = ctx.accounts.escrow.to_account_info().lamports()  // TODO: as_ref?
            .checked_sub(ctx.accounts.escrow.collateral)
            .ok_or(ProgramError::InvalidArgument)?;
        let metadata = mpl_token_metadata::state::Metadata::from_account_info(
            ctx.accounts.metadata.as_ref())?;
        let fees = metadata.data.seller_fee_basis_points;
        let total_fee = u128::from(fees)
            .checked_mul(u128::from(total_paid))
            .ok_or(ProgramError::InvalidArgument)?
            .checked_div(10000)
            .ok_or(ProgramError::InvalidArgument)?;

        match metadata.data.creators {
            Some(creators) => {
                let remaining_accounts = &mut ctx.remaining_accounts.iter();
                for creator in creators {
                    let creator_fee = u128::from(creator.share)
                        .checked_mul(total_fee)
                        .ok_or(ProgramError::InvalidArgument)?
                        .checked_div(100)
                        .ok_or(ProgramError::InvalidArgument)?
                        .try_into().map_err(|_| ProgramError::InvalidArgument)?;

                    let current_creator_info = next_account_info(remaining_accounts)?;
                    if creator.address != *current_creator_info.key {
                        return Err(ProgramError::InvalidArgument);
                    }

                    if creator_fee == 0 {
                        continue;
                    }

                    invoke_signed(
                        &system_instruction::transfer(
                            &ctx.accounts.escrow.key(),
                            current_creator_info.key,
                            creator_fee,
                        ),
                        &[
                            ctx.accounts.escrow.to_account_info(),
                            current_creator_info.clone(),
                            ctx.accounts.system_program.to_account_info(),
                        ],
                        escrow_signer_seeds,
                    )?;
                }
            }
            None => {
                msg!("No creators found in metadata. Skipping royalties");
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitEscrow<'info> {
    pub bidder: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [
            b"BidEscrow".as_ref(),
            bidder.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        payer = bidder
    )]
    pub escrow: Account<'info, BidEscrow>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseEscrow<'info> {
    pub bidder: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        seeds = [
            b"BidEscrow".as_ref(),
            bidder.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        mut,
        close = bidder
    )]
    pub escrow: Account<'info, BidEscrow>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptEscrow<'info> {
    /*
     * bidder / BidEscrow accounts
     */
    pub bidder: AccountInfo<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        seeds = [
            b"BidEscrow".as_ref(),
            bidder.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        mut,
    )]
    pub escrow: Account<'info, BidEscrow>,

    // checked by stealth_program
    pub bidder_elgamal_info: AccountInfo<'info>,

    /*
     * seller accept accounts
     */
    pub acceptor: AccountInfo<'info>,

    // checked during spl_token::transfer to `escrow_token_account`
    pub acceptor_token_account: AccountInfo<'info>,
    // ATA created with mint `mint`
    pub escrow_token_account: AccountInfo<'info>,

    // checked by stealth_program
    pub stealth: AccountInfo<'info>,
    pub transfer_buffer: AccountInfo<'info>,

    /*
     * programs
     */
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = stealth::ID)]
    pub stealth_program: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CompleteEscrow<'info> {
    /*
     * bidder / BidEscrow accounts
     */
    pub bidder: AccountInfo<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        seeds = [
            b"BidEscrow".as_ref(),
            bidder.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        mut,
        close = acceptor,
    )]
    pub escrow: Account<'info, BidEscrow>,

    // checked during spl_token::transfer from `escrow_token_account`
    pub bidder_token_account: AccountInfo<'info>,

    /*
     * seller accept accounts
     */
    pub acceptor: AccountInfo<'info>,

    pub escrow_token_account: AccountInfo<'info>,

    // checked by stealth_program
    pub stealth: AccountInfo<'info>,
    pub transfer_buffer: AccountInfo<'info>,


    #[account(
        seeds = [
            mpl_token_metadata::state::PREFIX.as_ref(),
            mpl_token_metadata::ID.as_ref(),
            mint.key().as_ref(),
        ],
        seeds::program = mpl_token_metadata::ID,
        bump,
    )]
    pub metadata: AccountInfo<'info>,

    /*
     * programs
     */
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    #[account(address = stealth::ID)]
    pub stealth_program: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(Default)]
pub struct BidEscrow {
    // encoded in PDA. stored for lookup
    pub bidder: Pubkey,
    pub mint: Pubkey,

    // lamports seller is asked to put up when accepting
    pub collateral: u64,

    // number of slots the seller has to complete stealth::FiniTransfer
    // i.e outside of [accept_slot, accept_slot + slots), the buyer can close this bid escrow and
    // reclaim collateral
    pub slots: u64,

    pub accept_slot: Option<u64>,
}
