use crate::{
    assertions::assert_owned_by,
    error::MetadataError,
    instruction::{Context, Verify},
    state::{Metadata, TokenMetadataAccount},
    utils::clean_write_metadata,
};
use mpl_utils::assert_signer;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

pub(crate) fn creator_verification_v1(
    program_id: &Pubkey,
    ctx: Context<Verify>,
    verified: bool,
) -> ProgramResult {
    if !verified {
        return Err(MetadataError::FeatureNotSupported.into());
    }
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

            creator.verified = verified;
        }
        None => return Err(MetadataError::NoCreatorsPresentOnMetadata.into()),
    }

    // Reserialize item metadata.
    clean_write_metadata(&mut metadata, ctx.accounts.metadata_info)?;
    Ok(())
}
