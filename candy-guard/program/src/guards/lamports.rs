use super::*;

use solana_program::{program::invoke, system_instruction};

use crate::errors::CandyGuardError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Lamports {
    pub amount: u64,
}

impl Guard for Lamports {
    fn size() -> usize {
        8 // amount
    }

    fn mask() -> u64 {
        0b1u64 << 2
    }
}

impl Condition for Lamports {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        if ctx.accounts.payer.lamports() < self.amount {
            msg!(
                "Require {} lamports, accounts has {} lamports",
                evaluation_context.amount,
                ctx.accounts.payer.lamports(),
            );
            return err!(CandyGuardError::NotEnoughSOL);
        }

        evaluation_context.amount = self.amount;

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // sanity check: other guards might have updated the price on the
        // evaluation context. While it would be usually to decrease the
        // value, we need to check that there is enough balance on the account
        if ctx.accounts.payer.lamports() < evaluation_context.amount {
            msg!(
                "Require {} lamports, accounts has {} lamports",
                evaluation_context.amount,
                ctx.accounts.payer.lamports(),
            );
            return err!(CandyGuardError::NotEnoughSOL);
        }

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.payer.key(),
                &ctx.accounts.wallet.key(),
                evaluation_context.amount,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.wallet.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}
