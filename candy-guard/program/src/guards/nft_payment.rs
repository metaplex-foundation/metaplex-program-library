use super::*;
use crate::{
    errors::CandyGuardError,
    utils::{assert_is_token_account, assert_keys_equal, spl_token_transfer, TokenTransferParams},
};
use mpl_token_metadata::{
    instruction::burn_nft,
    state::{Metadata, TokenMetadataAccount},
};
use solana_program::program::invoke;

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

        // when burn is enabled, we need mint_account and mint_master_edition
        let (mint_account, _mint_master_edition, _mint_collection_metadata) = if self.burn {
            (
                Some(Self::get_account_info(ctx, token_account_index + 3)?),
                Some(Self::get_account_info(ctx, token_account_index + 4)?),
                Some(Self::get_account_info(ctx, token_account_index + 5)?),
            )
        } else {
            (None, None, None)
        };

        let metadata: Metadata = Metadata::from_account_info(token_account_metadata)?;
        // validates the account information
        assert_keys_equal(token_account_metadata.owner, &mpl_token_metadata::id())?;

        if self.burn {
            evaluation_context.account_cursor += 3;
            assert_keys_equal(&metadata.mint, mint_account.unwrap().key)?;
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
        let token_account_metadata = Self::get_account_info(ctx, index + 1)?;
        let transfer_authority_info = Self::get_account_info(ctx, index + 2)?;

        if self.burn {
            let mint_account = Self::get_account_info(ctx, index + 3)?;
            let mint_master_edition = Self::get_account_info(ctx, index + 4)?;
            let mint_collection_metadata = Self::get_account_info(ctx, index + 5)?;

            let burn_nft_infos = vec![
                token_account_metadata.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                mint_account.to_account_info(),
                token_account_info.to_account_info(),
                mint_master_edition.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                mint_collection_metadata.to_account_info(),
            ];

            invoke(
                &burn_nft(
                    mpl_token_metadata::ID,
                    token_account_metadata.key(),
                    ctx.accounts.payer.key(),
                    mint_account.key(),
                    token_account_info.key(),
                    mint_master_edition.key(),
                    ::spl_token::ID,
                    Some(mint_collection_metadata.key()),
                ),
                burn_nft_infos.as_slice(),
            )?;
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
