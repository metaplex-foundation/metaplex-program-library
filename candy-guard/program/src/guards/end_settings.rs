use crate::utils::cmp_pubkeys;

use super::*;

/// Configurations options for end settings.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EndSettings {
    pub end_setting_type: EndSettingType,
    pub number: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum EndSettingType {
    Date,
    Amount,
}
impl Guard for EndSettings {
    fn size() -> usize {
        1  // end_setting_type
        + 8 // number
    }

    fn mask() -> u64 {
        0b1u64 << 7
    }
}

impl Condition for EndSettings {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let candy_machine = &ctx.accounts.candy_machine;
        if !cmp_pubkeys(&ctx.accounts.payer.key(), &candy_machine.authority) {
            match self.end_setting_type {
                EndSettingType::Date => {
                    if Clock::get().unwrap().unix_timestamp > self.number as i64 {
                        return err!(CandyGuardError::AfterEndSettingsDate);
                    }
                }
                EndSettingType::Amount => {
                    if candy_machine.items_redeemed >= self.number {
                        return err!(CandyGuardError::AfterEndSettingsMintAmount);
                    }
                }
            }
        }
        Ok(())
    }
}
