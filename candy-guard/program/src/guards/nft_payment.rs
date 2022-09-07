use super::*;
use crate::{
    errors::CandyGuardError,
    utils::{
        assert_is_token_account, assert_keys_equal, spl_token_burn, spl_token_transfer,
        TokenBurnParams, TokenTransferParams,
    },
};
use mpl_token_metadata::{
    instruction::burn_nft,
    state::{Metadata, TokenMetadataAccount},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftPayment {
    pub burn: bool,
    pub required_collection: Pubkey,
}

impl Guard for NftPayment {
    fn size() -> usize {
        1    // burn or transfer
        + 32 // required_collection
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
        let token_account_index = evaluation_context.account_cursor;
        // validates that we received all required accounts
        let token_account_info = Self::get_account_info(ctx, token_account_index)?;
        let token_account_metadata = Self::get_account_info(ctx, token_account_index + 1)?;
        // transfer_authority_info account
        let _ = Self::get_account_info(ctx, token_account_index + 2)?;
        evaluation_context.account_cursor += 3;

        // when burn is enabled, we need the mint_account
        let mint_account = if self.burn {
            Some(Self::get_account_info(ctx, token_account_index + 3)?)
        } else {
            None
        };

        let metadata: Metadata = Metadata::from_account_info(token_account_metadata)?;
        // validates the account information
        assert_keys_equal(token_account_metadata.owner, &mpl_token_metadata::id())?;

        if let Some(mint_account) = mint_account {
            evaluation_context.account_cursor += 1;
            assert_keys_equal(&metadata.mint, mint_account.key)?;
        }

        let token_account = assert_is_token_account(
            token_account_info,
            &ctx.accounts.payer.key(),
            &metadata.mint,
        )?;

        if token_account.amount == 1 {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        match metadata.collection {
            Some(c) if c.verified && c.key == self.required_collection => Ok(()),
            _ => Err(CandyGuardError::InvalidNFTCollectionPayment),
        }?;

        evaluation_context.amount = 1;
        evaluation_context
            .indices
            .insert("nft_payment_index", token_account_index);

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let index = evaluation_context.indices["nft_payment_index"];
        // the accounts have already been validated
        let token_account_info = Self::get_account_info(ctx, index)?;
        let transfer_authority_info = Self::get_account_info(ctx, index + 2)?;

        if self.burn {
            let mint_account = Self::get_account_info(ctx, index + 3)?;
            spl_token_burn(TokenBurnParams {
                mint: mint_account.to_account_info(),
                source: token_account_info.to_account_info(),
                authority: transfer_authority_info.to_account_info(),
                authority_signer_seeds: None,
                token_program: ctx.accounts.token_program.to_account_info(),
                amount: evaluation_context.amount,
            })?;
            /*
            let burn_nft_infos = vec![
                token_account_metadata,
                &ctx.accounts.payer.to_account_info(),
                mint_account,
                token_account_info,
            collection_mint.to_account_info(),
            collection_metadata.to_account_info(),
            collection_master_edition.to_account_info(),
            collection_authority_record.to_account_info(),
            ];
            */
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
