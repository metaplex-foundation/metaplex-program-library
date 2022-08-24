pub use anchor_lang::prelude::*;

pub use crate::errors::CandyGuardError;
pub use crate::instructions::mint::*;
pub use crate::state::CandyGuardData;

pub use bot_tax::BotTax;
pub use end_settings::EndSettings;
pub use gatekeeper::Gatekeeper;
pub use lamports::Lamports;
pub use live_date::LiveDate;
pub use spltoken::SplToken;
pub use third_party_signer::ThirdPartySigner;
pub use whitelist::Whitelist;

mod bot_tax;
mod end_settings;
mod gatekeeper;
mod lamports;
mod live_date;
mod spltoken;
mod third_party_signer;
mod whitelist;

pub trait Condition {
    /// Validate the condition of the guard. When the guard condition is
    /// not satisfied, it will return an error.
    ///
    /// This function should not perform any modification to accounts, since
    /// other guards might fail, causing the transaction to be aborted.
    ///
    /// Intermediary evaluation data can be stored in the `evaluation_context`,
    /// which will be shared with other guards and reused in the `actions` step
    /// of the process.
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()>;

    /// Perform the action associated with the guard before the CPI `mint` instruction.
    ///
    /// This function only gets called when all guards have been successfuly validated.
    /// Any error generated will make the transaction to fail.
    fn pre_actions<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        Ok(())
    }

    /// Perform the action associated with the guard after the CPI `mint` instruction.
    ///
    /// This function only gets called when all guards have been successfuly validated.
    /// Any error generated will make the transaction to fail.
    fn post_actions<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _candy_guard_data: &CandyGuardData,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        Ok(())
    }
}

pub trait Guard: Condition + AnchorSerialize + AnchorDeserialize {
    /// Return the number of bytes used by the guard configuration.
    fn size() -> usize;

    /// Return the feature mask for the guard.
    fn mask() -> u64;

    /// Returns whether the guards is enabled or not on the specified features.
    fn is_enabled(features: u64) -> bool {
        features & Self::mask() > 0
    }

    /// Enable the guard on the specified `features` value.
    fn enable(features: u64) -> u64 {
        features | Self::mask()
    }

    /// Disable the guard on the specified `features` value.
    fn disable(features: u64) -> u64 {
        features & !Self::mask()
    }

    /// Serialize the guard into the specified data array.
    fn save(&self, data: &mut [u8], offset: usize) -> Result<()> {
        let mut result = Vec::with_capacity(Self::size());
        self.serialize(&mut result)?;

        data[offset..(result.len() + offset)].copy_from_slice(&result[..]);

        Ok(())
    }

    /// Deserializes the guard from a slice of data. Only attempts the deserialization
    /// if the data slice is large enough.
    fn load(data: &mut [u8], offset: usize) -> Result<Option<Self>> {
        if offset <= data.len() {
            let mut slice = &data[offset - Self::size()..offset];
            let guard = Self::deserialize(&mut slice)?;
            Ok(Some(guard))
        } else {
            Ok(None)
        }
    }

    fn get_account_info<'c, 'info, T>(
        ctx: &Context<'_, '_, 'c, 'info, T>,
        index: usize,
    ) -> Result<&'c AccountInfo<'info>> {
        if index < ctx.remaining_accounts.len() {
            Ok(&ctx.remaining_accounts[index])
        } else {
            err!(CandyGuardError::MissingRemainingAccount)
        }
    }
}

pub struct EvaluationContext {
    /// Indicate whether the transaction was sent by the candy guard authority or not.
    pub is_authority: bool,
    /// The counter for the remaining account list. When a guard "consumes" one of the
    /// remaining accounts, it should increment the counter.
    pub remaining_account_counter: usize,
    // > live_date
    /// Indicates whether the transaction started before the live date.
    pub is_presale: bool,
    // > lamports
    /// The amount to charge for the mint (this can be updated by the whitelist guard).
    pub lamports: u64,
    // > spl_token
    /// The amount to charge for the mint (this can be updated by the whitelist guard
    /// when the `lamports_charge` is not in use).
    pub amount: u64,
    /// The index from the remaining accounts to find the token_account and
    /// transfer_authority_account
    pub spl_token_index: usize,
    // > whitelist
    /// Indicates whether the user is whitelisted or not.
    pub whitelist: bool,
    /// The index from the remaining accounts to find the whitelist_token_account,
    /// whitelist_token_mint and whitelist_burn_authority
    pub whitelist_index: usize,
    /// The index from the remaining accounts to find the gateway_token, gateway_program,
    /// and network_expire_feature
    pub gatekeeper_index: usize,
}
