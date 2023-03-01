use crate::{
    assertions::{assert_owned_by, metadata::assert_metadata_derivation},
    error::MetadataError,
    instruction::{Context, MetadataDelegateRole, Unverify, Verify},
    state::{AuthorityRequest, AuthorityType, Metadata, TokenMetadataAccount},
    utils::{clean_write_metadata, decrement_collection_size, increment_collection_size},
};
use mpl_utils::assert_signer;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

pub(crate) fn verify_collection_v1(program_id: &Pubkey, ctx: Context<Verify>) -> ProgramResult {
    // Assert program ownership/signers.

    // Authority account is the collection authority and must be a signer.
    assert_signer(ctx.accounts.authority_info)?;

    if let Some(delegate_record_info) = ctx.accounts.delegate_record_info {
        assert_owned_by(delegate_record_info, program_id)?;
    }

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

    // Verify the collection in the item's metadata matches the collection mint.  Also verify
    // the collection metadata matches the collection mint, and the collection edition derivation.
    match metadata.collection {
        Some(ref mut collection) => {
            // Short-circuit if it's already verified.
            if collection.verified {
                return Err(MetadataError::AlreadyVerified.into());
            }

            // The collection parent must be the actual parent of the collection item
            // and the metadata must be derived from the mint.
            if collection.key != *collection_mint_info.key
                || collection_metadata.mint != *collection_mint_info.key
            {
                return Err(MetadataError::MintMismatch.into());
            }

            // Set item metadata collection to verified.
            collection.verified = true;

            // In the case of a sized collection, update the size on the collection parent.
            if collection_metadata.collection_details.is_some() {
                increment_collection_size(&mut collection_metadata, collection_metadata_info)?;
            }

            // Reserialize metadata.
            clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)
        }
        None => Err(MetadataError::CollectionNotFound.into()),
    }
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub(crate) fn unverify_collection_v1(program_id: &Pubkey, ctx: Context<Unverify>) -> ProgramResult {
    // Not tested so returning.
    return Err(MetadataError::FeatureNotSupported.into());

    // Assert program ownership/signers.

    // Authority account is the collection authority and must be a signer.
    assert_signer(ctx.accounts.authority_info)?;

    if let Some(delegate_record_info) = ctx.accounts.delegate_record_info {
        assert_owned_by(delegate_record_info, program_id)?;
    }

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

    // Check if the collection metadata account is burned. If it is, there's no sized data to
    // update and the user can simply unverify the NFT.
    let parent_burned = collection_metadata_info.data_is_empty();

    if parent_burned {
        // If the collection parent is burned, we need to use an authority for the item rather than
        // the collection.  The required authority is either the item's metadata update authority,
        // or an update delegate for the item.  This call fails if no valid authority is present.
        let authority_response = AuthorityType::get_authority_type(AuthorityRequest {
            authority: ctx.accounts.authority_info.key,
            update_authority: &metadata.update_authority,
            mint: &metadata.mint,
            metadata_delegate_record_info: ctx.accounts.delegate_record_info,
            metadata_delegate_roles: vec![MetadataDelegateRole::Update],
            precedence: &[AuthorityType::Metadata, AuthorityType::MetadataDelegate],
            ..Default::default()
        })?;

        // Validate that authority type is expected.
        match authority_response.authority_type {
            AuthorityType::Metadata | AuthorityType::MetadataDelegate => (),
            _ => return Err(MetadataError::UpdateAuthorityIncorrect.into()),
        }
    } else {
        // If the parent is not burned, we need to ensure the collection metadata account is owned
        // by the token metadata program.
        assert_owned_by(collection_metadata_info, program_id)?;

        // Now we can deserialize the collection metadata account.
        let mut collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

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

        // In the case of a sized collection, update the size on the collection parent.
        if collection_metadata.collection_details.is_some() {
            decrement_collection_size(&mut collection_metadata, collection_metadata_info)?;
        }
    }

    // Set item metadata collection to unverified.
    collection.verified = false;

    // Reserialize metadata.
    clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)?;
    Ok(())
}
