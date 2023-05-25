use std::collections::HashMap;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::{
    assertions::{assert_initialized, assert_owned_by},
    error::MetadataError,
    pda::PREFIX,
    state::{
        Creator, Data, Metadata, TokenRecord, TokenState, MAX_CREATOR_LIMIT, MAX_NAME_LENGTH,
        MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
    },
};

pub fn assert_data_valid(
    data: &Data,
    update_authority: &Pubkey,
    existing_metadata: &Metadata,
    allow_direct_creator_writes: bool,
    update_authority_is_signer: bool,
) -> ProgramResult {
    if data.name.len() > MAX_NAME_LENGTH {
        return Err(MetadataError::NameTooLong.into());
    }

    if data.symbol.len() > MAX_SYMBOL_LENGTH {
        return Err(MetadataError::SymbolTooLong.into());
    }

    if data.uri.len() > MAX_URI_LENGTH {
        return Err(MetadataError::UriTooLong.into());
    }

    if data.seller_fee_basis_points > 10000 {
        return Err(MetadataError::InvalidBasisPoints.into());
    }

    // If the user passes in creators we get a reference to it, otherwise if the user passes in
    // None we make sure no current creators are verified before returning and allowing them to set
    // creators field to None.
    let creators = match data.creators {
        Some(ref creators) => creators,
        None => {
            if let Some(ref existing_creators) = existing_metadata.data.creators {
                if existing_creators.iter().any(|c| c.verified) {
                    return Err(MetadataError::CannotRemoveVerifiedCreator.into());
                }
            }
            return Ok(());
        }
    };

    if creators.len() > MAX_CREATOR_LIMIT {
        return Err(MetadataError::CreatorsTooLong.into());
    }

    if creators.is_empty() {
        return Err(MetadataError::CreatorsMustBeAtleastOne.into());
    }

    // Store caller-supplied creator's array into a hashmap for direct lookup.
    let new_creators_map: HashMap<&Pubkey, &Creator> =
        creators.iter().map(|c| (&c.address, c)).collect();

    // Do not allow duplicate entries in the creator's array.
    if new_creators_map.len() != creators.len() {
        return Err(MetadataError::DuplicateCreatorAddress.into());
    }

    // If there is an existing creator's array, store this in a hashmap as well.
    let existing_creators_map: Option<HashMap<&Pubkey, &Creator>> = existing_metadata
        .data
        .creators
        .as_ref()
        .map(|existing_creators| existing_creators.iter().map(|c| (&c.address, c)).collect());

    // Loop over new creator's map.
    let mut share_total: u8 = 0;
    for (address, creator) in &new_creators_map {
        // Add up creator shares.  After looping through all creators, will
        // verify it adds up to 100%.
        share_total = share_total
            .checked_add(creator.share)
            .ok_or(MetadataError::NumericalOverflowError)?;

        // If this flag is set we are allowing any and all creators to be marked as verified
        // without further checking.  This can only be done in special circumstances when the
        // metadata is fully trusted such as when minting a limited edition.  Note we are still
        // checking that creator share adds up to 100%.
        if allow_direct_creator_writes {
            continue;
        }

        // If this specific creator (of this loop iteration) is a signer and an update
        // authority, then we are fine with this creator either setting or clearing its
        // own `creator.verified` flag.
        if update_authority_is_signer && **address == *update_authority {
            continue;
        }

        // If the previous two conditions are not true then we check the state in the existing
        // metadata creators array (if it exists) before allowing `creator.verified` to be set.
        if let Some(existing_creators_map) = &existing_creators_map {
            if existing_creators_map.contains_key(address) {
                // If this specific creator (of this loop iteration) is in the existing
                // creator's array, then it's `creator.verified` flag must match the existing
                // state.
                if creator.verified && !existing_creators_map[address].verified {
                    return Err(MetadataError::CannotVerifyAnotherCreator.into());
                } else if !creator.verified && existing_creators_map[address].verified {
                    return Err(MetadataError::CannotUnverifyAnotherCreator.into());
                }
            } else if creator.verified {
                // If this specific creator is not in the existing creator's array, then we
                // cannot set `creator.verified`.
                return Err(MetadataError::CannotVerifyAnotherCreator.into());
            }
        } else if creator.verified {
            // If there is no existing creators array, we cannot set `creator.verified`.
            return Err(MetadataError::CannotVerifyAnotherCreator.into());
        }
    }

    // Ensure share total is 100%.
    if share_total != 100 {
        return Err(MetadataError::ShareTotalMustBe100.into());
    }

    // Next make sure there were not any existing creators that were already verified but not
    // listed in the new creator's array.
    if allow_direct_creator_writes {
        return Ok(());
    } else if let Some(existing_creators_map) = &existing_creators_map {
        for (address, existing_creator) in existing_creators_map {
            // If this specific existing creator (of this loop iteration is a signer and an
            // update authority, then we are fine with this creator clearing its own
            // `creator.verified` flag.
            if update_authority_is_signer && **address == *update_authority {
                continue;
            } else if !new_creators_map.contains_key(address) && existing_creator.verified {
                return Err(MetadataError::CannotUnverifyAnotherCreator.into());
            }
        }
    }

    Ok(())
}

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

