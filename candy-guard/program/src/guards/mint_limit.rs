use super::*;
use crate::utils::assert_keys_equal;

/// Configurations options for mint limit.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MintLimit {
    /// Unique identifier of the mint limit.
    pub id: u8,
    /// Limit of mints per individual address.
    pub limit: u32,
}

/// PDA to track the number of mints for an individual address.
#[account]
#[derive(Default)]
pub struct MintCounter {
    pub count: u32,
}

impl Guard for MintLimit {
    fn size() -> usize {
        1   // id
        + 4 // limit
    }

    fn mask() -> u64 {
        0b1u64 << 9
    }
}

impl Condition for MintLimit {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let allowance_account = Self::get_account_info(ctx, evaluation_context.account_cursor)?;
        evaluation_context
            .indices
            .insert("allowlist_index", evaluation_context.account_cursor);
        evaluation_context.account_cursor += 1;

        let user = ctx.accounts.payer.key();
        let candy_guard_key = &ctx.accounts.candy_guard.key();

        let seeds = [&[self.id], user.as_ref(), candy_guard_key.as_ref()];
        let (pda, _) = Pubkey::find_program_address(&seeds, &crate::ID);

        assert_keys_equal(allowance_account.key, &pda)?;

        let account_data = allowance_account.data.borrow();
        let mint_counter = MintCounter::try_from_slice(&account_data)?;

        if mint_counter.count >= self.limit {
            return err!(CandyGuardError::AllowedMintLimitReached);
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
        let allowance_account =
            Self::get_account_info(ctx, evaluation_context.indices["allowlist_index"])?;

        let user = ctx.accounts.payer.key();
        let candy_guard_key = &ctx.accounts.candy_guard.key();

        let seeds = [&[self.id], user.as_ref(), candy_guard_key.as_ref()];
        let (pda, _) = Pubkey::find_program_address(&seeds, &crate::ID);

        assert_keys_equal(allowance_account.key, &pda)?;

        let mut account_data = allowance_account.try_borrow_mut_data()?;
        let mut mint_counter = MintCounter::try_from_slice(&account_data)?;
        mint_counter.count += 1;
        // saves the changes back to the pda
        let data = &mut mint_counter.try_to_vec().unwrap();
        account_data[0..data.len()].copy_from_slice(data);

        Ok(())
    }
}
