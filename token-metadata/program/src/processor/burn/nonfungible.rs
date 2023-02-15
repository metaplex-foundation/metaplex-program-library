use super::*;

pub(crate) struct BurnNonFungibleArgs {
    pub(crate) collection_metadata: Option<Metadata>,
    pub(crate) metadata: Metadata,
}

pub(crate) fn burn_nonfungible(ctx: &Context<Burn>, args: BurnNonFungibleArgs) -> ProgramResult {
    let edition_info = ctx.accounts.edition_info.unwrap();

    // If the NFT is a verified part of a collection but the user has not provided the collection
    // metadata account, we cannot burn it because we need to check if we need to decrement the collection size.
    if args.collection_metadata.is_none()
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

    // Has a valid Master Edition or Print Edition.
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        ctx.accounts.mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    assert_derivation(&crate::ID, edition_info, &edition_info_path)?;

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

    if let Some(mut collection_metadata) = args.collection_metadata {
        if ctx.accounts.collection_metadata_info.is_none() {
            return Err(MetadataError::MissingCollectionMetadata.into());
        }
        let collection_metadata_info = ctx.accounts.collection_metadata_info.unwrap();

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
            // Owned by token metadata program.
            assert_owned_by(collection_metadata_info, &crate::ID)?;

            // NFT is actually a verified member of the specified collection.
            assert_verified_member_of_collection(&args.metadata, &collection_metadata)?;

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
