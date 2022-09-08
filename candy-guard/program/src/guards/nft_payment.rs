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
        let index = evaluation_context.account_cursor;
        // validates that we received all required accounts
        let token_account_info = Self::get_account_info(ctx, index)?;
        let token_metadata = Self::get_account_info(ctx, index + 1)?;
        evaluation_context.account_cursor += 2;

        let metadata: Metadata = Metadata::from_account_info(token_metadata)?;
        // validates the account information
        assert_keys_equal(token_metadata.owner, &mpl_token_metadata::id())?;

        let token_account = assert_is_token_account(
            token_account_info,
            &ctx.accounts.payer.key(),
            &metadata.mint,
        )?;

        if token_account.amount < 1 {
            return err!(CandyGuardError::NotEnoughTokens);
        }

        match metadata.collection {
            Some(c) if c.verified && c.key == self.required_collection => Ok(()),
            _ => Err(CandyGuardError::InvalidNFTCollectionPayment),
        }?;

        if self.burn {
            let _token_edition = Self::get_account_info(ctx, index + 2)?;
            let mint_account = Self::get_account_info(ctx, index + 3)?;
            let _mint_collection_metadata = Self::get_account_info(ctx, index + 4)?;
            evaluation_context.account_cursor += 3;

            let metadata: Metadata = Metadata::from_account_info(token_metadata)?;
            // validates the account information
            assert_keys_equal(token_metadata.owner, &mpl_token_metadata::id())?;
            assert_keys_equal(&metadata.mint, mint_account.key)?;
        } else {
            let _transfer_authority = Self::get_account_info(ctx, index + 2)?;
            let _destination_ata = Self::get_account_info(ctx, index + 3)?;
            evaluation_context.account_cursor += 2;
        }

        evaluation_context
            .indices
            .insert("nft_payment_index", index);

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
        let token_account = Self::get_account_info(ctx, index)?;

        if self.burn {
            let token_metadata = Self::get_account_info(ctx, index + 1)?;
            let token_edition = Self::get_account_info(ctx, index + 2)?;
            let mint_account = Self::get_account_info(ctx, index + 3)?;
            let mint_collection_metadata = Self::get_account_info(ctx, index + 4)?;

            let burn_nft_infos = vec![
                token_metadata.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                mint_account.to_account_info(),
                token_account.to_account_info(),
                token_edition.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                mint_collection_metadata.to_account_info(),
            ];

            invoke(
                &burn_nft(
                    mpl_token_metadata::ID,
                    token_metadata.key(),
                    ctx.accounts.payer.key(),
                    mint_account.key(),
                    token_account.key(),
                    token_edition.key(),
                    ::spl_token::ID,
                    Some(mint_collection_metadata.key()),
                ),
                burn_nft_infos.as_slice(),
            )?;
        } else {
            let transfer_authority = Self::get_account_info(ctx, index + 2)?;
            let destination_ata = Self::get_account_info(ctx, index + 3)?;

            spl_token_transfer(TokenTransferParams {
                source: token_account.to_account_info(),
                destination: destination_ata.to_account_info(),
                authority: transfer_authority.to_account_info(),
                authority_signer_seeds: &[],
                token_program: ctx.accounts.token_program.to_account_info(),
                amount: 1, // fixed to always require 1 NFT
            })?;
        }

        Ok(())
    }
}
