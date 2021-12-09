use crate::{token_metadata_utils, ErrorCode, Formula};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, MintTo, Token, burn, mint_to};

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Craft<'info> {
    pub formula: Account<'info, Formula>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"crafting", &formula.to_account_info().key.to_bytes()[..32]],
        bump = bump
    )]
    pub pda_auth: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, Craft<'info>>,
    bump: u8,
) -> ProgramResult {
    let formula = &ctx.accounts.formula;

    let accounts_info_iter = &mut ctx.remaining_accounts.iter();

    for ingredient in formula.ingredients.iter() {
        let ingredient_token = next_account_info(accounts_info_iter)?;
        let ingredient_mint = next_account_info(accounts_info_iter)?;

        // these accounts are unchecked...check them
        if *ingredient_token.owner != anchor_spl::token::ID
            || *ingredient_mint.owner != anchor_spl::token::ID
        {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let token_mint = token::accessor::mint(ingredient_token)?;
        let token_amount = token::accessor::amount(ingredient_token)? as u8;
        let token_authority = token::accessor::authority(ingredient_token)?;

        // Validate token mint
        if token_mint != ingredient.mint {
            return Err(ErrorCode::InvalidMint.into());
        }

        // Validate token balance
        if token_amount < ingredient.amount {
            return Err(ErrorCode::InvalidAmount.into());
        }

        // Validate token authority is signer
        if token_authority != *ctx.accounts.authority.key {
            return Err(ErrorCode::InvalidAuthority.into());
        }

        // If burn is true, burn the tokens
        if ingredient.burn_on_craft {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                Burn {
                    mint: ingredient_mint.clone(),
                    to: ingredient_token.clone(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            );
            burn(cpi_ctx, ingredient.amount as u64)?;
        }
    }

    // Derive PDA signer
    let seeds = &[
        &"crafting".as_bytes(),
        &formula.to_account_info().key.to_bytes()[..32],
        &[bump],
    ];
    let signer = &[&seeds[..]];

    for item in formula.output_items.iter() {
        // handle case where the output Item is a master edition
        if item.is_master_edition {
            token_metadata_utils::mint_new_edition_cpi(
                accounts_info_iter,
                &ctx.accounts.authority.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
                signer,
            )?;
        } else {
            let output_item_token = next_account_info(accounts_info_iter)?;
            let output_item_mint = next_account_info(accounts_info_iter)?;

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                MintTo {
                    mint: output_item_mint.clone(),
                    authority: ctx.accounts.pda_auth.clone(),
                    to: output_item_token.clone(),
                },
                signer,
            );
            mint_to(cpi_ctx, item.amount as u64)?;
        }
    }

    Ok(())
}
