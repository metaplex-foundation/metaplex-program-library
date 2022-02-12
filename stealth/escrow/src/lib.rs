
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

#[cfg(not(target_arch = "bpf"))]
use {
    anchor_lang::{
        InstructionData,
        ToAccountMetas,
    },
    solana_sdk::{
        instruction::Instruction,
        system_program,
    },
};

declare_id!("BNJ3tosyYaVoShvznwM5cSvCDf91WDtt3957UegPQvko");

#[program]
pub mod stealth_escrow {
    use super::*;

    pub fn init_escrow(
        ctx: Context<InitEscrow>,
        collateral: u64,
        slots: u64,
    ) -> ProgramResult {
        let escrow = &mut ctx.accounts.escrow;

        escrow.bidder = ctx.accounts.bidder.key();
        escrow.mint = ctx.accounts.mint.key();
        // TODO: would be nice to dedup with anchor find
        escrow.bump_seed = Pubkey::find_program_address(
            &[
                b"BidEscrow".as_ref(),
                escrow.bidder.as_ref(),
                escrow.mint.as_ref(),
            ],
            &ID,
        ).1;
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
            if clock.slot < unlocked_slot {
                return Err(ProgramError::InvalidArgument);
            }

            // return the NFT
            let remaining_accounts = &mut ctx.remaining_accounts.iter();
            let escrow_token_account_info = next_account_info(remaining_accounts)?;
            let acceptor_token_account_info = next_account_info(remaining_accounts)?;
            let token_program = next_account_info(remaining_accounts)?;

            let escrow_signer_seeds: &[&[&[u8]]] = &[
                &[
                    b"BidEscrow".as_ref(),
                    escrow.bidder.as_ref(),
                    escrow.mint.as_ref(),
                    &[escrow.bump_seed],
                ],
            ];

            use solana_program::program_pack::Pack;
            let accept_token_account = spl_token::state::Account::unpack_from_slice(
                &acceptor_token_account_info.try_borrow_data()?)?;
            if accept_token_account.owner != escrow.acceptor {
                return Err(ProgramError::InvalidArgument);
            }

            token::transfer(
                CpiContext::new_with_signer(
                    token_program.clone(),
                    token::Transfer {
                        from: escrow_token_account_info.clone(),
                        to: acceptor_token_account_info.clone(),
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
                        account: escrow_token_account_info.clone(),
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
            // TODO: skip PDA lookups
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
        escrow.acceptor = ctx.accounts.acceptor.key();

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
            // TODO: skip PDA lookups
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


        let escrow = &ctx.accounts.escrow;
        let escrow_signer_seeds: &[&[&[u8]]] = &[
            &[
                b"BidEscrow".as_ref(),
                escrow.bidder.as_ref(),
                escrow.mint.as_ref(),
                &[escrow.bump_seed],
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
        payer = bidder,
        space = 8       // discriminant
            + 32        // bidder pubkey
            + 32        // mint pubkey
            + 1         // bump_seed u8
            + 8         // collateral u64
            + 8         // slots u64
            + 1 + 8     // accept_slot option<u64>
            + 32        // acceptor pubkey
            ,
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
    #[account(mut)]
    pub acceptor: Signer<'info>,

    // checked during spl_token::transfer to `escrow_token_account`
    #[account(mut)]
    pub acceptor_token_account: AccountInfo<'info>,
    // ATA created with mint `mint`
    #[account(mut)]
    pub escrow_token_account: AccountInfo<'info>,

    // checked by stealth_program
    pub stealth: AccountInfo<'info>,
    #[account(mut)]
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
    #[account(mut)]
    pub bidder_token_account: AccountInfo<'info>,

    /*
     * seller accept accounts
     */
    pub acceptor: Signer<'info>,

    #[account(mut)]
    pub escrow_token_account: AccountInfo<'info>,

    // checked by stealth_program
    #[account(mut)]
    pub stealth: AccountInfo<'info>,
    #[account(mut)]
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
    pub bump_seed: u8,

    // lamports seller is asked to put up when accepting
    pub collateral: u64,

    // number of slots the seller has to complete stealth::FiniTransfer
    // i.e outside of [accept_slot, accept_slot + slots), the buyer can close this bid escrow and
    // reclaim collateral
    pub slots: u64,

    pub accept_slot: Option<u64>,
    pub acceptor: Pubkey,
}

#[cfg(not(target_arch = "bpf"))]
pub fn accept_escrow(
    bidder: Pubkey,
    mint: Pubkey,
    escrow: Pubkey, // could be calculated from bidder + mint
    acceptor: Pubkey,
) -> Instruction {
    Instruction {
        program_id: id(),
        data: instruction::AcceptEscrow {}.data(),
        accounts: accounts::AcceptEscrow {
            bidder,
            mint,
            escrow,
            bidder_elgamal_info:
                stealth::instruction::get_elgamal_pubkey_address(
                    &bidder, &mint).0,
            acceptor,
            acceptor_token_account:
                spl_associated_token_account::get_associated_token_address(
                    &acceptor,
                    &mint,
                ),
            escrow_token_account:
                spl_associated_token_account::get_associated_token_address(
                    &escrow,
                    &mint,
                ),
            stealth:
                stealth::instruction::get_stealth_address(&mint).0,
            transfer_buffer:
                stealth::instruction::get_transfer_buffer_address(
                    &bidder, &mint).0,
            system_program: system_program::id(),
            token_program: spl_token::id(),
            associated_token_program: spl_associated_token_account::id(),
            stealth_program: stealth::id(),
            rent: solana_sdk::sysvar::rent::id(),
        }.to_account_metas(None),
    }
}
