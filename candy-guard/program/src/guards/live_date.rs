use super::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiveDate {
    pub date: Option<i64>,
}

impl Guard for LiveDate {
    fn size() -> usize {
        1   // option
        + 8 // date
    }

    fn mask() -> u64 {
        0b1u64 << 3
    }
}

impl Condition for LiveDate {
    fn validate<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // the decision on whether or not the user is part of the whitelist
        // is done by the whitelist guard
        let whitelist_presale = if let Some(whitelist) = &guard_set.whitelist {
            whitelist.presale
        } else {
            false
        };

        match self.date {
            Some(date) => {
                let clock = Clock::get()?;

                if clock.unix_timestamp < date {
                    if !(whitelist_presale || evaluation_context.is_authority) {
                        return err!(CandyGuardError::MintNotLive);
                    }
                    // indicates that the evaluation was performed before the live date
                    evaluation_context.is_presale = true;
                }
            }
            None => {
                // when the live date is null, only the authority or whilelist users can mint
                if !(whitelist_presale || evaluation_context.is_authority) {
                    return err!(CandyGuardError::MintNotLive);
                }
            }
        }

        Ok(())
    }
}
