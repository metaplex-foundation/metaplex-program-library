use super::*;

use solana_program::{program::invoke, system_instruction};

use crate::{errors::CandyGuardError, utils::assert_keys_equal};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Lamports {
    pub amount: u64,
    pub destination: Pubkey,
}

impl Guard for Lamports {
    fn size() -> usize {
        8    // amount
        + 32 // destination
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
        let index = evaluation_context.account_cursor;
        // validates that we received all required accounts
        let destination = Self::get_account_info(ctx, index)?;
        evaluation_context.account_cursor += 1;
        // validates the account information
        assert_keys_equal(destination.key, &self.destination)?;

        evaluation_context
            .indices
            .insert("lamports_destination", index);

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

        let destination =
            Self::get_account_info(ctx, evaluation_context.indices["lamports_destination"])?;

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.payer.key(),
                &destination.key(),
                evaluation_context.lamports,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                destination.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}
