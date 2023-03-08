use crate::{
    assertions::assert_owned_by,
    error::MetadataError,
    instruction::{Context, Unverify, Verify},
    state::{Creator, Metadata, TokenMetadataAccount},
    utils::clean_write_metadata,
};
use mpl_utils::assert_signer;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

pub(crate) fn verify_creator_v1(program_id: &Pubkey, ctx: Context<Verify>) -> ProgramResult {
    // Assert program ownership/signers.

    // Authority account is the creator and must be a signer.
    assert_signer(ctx.accounts.authority_info)?;
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;

    // Deserialize item metadata.
    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

    // Find creator in creator array and if found mark as verified.
    find_and_set_creator(
        &mut metadata.data.creators,
        *ctx.accounts.authority_info.key,
        true,
    )?;

    // Reserialize item metadata.
    clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)
}

pub(crate) fn unverify_creator_v1(program_id: &Pubkey, ctx: Context<Unverify>) -> ProgramResult {
    // Assert program ownership/signers.

    // Authority account is the creator and must be a signer.
    assert_signer(ctx.accounts.authority_info)?;
    assert_owned_by(ctx.accounts.metadata_info, program_id)?;

    // Deserialize item metadata.
    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

    // Find creator in creator array and if found mark as verified.
    find_and_set_creator(
        &mut metadata.data.creators,
        *ctx.accounts.authority_info.key,
        false,
    )?;

    // Reserialize item metadata.
    clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)
}

fn find_and_set_creator(
    creators: &mut Option<Vec<Creator>>,
    creator_to_match: Pubkey,
    verified: bool,
) -> ProgramResult {
    // Find creator in creator array and if found mark as verified.
    match creators {
        Some(creators) => {
            let creator = creators
                .iter_mut()
                .find(|c| c.address == creator_to_match)
                .ok_or(MetadataError::CreatorNotFound)?;

            creator.verified = verified;
            Ok(())
        }
        None => Err(MetadataError::NoCreatorsPresentOnMetadata.into()),
    }
}
