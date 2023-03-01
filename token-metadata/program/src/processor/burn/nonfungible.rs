use crate::utils::decrement_collection_size;

use super::*;

pub(crate) struct BurnNonFungibleArgs {
    pub(crate) metadata: Metadata,
}

pub(crate) fn burn_nonfungible(ctx: &Context<Burn>, args: BurnNonFungibleArgs) -> ProgramResult {
    let edition_info = ctx.accounts.edition_info.unwrap();

    // If you're passing in parent accounts for a nonfungible you're using this handler wrong.
    // Parent accounts are only for burning nonfungible editions.
    if ctx.accounts.parent_mint_info.is_some()
        || ctx.accounts.parent_edition_info.is_some()
        || ctx.accounts.parent_token_info.is_some()
        || ctx.accounts.edition_marker_info.is_some()
    {
        return Err(MetadataError::InvalidParentAccounts.into());
    }

    // If the NFT is a verified part of a collection but the user has not provided the collection
    // metadata account, we cannot burn it because we need to check if we need to decrement the collection size.
    if ctx.accounts.collection_metadata_info.is_none()
        && args.metadata.collection.is_some()
        && args.metadata.collection.as_ref().unwrap().verified
    {
        return Err(MetadataError::MissingCollectionMetadata.into());
    }

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

    // Cannot burn Master Editions with existing prints
    if supply > 0 {
        return Err(MetadataError::MasterEditionHasPrints.into());
    }

    // Drop the borrow since we're done with it and will need a new borrow in
    // close_program_account.
    drop(edition_account_data);

    // Burn the SPL token
    let params = TokenBurnParams {
        mint: ctx.accounts.mint_info.clone(),
        source: ctx.accounts.token_info.clone(),
        authority: ctx.accounts.authority_info.clone(),
        token_program: ctx.accounts.spl_token_program_info.clone(),
        amount: 1,
        authority_signer_seeds: None,
    };
    spl_token_burn(params)?;

    let params = TokenCloseParams {
        token_program: ctx.accounts.spl_token_program_info.clone(),
        account: ctx.accounts.token_info.clone(),
        destination: ctx.accounts.authority_info.clone(),
        owner: ctx.accounts.authority_info.clone(),
        authority_signer_seeds: None,
    };
    spl_token_close(params)?;

    close_program_account(ctx.accounts.metadata_info, ctx.accounts.authority_info)?;
    close_program_account(edition_info, ctx.accounts.authority_info)?;

    if let Some(collection_metadata_info) = ctx.accounts.collection_metadata_info {
        if collection_metadata_info.data_is_empty() {
            let Collection {
                key: expected_collection_mint,
                ..
            } = args
                .metadata
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
            assert_owned_by(collection_metadata_info, &crate::ID)?;
            let mut collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

            // NFT is actually a verified member of the specified collection.
            assert_verified_member_of_collection(&args.metadata, &collection_metadata)?;

            // Update collection size if it's sized.
            if collection_metadata.collection_details.is_some() {
                decrement_collection_size(&mut collection_metadata, collection_metadata_info)?;
            }
        }
    }

    Ok(())
}
