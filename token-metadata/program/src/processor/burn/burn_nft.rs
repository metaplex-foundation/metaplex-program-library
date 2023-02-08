use arrayref::array_ref;
use mpl_utils::{
    assert_signer,
    token::{spl_token_burn, spl_token_close, TokenBurnParams, TokenCloseParams},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_memory::sol_memset,
    pubkey::Pubkey,
};

use crate::{
    assertions::{
        assert_derivation, assert_owned_by,
        metadata::{assert_currently_holding, assert_verified_member_of_collection},
    },
    error::MetadataError,
    pda::find_metadata_account,
    state::{
        Collection, CollectionDetails, Key, Metadata, TokenMetadataAccount, EDITION,
        MAX_METADATA_LEN, PREFIX,
    },
    utils::clean_write_metadata,
};

pub fn process_burn_nft<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;

    let collection_nft_provided = accounts.len() == 7;

    let metadata = Metadata::from_account_info(metadata_info)?;

    // If the NFT is a verified part of a collection but the user has not provided the collection
    // metadata account, we cannot burn it because we need to check if we need to decrement the collection size.
    if !collection_nft_provided
        && metadata.collection.is_some()
        && metadata.collection.as_ref().unwrap().verified
    {
        return Err(MetadataError::MissingCollectionMetadata.into());
    }

    // Ensure this is a Master Edition and not a Print.

    // Scope this so the borrow gets dropped and doesn't conflict with the mut borrow
    // later in the handler when overwriting data.
    {
        let edition_account_data = edition_info.try_borrow_data()?;

        // First byte is the object key.
        let key = edition_account_data
            .first()
            .ok_or(MetadataError::InvalidMasterEdition)?;
        if *key != Key::MasterEditionV1 as u8 && *key != Key::MasterEditionV2 as u8 {
            return Err(MetadataError::NotAMasterEdition.into());
        }

        // Next eight bytes are the supply, which must be converted to a u64.
        let supply_bytes = array_ref![edition_account_data, 1, 8];
        let supply = u64::from_le_bytes(*supply_bytes);

        // Cannot burn Master Editions with existing prints in this handler.
        if supply > 0 {
            return Err(MetadataError::MasterEditionHasPrints.into());
        }
    }

    // Checks:
    // * Metadata is owned by the token-metadata program
    // * Mint is owned by the spl-token program
    // * Token is owned by the spl-token program
    // * Token account is initialized
    // * Token account data owner is 'owner'
    // * Token account belongs to mint
    // * Token account has 1 or more tokens
    // * Mint matches metadata.mint
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_info,
    )?;

    // Owned by token-metadata program.
    assert_owned_by(edition_info, program_id)?;

    // Owner is a signer.
    assert_signer(owner_info)?;

    // Has a valid Master Edition or Print Edition.
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    assert_derivation(program_id, edition_info, &edition_info_path)?;

    // Burn the SPL token
    let params = TokenBurnParams {
        mint: mint_info.clone(),
        source: token_info.clone(),
        authority: owner_info.clone(),
        token_program: spl_token_program_info.clone(),
        amount: 1,
        authority_signer_seeds: None,
    };
    spl_token_burn(params)?;

    // Close token account.
    let params = TokenCloseParams {
        token_program: spl_token_program_info.clone(),
        account: token_info.clone(),
        destination: owner_info.clone(),
        owner: owner_info.clone(),
        authority_signer_seeds: None,
    };
    spl_token_close(params)?;

    // Close metadata and edition accounts by transferring rent funds to owner and
    // zeroing out the data.
    let metadata_lamports = metadata_info.lamports();
    **metadata_info.try_borrow_mut_lamports()? = 0;
    **owner_info.try_borrow_mut_lamports()? = owner_info
        .lamports()
        .checked_add(metadata_lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;

    let edition_lamports = edition_info.lamports();
    **edition_info.try_borrow_mut_lamports()? = 0;
    **owner_info.try_borrow_mut_lamports()? = owner_info
        .lamports()
        .checked_add(edition_lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;

    let metadata_data = &mut metadata_info.try_borrow_mut_data()?;
    let edition_data = &mut edition_info.try_borrow_mut_data()?;
    let edition_data_len = edition_data.len();

    // Use MAX_METADATA_LEN because it has unused padding, making it longer than current metadata len.
    sol_memset(metadata_data, 0, MAX_METADATA_LEN);
    sol_memset(edition_data, 0, edition_data_len);

    if collection_nft_provided {
        let collection_metadata_info = next_account_info(account_info_iter)?;
        if collection_metadata_info.data_is_empty() {
            let Collection {
                key: expected_collection_mint,
                ..
            } = metadata
                .collection
                .as_ref()
                .ok_or(MetadataError::CollectionNotFound)?;
            let (expected_collection_metadata_key, _) =
                find_metadata_account(expected_collection_mint);
            // Check that the empty collection account passed in is actually the burned collection nft
            if expected_collection_metadata_key != *collection_metadata_info.key {
                return Err(MetadataError::NotAMemberOfCollection.into());
            }
        } else {
            // Get our collections metadata into a Rust type so we can update the collection size after burning.
            let mut collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

            // Owned by token metadata program.
            assert_owned_by(collection_metadata_info, program_id)?;

            // NFT is actually a verified member of the specified collection.
            assert_verified_member_of_collection(&metadata, &collection_metadata)?;

            // Update collection size if it's sized.
            if let Some(ref details) = collection_metadata.collection_details {
                match details {
                    CollectionDetails::V1 { size } => {
                        collection_metadata.collection_details = Some(CollectionDetails::V1 {
                            size: size
                                .checked_sub(1)
                                .ok_or(MetadataError::NumericalOverflowError)?,
                        });
                        clean_write_metadata(&mut collection_metadata, collection_metadata_info)?;
                    }
                }
            }
        }
    }

    Ok(())
}
