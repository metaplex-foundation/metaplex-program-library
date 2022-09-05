use super::*;

use crate::utils::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Whitelist {
    pub mint: Pubkey,
    pub presale: bool,
    pub discount_price: Option<u64>,
    pub mode: WhitelistTokenMode,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Clone, Debug)]
pub enum WhitelistTokenMode {
    BurnEveryTime,
    NeverBurn,
}

impl Guard for Whitelist {
    fn size() -> usize {
        32      // mint
        + 1     // presale
        + 1 + 8 // option + discount_price
        + 1 // mode
    }

    fn mask() -> u64 {
        0b1u64 << 5
    }
}

impl Condition for Whitelist {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // retrieves the (potential) whitelist token account
        let whitelist_index = evaluation_context.account_cursor;
        let whitelist_token_account = Self::get_account_info(ctx, whitelist_index)?;
        // consumes the whitelist token account
        evaluation_context.account_cursor += 1;

        // if the user has not actually made this account, this explodes and we just
        // check normal dates. if they have, we check amount: if it's > 0 we let them
        // use the logic; if 0, check normal dates.
        match assert_is_ata(
            whitelist_token_account,
            &ctx.accounts.payer.key(),
            &self.mint,
        ) {
            Ok(wta) => {
                // if the amount is greater than 0, the user is allowed to mint
                // we only need to check whether there is a discount price or not
                // and burn the token if needed
                if wta.amount > 0 {
                    if let Some(price) = self.discount_price {
                        // user will pay the discount price (either lamports or spl-token
                        // amount)
                        if guard_set.lamports.is_some() {
                            evaluation_context.lamports = price;
                        } else if guard_set.spl_token.is_some() {
                            evaluation_context.amount = price;
                        }
                    }
                    // should we burn the token?
                    if self.mode == WhitelistTokenMode::BurnEveryTime {
                        let whitelist_token_mint =
                            Self::get_account_info(ctx, whitelist_index + 1)?;
                        // validates that we have the whitelist_burn_authority account
                        let _ = Self::get_account_info(ctx, whitelist_index + 2)?;
                        // consumes the remaning account
                        evaluation_context.account_cursor += 2;
                        // is the mint account the one expected?
                        assert_keys_equal(&whitelist_token_mint.key(), &self.mint)?;

                        evaluation_context
                            .indices
                            .insert("whitelist_index", whitelist_index);
                    }
                    // user is whitelisted
                    evaluation_context.whitelist = true;
                } else {
                    // if the user does not have balance, we need to check whether the mint
                    // is in presale period or limited to only whitelist users
                    if wta.amount == 0
                        && ((self.discount_price.is_none() && !self.presale)
                            || evaluation_context.is_presale)
                        && !evaluation_context.is_authority
                    {
                        // (only whitelist users can mint) a non-presale whitelist with no discount
                        // price is a forced whitelist or we are in presale period
                        return err!(CandyGuardError::NoWhitelistToken);
                    }
                    // no presale period, consumes the remaning accounts if needed
                    if self.mode == WhitelistTokenMode::BurnEveryTime {
                        evaluation_context.account_cursor += 2;
                    }
                }
            }
            Err(_) => {
                // no token, we need to check whether the mint is in presale period or limited
                // to only whitelist users
                if ((self.discount_price.is_none() && !self.presale)
                    || evaluation_context.is_presale)
                    && !evaluation_context.is_authority
                {
                    // (only whitelist users can mint) a non-presale whitelist with no discount
                    // price is a forced whitelist or if we are in presale period
                    return err!(CandyGuardError::NoWhitelistToken);
                }
                // no presale period, consumes the remaning accounts if needed
                if self.mode == WhitelistTokenMode::BurnEveryTime {
                    evaluation_context.account_cursor += 2;
                }
            }
        }

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        if evaluation_context.whitelist && self.mode == WhitelistTokenMode::BurnEveryTime {
            let index = evaluation_context.indices["whitelist_index"];
            // the accounts have already being validated
            let whitelist_token_account = Self::get_account_info(ctx, index)?;
            let whitelist_token_mint = Self::get_account_info(ctx, index + 1)?;
            let whitelist_burn_authority = Self::get_account_info(ctx, index + 2)?;

            spl_token_burn(TokenBurnParams {
                mint: whitelist_token_mint.to_account_info(),
                source: whitelist_token_account.to_account_info(),
                amount: 1,
                authority: whitelist_burn_authority.to_account_info(),
                authority_signer_seeds: None,
                token_program: ctx.accounts.token_program.to_account_info(),
            })?;
        }
        Ok(())
    }
}
