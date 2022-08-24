use super::*;

use crate::errors::CandyGuardError;
use crate::utils::{assert_is_ata, spl_token_transfer, TokenTransferParams};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SplToken {
    pub amount: u64,
    pub token_mint: Pubkey,
}

impl Guard for SplToken {
    fn size() -> usize {
        8    // amount
        + 32 // token mint
    }

    fn mask() -> u64 {
        0b1u64 << 3
    }
}

impl Condition for SplToken {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // token
        let token_account_index = evaluation_context.remaining_account_counter;
        let token_account_info = Self::get_account_info(ctx, token_account_index)?;
        // validates that we have the transfer_authority_info account
        let _ = Self::get_account_info(ctx, token_account_index + 1)?;
        evaluation_context.remaining_account_counter += 2;

        let token_account = assert_is_ata(
            token_account_info,
            &ctx.accounts.payer.key(),
            &self.token_mint,
        )?;

        if token_account.amount < self.amount {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        evaluation_context.amount = self.amount;
        evaluation_context.spl_token_index = token_account_index;

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let index = evaluation_context.spl_token_index;
        // the accounts have already been validated
        let token_account_info = Self::get_account_info(ctx, index)?;
        let transfer_authority_info = Self::get_account_info(ctx, index + 1)?;

        spl_token_transfer(TokenTransferParams {
            source: token_account_info.to_account_info(),
            destination: ctx.accounts.wallet.to_account_info(),
            authority: transfer_authority_info.to_account_info(),
            authority_signer_seeds: &[],
            token_program: ctx.accounts.token_program.to_account_info(),
            amount: evaluation_context.amount,
        })?;

        Ok(())
    }
}