pub fn assert_currently_holding(
    program_id: &Pubkey,
    owner_info: &AccountInfo,
    metadata_info: &AccountInfo,
    metadata: &Metadata,
    mint_info: &AccountInfo,
    token_account_info: &AccountInfo,
) -> ProgramResult {
    assert_holding_amount(
        program_id,
        owner_info,
        metadata_info,
        metadata,
        mint_info,
        token_account_info,
        1,
    )
}

pub fn assert_holding_amount(
    program_id: &Pubkey,
    owner_info: &AccountInfo,
    metadata_info: &AccountInfo,
    metadata: &Metadata,
    mint_info: &AccountInfo,
    token_account_info: &AccountInfo,
    amount: u64,
) -> ProgramResult {
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::ID)?;

    let token_account: Account = assert_initialized(token_account_info)?;

    assert_owned_by(token_account_info, &spl_token::ID)?;

    if token_account.owner != *owner_info.key {
        return Err(MetadataError::InvalidOwner.into());
    }

    if token_account.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if token_account.amount < amount {
        return Err(MetadataError::InsufficientTokenBalance.into());
    }

    if token_account.mint != metadata.mint {
        return Err(MetadataError::MintMismatch.into());
    }
    Ok(())
}

pub fn assert_metadata_valid(
    program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    metadata_account_info: &AccountInfo,
) -> ProgramResult {
    let seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
    let (metadata_pubkey, _) = Pubkey::find_program_address(seeds, program_id);
    if metadata_pubkey != *metadata_account_info.key {
        return Err(MetadataError::InvalidMetadataKey.into());
    }

    Ok(())
}

pub fn assert_state(token_record: &TokenRecord, state: TokenState) -> ProgramResult {
    match state {
        TokenState::Locked => {
            if !token_record.is_locked() {
                return Err(MetadataError::UnlockedToken.into());
            }
        }
        TokenState::Unlocked => {
            if token_record.is_locked() {
                return Err(MetadataError::LockedToken.into());
            }
        }
        TokenState::Listed => {
            if !matches!(token_record.state, TokenState::Listed) {
                return Err(MetadataError::IncorrectTokenState.into());
            }
        }
    }

    Ok(())
}

pub fn assert_not_locked(token_record: &TokenRecord) -> ProgramResult {
    assert_state(token_record, TokenState::Unlocked)
}

pub fn assert_metadata_derivation(
    program_id: &Pubkey,
    metadata_info: &AccountInfo,
    mint_info: &AccountInfo,
) -> Result<u8, ProgramError> {
    let path = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (pubkey, bump) = Pubkey::find_program_address(path, program_id);
    if pubkey != *metadata_info.key {
        return Err(MetadataError::MintMismatch.into());
    }
    Ok(bump)
}
