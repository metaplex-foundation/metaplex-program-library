use super::*;
use crate::{
    errors::CandyGuardError,
    utils::{spl_token_transfer, TokenTransferParams, assert_is_token_account, assert_keys_equal, spl_token_burn, TokenBurnParams},
};
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftPayment {
    pub burn: bool,
    pub required_collection: Pubkey,
}

impl Guard for NftPayment {
    fn size() -> usize {
        1 + // Burn or transfer
        32 // required_collection
    }

    fn mask() -> u64 {
        0b1u64 << 10
    }
}

impl Condition for NftPayment {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // token
        let token_account_index = evaluation_context.account_cursor;
        let token_account_info = Self::get_account_info(ctx, token_account_index)?;
        // validates that we have the transfer_authority_info account
        let _ = Self::get_account_info(ctx, token_account_index + 1)?;
        //
        let mint_account = Self::get_account_info(ctx, token_account_index + 2)?;
        let metadata_account = Self::get_account_info(ctx, token_account_index + 3)?;
        let meta: Metadata = Metadata::from_account_info(metadata_account)?;
        assert_keys_equal(metadata_account.owner, &mpl_token_metadata::id())?;
        assert_keys_equal(&meta.mint, mint_account.key)?;
        evaluation_context.account_cursor += 4;
        let token_account = assert_is_token_account(
            token_account_info,
            &ctx.accounts.payer.key(),
            &meta.mint
        )?;
        if token_account.amount == 1 {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        match meta.collection {
            Some(c) if c.verified && c.key == self.required_collection => Ok(()),
            _ => Err(CandyGuardError::InvalidNFTCollectionPayment)
        }?;

        evaluation_context.amount = 1;
        evaluation_context
            .indices
            .insert("spl_token_index", token_account_index);

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let index = evaluation_context.indices["spl_token_index"];
        // the accounts have already been validated
        let token_account_info = Self::get_account_info(ctx, index)?;
        let transfer_authority_info = Self::get_account_info(ctx, index + 1)?;
        let mint_account = Self::get_account_info(ctx, index + 2)?;

        if self.burn {
            spl_token_burn(TokenBurnParams {
                mint: mint_account.to_account_info(),
                source: token_account_info.to_account_info(),
                authority: transfer_authority_info.to_account_info(),
                authority_signer_seeds: None,
                token_program: ctx.accounts.token_program.to_account_info(),
                amount: evaluation_context.amount,
            })?;
        } else {
            spl_token_transfer(TokenTransferParams {
                source: token_account_info.to_account_info(),
                destination: ctx.accounts.wallet.to_account_info(),
                authority: transfer_authority_info.to_account_info(),
                authority_signer_seeds: &[],
                token_program: ctx.accounts.token_program.to_account_info(),
                amount: evaluation_context.amount,
            })?;
        }


        Ok(())
    }
}
