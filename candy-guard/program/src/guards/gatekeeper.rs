use solana_gateway::{instruction::expire_token, state::get_expire_address_with_seed, Gateway};
use solana_program::program::invoke;

use crate::utils::assert_keys_equal;

use super::*;

// the program ID is not exported from the gateway integration crate, so we hard code it here.
const GATEWAY_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("gatem74V238djXdzWnJf94Wo1DcnuGkfijbf3AuBhfs");

/// Configurations options for the gatekeeper.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Gatekeeper {
    /// The network for the gateway token required
    pub gatekeeper_network: Pubkey,
    /// Whether or not the token should expire after minting.
    /// The gatekeeper network must support this if true.
    pub expire_on_use: bool,
}

impl Guard for Gatekeeper {
    fn size() -> usize {
        32  // gatekeeper network
        + 1 // expire on use
    }

    fn mask() -> u64 {
        0b1u64 << 6
    }
}

impl Condition for Gatekeeper {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // retrieves the (potential) gateway token
        let gateway_index = evaluation_context.account_cursor;
        let gateway_token_account = Self::get_account_info(ctx, gateway_index)?;
        // consumes the gatekeeper token account
        evaluation_context.account_cursor += 1;
        // Splits up verify and burn. We verify everything regardless of whether
        // it should be burned
        Gateway::verify_gateway_token_account_info(
            gateway_token_account,
            ctx.accounts.payer.key,
            &self.gatekeeper_network,
            None,
        )
        .map_err(|_| error!(CandyGuardError::GatewayTokenInvalid))?;
        if self.expire_on_use {
            // if expire on use is true, two more accounts are needed.
            // Ensure they are present and correct
            let gateway_program_key = Self::get_account_info(ctx, gateway_index + 1)?.key;
            assert_keys_equal(gateway_program_key, &GATEWAY_PROGRAM_ID)?;
            let expiry_key = Self::get_account_info(ctx, gateway_index + 2)?.key;
            // increment counter for next guard
            evaluation_context.account_cursor += 2;
            let expected_expiry_key = get_expire_address_with_seed(&self.gatekeeper_network).0;
            assert_keys_equal(expiry_key, &expected_expiry_key)?;
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
        if self.expire_on_use {
            let gateway_index = evaluation_context.account_cursor;
            // the accounts have already been validated
            let gateway_token_info = Self::get_account_info(ctx, gateway_index)?;
            let gateway_program_info = Self::get_account_info(ctx, gateway_index + 1)?;
            let expiry_info = Self::get_account_info(ctx, gateway_index + 2)?;
            invoke(
                &expire_token(
                    *gateway_token_info.key,
                    *ctx.accounts.payer.key,
                    self.gatekeeper_network,
                ),
                &[
                    gateway_token_info.to_account_info(),
                    ctx.accounts.payer.to_account_info(),
                    expiry_info.to_account_info(),
                    gateway_program_info.to_account_info(),
                ],
            )?;
        }
        Ok(())
    }
}
