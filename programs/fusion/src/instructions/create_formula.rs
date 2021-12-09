use crate::{formula_objects::Formula, token_utils, AuthorityType, Ingredient, Item};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, Token, set_authority};

#[derive(Accounts)]
#[instruction(
    ingredients: Vec<Ingredient>,
    output_items: Vec<Item>
)]
pub struct CreateFormula<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        // The 8 is to account for anchors hash prefix
        // The 4's are for the u32 Vec::len
        space = 8 + 4 + std::mem::size_of::<Ingredient>() * ingredients.len() as usize + 4 + std::mem::size_of::<Item>() * output_items.len() as usize
    )]
    pub formula: Account<'info, Formula>,
    /// The PDA that controls the out minting and transfering
    pub output_authority: AccountInfo<'info>,

    // Misc accounts
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,
}

pub fn handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, CreateFormula<'info>>,
    ingredients: Vec<Ingredient>,
    output_items: Vec<Item>,
    bump: u8, // Run `find_program_address` offchain for canonical bump
) -> ProgramResult {
    let new_output_items = output_items.clone();

    let output_authority_seeds = &[
        &"crafting".as_bytes(),
        &ctx.accounts.formula.key().to_bytes()[..32],
        &[bump],
    ];

    // Hand over control of the mint account to PDA
    let pda_pubkey = Pubkey::create_program_address(output_authority_seeds, &ctx.program_id)?;

    let account_iter = &mut ctx.remaining_accounts.iter();

    for (_index, item) in output_items.iter().enumerate() {
        let output_mint = next_account_info(account_iter)?;

        if item.is_master_edition {
            let cur_master_edition_holder = next_account_info(account_iter)?;
            let program_master_token_acct = next_account_info(account_iter)?;

            // Validate the SPL Token program owns the accounts
            if *cur_master_edition_holder.owner != anchor_spl::token::ID {
                return Err(ProgramError::InvalidAccountData.into());
            }
            // Create the new master token account
            token_utils::create_master_token_account(
                &ctx.accounts.formula.key(),
                &item.mint,
                ctx.accounts.authority.to_account_info(),
                program_master_token_acct.clone(),
                output_mint.clone(),
                ctx.accounts.output_authority.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            )?;

            // Transfer the MasterEdition token
            let cpi_accounts = token::Transfer {
                from: cur_master_edition_holder.clone(),
                to: program_master_token_acct.clone(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info().clone(), cpi_accounts);
            token::transfer(cpi_ctx, 1)?;
        } else {
            // If the item isn't a master edition, simply transfer mint authority to the PDA
            let cpi_accounts = SetAuthority {
                account_or_mint: output_mint.clone(),
                current_authority: ctx.accounts.authority.to_account_info().clone(),
            };

            let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info().clone(), cpi_accounts);
            set_authority(cpi_ctx, AuthorityType::MintTokens.into(), Some(pda_pubkey))?;
        }
    }

    let formula = &mut ctx.accounts.formula;
    formula.ingredients = ingredients;
    formula.output_items = new_output_items;
    Ok(())
}
