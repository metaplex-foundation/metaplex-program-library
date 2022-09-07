use super::*;

use solana_program::{program::invoke, system_instruction};

use crate::errors::CandyGuardError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Lamports {
    pub amount: u64,
    pub wallet: Pubkey,
}

impl Guard for Lamports {
    fn size() -> usize {
        8    // amount
        + 32 // wallet
    }

    fn mask() -> u64 {
        0b1u64 << 1
    }
}

impl Condition for Lamports {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let wallet_index = evaluation_context.account_cursor;
        // validates that we received all required accounts
        let _wallet = Self::get_account_info(ctx, wallet_index)?;
        evaluation_context
            .indices
            .insert("lamports_wallet", wallet_index);
        evaluation_context.account_cursor += 1;

        if ctx.accounts.payer.lamports() < self.amount {
            msg!(
                "Require {} lamports, accounts has {} lamports",
                self.amount,
                ctx.accounts.payer.lamports(),
            );
            return err!(CandyGuardError::NotEnoughSOL);
        }

        evaluation_context.lamports = self.amount;

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // sanity check: other guards might have updated the price on the
        // evaluation context. While it would be usually to decrease the
        // value, we need to check that there is enough balance on the account
        if ctx.accounts.payer.lamports() < evaluation_context.lamports {
            msg!(
                "Require {} lamports, accounts has {} lamports",
                evaluation_context.lamports,
                ctx.accounts.payer.lamports(),
            );
            return err!(CandyGuardError::NotEnoughSOL);
        }

        let wallet = Self::get_account_info(ctx, evaluation_context.indices["lamports_wallet"])?;

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.payer.key(),
                &wallet.key(),
                evaluation_context.lamports,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                wallet.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}
