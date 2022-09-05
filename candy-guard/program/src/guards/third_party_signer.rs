use super::*;
use crate::utils::cmp_pubkeys;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ThirdPartySigner {
    pub signer_key: Pubkey,
}

impl Guard for ThirdPartySigner {
    fn size() -> usize {
        32 // Pubkey
    }

    fn mask() -> u64 {
        0b1u64 << 4
    }
}

impl Condition for ThirdPartySigner {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let signer_index = evaluation_context.account_cursor;
        evaluation_context.account_cursor += 1;
        let signer_account = Self::get_account_info(ctx, signer_index)?;

        if !(cmp_pubkeys(signer_account.key, &self.signer_key) && signer_account.is_signer) {
            return err!(CandyGuardError::MissingRequiredSignature);
        }

        Ok(())
    }
}
