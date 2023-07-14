use super::*;

pub(crate) struct BurnNonFungibleArgs {
    pub(crate) metadata: Metadata,
    pub(crate) me_close_authority: bool,
}

pub(crate) fn burn_nonfungible(ctx: &Context<Burn>, args: BurnNonFungibleArgs) -> ProgramResult {
    let edition_info = ctx.accounts.edition_info.unwrap();

    // If you're passing in parent master edition accounts for a nonfungible you're using this handler wrong.
    // Parent accounts are only for burning nonfungible editions.
    if ctx.accounts.master_edition_mint_info.is_some()
        || ctx.accounts.master_edition_info.is_some()
        || ctx.accounts.master_edition_token_info.is_some()
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

    // Has a valid Master Edition or Print Edition.
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        ctx.accounts.mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let bump = assert_derivation(&crate::ID, edition_info, &edition_info_path)?;

    let edition_seeds = &[
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        ctx.accounts.mint_info.key.as_ref(),
        EDITION.as_bytes(),
        &[bump],
    ];

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

    let close_params = TokenCloseParams {
        account: ctx.accounts.token_info.clone(),
        destination: ctx.accounts.authority_info.clone(),
        owner: if args.me_close_authority {
            edition_info.clone()
        } else {
            ctx.accounts.authority_info.clone()
        },
        authority_signer_seeds: if args.me_close_authority {
            Some(edition_seeds.as_slice())
        } else {
            None
        },
        token_program: ctx.accounts.spl_token_program_info.clone(),
    };
    // CPIs panic if there's an error so unwrapping is fine here.
    mpl_utils::token::spl_token_close(close_params).unwrap();

    close_program_account(
        ctx.accounts.metadata_info,
        ctx.accounts.authority_info,
        Key::MetadataV1,
    )?;
    close_program_account(
        edition_info,
        ctx.accounts.authority_info,
        Key::MasterEditionV2,
    )?;

    if let Some(collection_metadata_info) = ctx.accounts.collection_metadata_info {
        // If collection parent is burned or Uninitialized because it stores fees, we don't need to decrement the size.
        if collection_metadata_info.data_is_empty()
            || collection_metadata_info.data.borrow()[0] == 0
        {
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
