use crate::{
    assertions::{assert_owned_by, collection::assert_collection_verify_is_valid},
    error::MetadataError,
    instruction::{Context, MetadataDelegateRole, Verify, VerifyArgs},
    state::{AuthorityRequest, AuthorityType, Metadata, TokenMetadataAccount},
    utils::{clean_write_metadata, increment_collection_size},
};

use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn verify<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: VerifyArgs,
) -> ProgramResult {
    let context = Verify::to_context(accounts)?;

    match args {
        VerifyArgs::CreatorV1 => verify_creator_v1(program_id, context),
        VerifyArgs::CollectionV1 => verify_collection_v1(program_id, context),
    }
}

fn verify_creator_v1(program_id: &Pubkey, ctx: Context<Verify>) -> ProgramResult {
    // Assert program ownership/signers.

    // Authority account is the creator and must be a signer.
    assert_signer(ctx.accounts.authority_info)?;
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;

    // Deserialize item metadata.
    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

    // Find creator in creator array and if found mark as verified.
    match &mut metadata.data.creators {
        Some(creators) => {
            let creator = creators
                .iter_mut()
                .find(|c| c.address == *ctx.accounts.authority_info.key)
                .ok_or(MetadataError::CreatorNotFound)?;

            creator.verified = true;
        }
        None => return Err(MetadataError::NoCreatorsPresentOnMetadata.into()),
    }

    // Reserialize item metadata.
    clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)?;
    Ok(())
}

fn verify_collection_v1(program_id: &Pubkey, ctx: Context<Verify>) -> ProgramResult {
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

    // Don't verify already verified items, otherwise for sized collections we end up with
    // invalid size data.
    if let Some(collection) = &metadata.collection {
        if collection.verified {
            return Err(MetadataError::AlreadyVerified.into());
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

    // Determines if we have a valid authority to perform the collection verification.  This call
    // fails if no valid authority is present.
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

    // If the item has unverified collection data, we set it to be verified and in the case of a
    // sized collection, update the size on the Collection Parent.
    if let Some(collection) = &mut metadata.collection {
        if collection_metadata.collection_details.is_some() {
            increment_collection_size(&mut collection_metadata, collection_metadata_info)?;
        }

        collection.verified = true;
    } else {
        return Err(MetadataError::CollectionNotFound.into());
    }

    // Reserialize metadata.
    clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)?;
    Ok(())
}
