use crate::state::{Collection, Data, Key, Metadata, TokenStandard, Uses};
use borsh::BorshDeserialize;
use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

// Custom deserialization function to handle NFTs with corrupted data.
pub fn meta_deser(buf: &mut &[u8]) -> Result<Metadata, ProgramError> {
    // Metadata corruption shouldn't appear until after edition_nonce.
    let key: Key = BorshDeserialize::deserialize(buf)?;
    let update_authority: Pubkey = BorshDeserialize::deserialize(buf)?;
    let mint: Pubkey = BorshDeserialize::deserialize(buf)?;
    let data: Data = BorshDeserialize::deserialize(buf)?;
    let primary_sale_happened: bool = BorshDeserialize::deserialize(buf)?;
    let is_mutable: bool = BorshDeserialize::deserialize(buf)?;
    let edition_nonce: Option<u8> = BorshDeserialize::deserialize(buf)?;

    let token_standard: Option<TokenStandard> = match BorshDeserialize::deserialize(buf) {
        Ok(token_standard) => Some(token_standard),
        Err(_) => {
            msg!("Corrupted metadata on token standard, setting to None");
            None
        }
    };

    let collection: Option<Collection> = match BorshDeserialize::deserialize(buf) {
        Ok(collection) => Some(collection),
        Err(_) => {
            msg!("Corrupted metadata on collection, setting to None");
            None
        }
    };

    let uses: Option<Uses> = match BorshDeserialize::deserialize(buf) {
        Ok(uses) => Some(uses),
        Err(_) => {
            msg!("Corrupted metadata on uses, setting to None");
            None
        }
    };

    /*
     Add deserizalition match statements for any future additions to the Metadata struct here.
    */

    let metadata = Metadata {
        key,
        update_authority,
        mint,
        data,
        primary_sale_happened,
        is_mutable,
        edition_nonce,
        token_standard,
        collection,
        uses,
    };

    Ok(metadata)
}
