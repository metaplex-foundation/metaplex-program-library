use crate::{
    assertions::{
        assert_owned_by, collection::assert_collection_verify_is_valid,
        metadata::assert_metadata_derivation,
    },
    error::MetadataError,
    instruction::{Context, MetadataDelegateRole, Unverify, Verify},
    state::{AuthorityRequest, AuthorityType, Metadata, TokenMetadataAccount},
    utils::{clean_write_metadata, decrement_collection_size, increment_collection_size},
};
use mpl_utils::assert_signer;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

pub(crate) fn verify_collection_v1(program_id: &Pubkey, ctx: Context<Verify>) -> ProgramResult {
    // Assert program ownership/signers.

    // Authority account must be a signer.  What this authority account actually represents is
    // checked below.
    assert_signer(ctx.accounts.authority_info)?;

    // Note: `ctx.accounts.delegate_record_info` owner check done inside of `get_authority_type`.

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;

    let collection_mint_info = ctx
        .accounts
        .collection_mint_info
        .ok_or(MetadataError::MissingCollectionMint)?;
    assert_owned_by(collection_mint_info, &spl_token::ID)?;

    let collection_metadata_info = ctx
        .accounts
        .collection_metadata_info
        .ok_or(MetadataError::MissingCollectionMetadata)?;
    assert_owned_by(collection_metadata_info, program_id)?;

    let collection_master_edition_info = ctx
        .accounts
        .collection_master_edition_info
        .ok_or(MetadataError::MissingCollectionMasterEdition)?;
    assert_owned_by(collection_master_edition_info, program_id)?;

    // Deserialize item metadata and collection parent metadata.
    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    let mut collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

    // Short circuit if its already verified.  If we let the rest of this instruction run, then for
    // sized collections we would end up with invalid size data.
    if let Some(collection) = &metadata.collection {
        if collection.verified {
            return Ok(());
        }
    }

    // Verify the collection in the item's metadata matches the collection mint.  Also verify
    // the collection metadata matches the collection mint, and the collection edition derivation.
    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_metadata,
        collection_mint_info,
        collection_master_edition_info,
    )?;

    // Determines if we have a valid authority to perform the collection verification.  The
    // required authority is either the collection parent's metadata update authority, or a
    // collection delegate for the collection parent.  This call fails if no valid authority is
    // present.
    let authority_response = AuthorityType::get_authority_type(AuthorityRequest {
        authority: ctx.accounts.authority_info.key,
        update_authority: &collection_metadata.update_authority,
        mint: collection_mint_info.key,
        metadata_delegate_record_info: ctx.accounts.delegate_record_info,
        metadata_delegate_roles: vec![MetadataDelegateRole::Collection],
        precedence: &[AuthorityType::Metadata, AuthorityType::MetadataDelegate],
        ..Default::default()
    })?;

    // Validate that authority type is expected.
    match authority_response.authority_type {
        AuthorityType::Metadata | AuthorityType::MetadataDelegate => (),
        _ => return Err(MetadataError::UpdateAuthorityIncorrect.into()),
    }

    // Destructure the collection field from the item metadata.
    match metadata.collection.as_mut() {
        Some(collection) => {
            // Set item metadata collection to verified.
            collection.verified = true;

            // In the case of a sized collection, update the size on the collection parent.
            if collection_metadata.collection_details.is_some() {
                increment_collection_size(&mut collection_metadata, collection_metadata_info)?;
            }
        }
        None => return Err(MetadataError::CollectionNotFound.into()),
    };

    // Reserialize metadata.
    clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)
}

