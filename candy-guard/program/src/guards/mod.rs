use std::collections::BTreeMap;

pub use anchor_lang::prelude::*;

pub use crate::{errors::CandyGuardError, instructions::mint::*, state::GuardSet};

pub use self::spl_token::SplToken;
pub use allow_list::AllowList;
pub use bot_tax::BotTax;
pub use end_settings::EndSettings;
pub use gatekeeper::Gatekeeper;
pub use lamports::Lamports;
pub use live_date::LiveDate;
pub use mint_limit::MintLimit;
pub use nft_payment::NftPayment;
pub use third_party_signer::ThirdPartySigner;
pub use whitelist::Whitelist;

mod allow_list;
mod bot_tax;
mod end_settings;
mod gatekeeper;
mod lamports;
mod live_date;
mod mint_limit;
mod nft_payment;
mod spl_token;
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
        mint_args: &[u8],
        guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()>;

    /// Perform the action associated with the guard before the CPI `mint` instruction.
    ///
    /// This function only gets called when all guards have been successfuly validated.
    /// Any error generated will make the transaction to fail.
    fn pre_actions<'info>(
        &self,
        _ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
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
        _mint_args: &[u8],
        _guard_set: &GuardSet,
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
    fn load(data: &[u8], offset: usize) -> Result<Option<Self>> {
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
// TODO -> this should be a poly morphic btree map
pub struct EvaluationContext<'a> {
    /// Indicate whether the transaction was sent by the candy guard authority or not.
    pub is_authority: bool,
    /// The cursor for the remaining account list. When a guard "consumes" one of the
    /// remaining accounts, it should increment the cursor.
    pub account_cursor: usize,
    /// The cursor for the remaining bytes on the mint args. When a guard "consumes" one
    /// argument, it should increment the number of bytes read.
    pub args_cursor: usize,
    /// Convenience mapping of remaining account indices.
    pub indices: BTreeMap<&'a str, usize>,
    // > live_date
    /// Indicates whether the transaction started before the live date.
    pub is_presale: bool,
    // > lamports
    /// The amount to charge for the mint (this can be updated by guards).
    pub lamports: u64,
    // > spl_token
    /// The amount to charge for the mint (this can be updated by guards).
    pub amount: u64,
    // > whitelist
    /// Indicates whether the user is whitelisted or not.
    pub whitelist: bool,
}
