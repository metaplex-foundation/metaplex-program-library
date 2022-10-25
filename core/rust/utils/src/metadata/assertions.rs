use mpl_token_metadata::{
    error::MetadataError,
    pda::find_master_edition_account,
    state::{MasterEditionV1, Metadata, EDITION, PREFIX},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_option::COption,
    program_pack::Pack, pubkey::Pubkey,
};

pub use token::*;

use crate::assertions::{assert_initialized, assert_owned_by};

pub fn assert_update_authority_is_correct(
    metadata: &Metadata,
    update_authority_info: &AccountInfo,
) -> ProgramResult {
    if metadata.update_authority != *update_authority_info.key {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }

    if !update_authority_info.is_signer {
        return Err(MetadataError::UpdateAuthorityIsNotSigner.into());
    }

    Ok(())
}

// Todo deprecate this for assert derivation
pub fn assert_edition_valid(
    program_id: &Pubkey,
    mint: &Pubkey,
    edition_account_info: &AccountInfo,
) -> ProgramResult {
    let edition_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint.as_ref(),
        EDITION.as_bytes(),
    ];
    let (edition_key, _) = Pubkey::find_program_address(edition_seeds, program_id);
    if edition_key != *edition_account_info.key {
        return Err(MetadataError::InvalidEditionKey.into());
    }

    Ok(())
}

pub fn assert_verified_member_of_collection(
    item_metadata: &Metadata,
    collection_metadata: &Metadata,
) -> ProgramResult {
    if let Some(ref collection) = item_metadata.collection {
        if collection_metadata.mint != collection.key {
            return Err(MetadataError::NotAMemberOfCollection.into());
        }
        if !collection.verified {
            return Err(MetadataError::NotVerifiedMemberOfCollection.into());
        }
    } else {
        return Err(MetadataError::NotAMemberOfCollection.into());
    }

    Ok(())
}

pub fn assert_freeze_authority_matches_mint(
    freeze_authority: &COption<Pubkey>,
    freeze_authority_info: &AccountInfo,
) -> ProgramResult {
    match freeze_authority {
        COption::None => {
            return Err(MetadataError::InvalidFreezeAuthority.into());
        }
        COption::Some(key) => {
            if freeze_authority_info.key != key {
                return Err(MetadataError::InvalidFreezeAuthority.into());
            }
        }
    }
    Ok(())
}

pub fn assert_mint_authority_matches_mint(
    mint_authority: &COption<Pubkey>,
    mint_authority_info: &AccountInfo,
) -> ProgramResult {
    match mint_authority {
        COption::None => {
            return Err(MetadataError::InvalidMintAuthority.into());
        }
        COption::Some(key) => {
            if mint_authority_info.key != key {
                return Err(MetadataError::InvalidMintAuthority.into());
            }
        }
    }

    if !mint_authority_info.is_signer {
        return Err(MetadataError::NotMintAuthority.into());
    }

    Ok(())
}

#[cfg(feature = "spl-token")]
mod token {
    use spl_token::state::{Account, Mint};

    use super::*;

    pub fn assert_supply_invariance(
        master_edition: &MasterEditionV1,
        printing_mint: &Mint,
        new_supply: u64,
    ) -> ProgramResult {
        // The supply of printed tokens and the supply of the master edition should, when added, never exceed max supply.
        // Every time a printed token is burned, master edition.supply goes up by 1.
        if let Some(max_supply) = master_edition.max_supply {
            let current_supply = printing_mint
                .supply
                .checked_add(master_edition.supply)
                .ok_or(MetadataError::NumericalOverflowError)?;
            let new_proposed_supply = current_supply
                .checked_add(new_supply)
                .ok_or(MetadataError::NumericalOverflowError)?;
            if new_proposed_supply > max_supply {
                return Err(MetadataError::PrintingWouldBreachMaximumSupply.into());
            }
        }

        Ok(())
    }
    pub fn assert_currently_holding(
        program_id: &Pubkey,
        owner_info: &AccountInfo,
        metadata_info: &AccountInfo,
        metadata: &Metadata,
        mint_info: &AccountInfo,
        token_account_info: &AccountInfo,
    ) -> ProgramResult {
        assert_owned_by(metadata_info, program_id, MetadataError::InvalidOwner)?;
        assert_owned_by(mint_info, &spl_token::id(), MetadataError::InvalidOwner)?;

        let token_account: Account =
            assert_initialized(token_account_info, MetadataError::Uninitialized)?;

        assert_owned_by(
            token_account_info,
            &spl_token::id(),
            MetadataError::InvalidOwner,
        )?;

        if token_account.owner != *owner_info.key {
            return Err(MetadataError::InvalidOwner.into());
        }

        if token_account.mint != *mint_info.key {
            return Err(MetadataError::MintMismatch.into());
        }

        if token_account.amount < 1 {
            return Err(MetadataError::NotEnoughTokens.into());
        }

        if token_account.mint != metadata.mint {
            return Err(MetadataError::MintMismatch.into());
        }
        Ok(())
    }

    pub fn assert_edition_is_not_mint_authority(mint_account_info: &AccountInfo) -> ProgramResult {
        let mint = Mint::unpack_from_slice(*mint_account_info.try_borrow_mut_data()?)?;

        let (edition_pda, _) = find_master_edition_account(mint_account_info.key);

        if mint.mint_authority == COption::Some(edition_pda) {
            return Err(MetadataError::MissingEditionAccount.into());
        }

        Ok(())
    }

    pub fn assert_delegated_tokens(
        delegate: &AccountInfo,
        mint_info: &AccountInfo,
        token_account_info: &AccountInfo,
    ) -> ProgramResult {
        assert_owned_by(mint_info, &spl_token::id(), MetadataError::InvalidOwner)?;

        let token_account: Account =
            assert_initialized(token_account_info, MetadataError::Uninitialized)?;

        assert_owned_by(
            token_account_info,
            &spl_token::id(),
            MetadataError::InvalidOwner,
        )?;

        if token_account.mint != *mint_info.key {
            return Err(MetadataError::MintMismatch.into());
        }

        if token_account.amount < 1 {
            return Err(MetadataError::NotEnoughTokens.into());
        }

        if token_account.delegate == COption::None
            || token_account.delegated_amount != token_account.amount
            || token_account.delegate.unwrap() != *delegate.key
        {
            return Err(MetadataError::InvalidDelegate.into());
        }
        Ok(())
    }
}