pub(crate) fn unverify_collection_v1(program_id: &Pubkey, ctx: Context<Unverify>) -> ProgramResult {
    // Assert program ownership/signers.

    // Authority account must be a signer.  What this authority account actually represents is
    // checked below.
    assert_signer(ctx.accounts.authority_info)?;

    // Note: `ctx.accounts.delegate_record_info` owner check done inside of `get_authority_type`.

    assert_owned_by(ctx.accounts.metadata_info, program_id)?;

    let collection_mint_info = ctx
        .accounts
        .collection_mint_info
        .ok_or(MetadataError::MissingCollectionMint)?;
    assert_owned_by(collection_mint_info, &spl_token::ID)?;

    let collection_metadata_info = ctx
        .accounts
        .collection_metadata_info
        .ok_or(MetadataError::MissingCollectionMetadata)?;
    // Owner check done below after derivation check (since collection parent may be
    // burned and if so would be owned by System Program).

    // Deserialize item metadata.
    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

    // Destructure the collection field from the item metadata.  If there's no collection set, we
    // can just short-circuit since there's nothing to unverify.
    let collection = match metadata.collection.as_mut() {
        Some(collection) => collection,
        None => return Ok(()),
    };

    // Short-circuit if it's already unverified.
    if !collection.verified {
        return Ok(());
    }

    // The collection parent must be the actual parent of the collection item.
    if collection.key != *collection_mint_info.key {
        return Err(MetadataError::NotAMemberOfCollection.into());
    }

    // Ensure the metadata is derived from the mint.
    assert_metadata_derivation(program_id, collection_metadata_info, collection_mint_info)?;

    // Set up authority request for a Metadata delegate.  We will fill in update authority
    // and metadata delegate roles below.
    let mut auth_request = AuthorityRequest {
        authority: ctx.accounts.authority_info.key,
        metadata_delegate_record_info: ctx.accounts.delegate_record_info,
        precedence: &[AuthorityType::Metadata, AuthorityType::MetadataDelegate],
        ..Default::default()
    };

    // If the collection parent metadata account has been burned then its data will be empty.
    let parent_burned =
        collection_metadata_info.data_is_empty() || collection_metadata_info.data.borrow()[0] == 0;

    let authority_response = if parent_burned {
        // If the collection parent is burned, we need to use an authority for the item rather than
        // the collection.  The required authority is either the item's metadata update authority
        // or a delegate for the item that can update the item's collection field.  This call fails
        // if no valid authority is present.
        auth_request.mint = &metadata.mint;
        auth_request.update_authority = &metadata.update_authority;
        auth_request.metadata_delegate_roles = vec![
            MetadataDelegateRole::Collection,
            MetadataDelegateRole::CollectionItem,
        ];
        AuthorityType::get_authority_type(auth_request)
    } else {
        // If the parent is not burned, we need to ensure the collection metadata account is owned
        // by the token metadata program.
        assert_owned_by(collection_metadata_info, program_id)?;

        // Now we can deserialize the collection metadata account.
        let mut collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

        // In the case of a sized collection, update the size on the collection parent.
        if collection_metadata.collection_details.is_some() {
            decrement_collection_size(&mut collection_metadata, collection_metadata_info)?;
        }

        // If the collection parent is not burned, the required authority is either the collection
        // parent's metadata update authority, or a collection delegate for the collection parent.
        // This call fails if no valid authority is present.
        //
        // Note that this is sending the delegate in the `metadata_delegate_roles` vec and NOT the
        // `collection_metadata_delegate_roles` vec because in this case we are authorizing using
        // the collection parent's update authority.
        auth_request.mint = collection_mint_info.key;
        auth_request.update_authority = &collection_metadata.update_authority;
        auth_request.metadata_delegate_roles = vec![MetadataDelegateRole::Collection];
        AuthorityType::get_authority_type(auth_request)
    }?;

    // Validate that authority type is expected.
    match authority_response.authority_type {
        AuthorityType::Metadata | AuthorityType::MetadataDelegate => (),
        _ => return Err(MetadataError::UpdateAuthorityIncorrect.into()),
    }

    // Set item metadata collection to unverified.
    collection.verified = false;

    // Reserialize metadata.
    clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)
}
