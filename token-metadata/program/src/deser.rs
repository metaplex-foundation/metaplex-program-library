use crate::state::{Collection, CollectionDetails, Data, Key, Metadata, TokenStandard, Uses};
use borsh::{maybestd::io::Error as BorshError, BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

// Custom deserialization function to handle NFTs with corrupted data.
// This function is used in a custom deserialization implementation for the
// `Metadata` struct, so should never have `msg` macros used in it as it may be used client side
// either in tests or client code.
//
// It does not check `Key` type or account length and should only be used through the custom functions
// `from_account_info` and `deserialize` implemented on the Metadata struct.
pub fn meta_deser_unchecked(buf: &mut &[u8]) -> Result<Metadata, BorshError> {
    // Metadata corruption shouldn't appear until after edition_nonce.
    let key: Key = BorshDeserialize::deserialize(buf)?;
    let update_authority: Pubkey = BorshDeserialize::deserialize(buf)?;
    let mint: Pubkey = BorshDeserialize::deserialize(buf)?;
    let data: Data = BorshDeserialize::deserialize(buf)?;
    let primary_sale_happened: bool = BorshDeserialize::deserialize(buf)?;
    let is_mutable: bool = BorshDeserialize::deserialize(buf)?;
    let edition_nonce: Option<u8> = BorshDeserialize::deserialize(buf)?;

    // V1.2
    let token_standard_res: Result<Option<TokenStandard>, BorshError> =
        BorshDeserialize::deserialize(buf);
    let collection_res: Result<Option<Collection>, BorshError> = BorshDeserialize::deserialize(buf);
    let uses_res: Result<Option<Uses>, BorshError> = BorshDeserialize::deserialize(buf);

    // V1.3
    let collection_details_res: Result<Option<CollectionDetails>, BorshError> =
        BorshDeserialize::deserialize(buf);

    /* We can have accidentally valid, but corrupted data, particularly on the Collection struct,
    so to increase probability of catching errors If any of these deserializations fail, set all values to None.
    */
    let (token_standard, collection, uses) = match (token_standard_res, collection_res, uses_res) {
        (Ok(token_standard_res), Ok(collection_res), Ok(uses_res)) => {
            (token_standard_res, collection_res, uses_res)
        }
        _ => (None, None, None),
    };

    // Handle v1.3 separately
    let collection_details = match collection_details_res {
        Ok(details) => details,
        Err(_) => {
            println!("no collection details found");
            None
        }
    };

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
        collection_details,
    };

    Ok(metadata)
}

pub fn clean_write_metadata(
    metadata: &mut Metadata,
    metadata_account_info: &AccountInfo,
) -> ProgramResult {
    // Clear all data to ensure it is serialized cleanly with no trailing data due to creators array resizing.
    let mut metadata_account_info_data = metadata_account_info.try_borrow_mut_data()?;
    metadata_account_info_data[0..].fill(0);

    metadata.serialize(&mut *metadata_account_info_data)?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use solana_program::pubkey;

    use super::*;
    pub use crate::{state::Creator, utils::puff_out_data_fields};

    // Pesky Penguins #8060 (NOOT!)
    // Corrupted data that can't be deserialized with the standard BoshDeserialization implementation.
    pub fn pesky_data() -> &'static [u8] {
        &[
            4, 12, 25, 250, 103, 242, 3, 129, 143, 173, 110, 204, 157, 11, 1, 247, 211, 138, 199,
            219, 79, 142, 183, 195, 96, 206, 63, 208, 102, 152, 127, 62, 43, 181, 253, 142, 126,
            95, 96, 46, 202, 26, 76, 133, 228, 219, 191, 64, 186, 139, 115, 88, 216, 76, 125, 144,
            12, 216, 198, 54, 196, 128, 102, 191, 96, 32, 0, 0, 0, 80, 101, 115, 107, 121, 32, 80,
            101, 110, 103, 117, 105, 110, 115, 32, 35, 56, 48, 54, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 10, 0, 0, 0, 78, 79, 79, 84, 0, 0, 0, 0, 0, 0, 200, 0, 0, 0, 104, 116, 116,
            112, 115, 58, 47, 47, 97, 114, 119, 101, 97, 118, 101, 46, 110, 101, 116, 47, 72, 122,
            79, 110, 102, 78, 77, 87, 81, 66, 72, 84, 57, 118, 48, 68, 87, 56, 69, 114, 57, 89, 70,
            119, 100, 105, 71, 74, 88, 52, 45, 117, 75, 57, 82, 83, 89, 65, 82, 56, 102, 120, 69,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 244, 1, 1, 3, 0, 0, 0,
            135, 35, 134, 27, 83, 153, 173, 73, 166, 213, 73, 13, 254, 1, 156, 113, 34, 24, 205,
            42, 233, 242, 137, 173, 173, 195, 214, 108, 110, 42, 89, 229, 1, 0, 12, 25, 250, 103,
            242, 3, 129, 143, 173, 110, 204, 157, 11, 1, 247, 211, 138, 199, 219, 79, 142, 183,
            195, 96, 206, 63, 208, 102, 152, 127, 62, 43, 1, 40, 12, 63, 245, 233, 144, 127, 205,
            69, 77, 225, 56, 60, 107, 184, 84, 240, 194, 136, 55, 121, 217, 128, 246, 223, 140, 64,
            40, 122, 145, 17, 203, 60, 0, 60, 1, 1, 1, 255, 149, 248, 123, 137, 230, 77, 203, 8,
            124, 145, 63, 132, 220, 224, 64, 60, 253, 17, 33, 18, 81, 80, 186, 15, 248, 247, 249,
            243, 1, 20, 26, 244, 47, 94, 35, 232, 64, 68, 124, 40, 100, 36, 93, 190, 82, 38, 36,
            149, 248, 56, 72, 95, 68, 50, 157, 1, 155, 95, 113, 49, 247, 176, 1, 20, 1, 1, 1, 255,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ]
    }

    pub fn expected_pesky_metadata() -> Metadata {
        let creators = vec![
            Creator {
                address: pubkey!("A6XTVFiwGVsG6b6LsvQTGnV5LH3Pfa3qW3TGz8RjToLp"),
                verified: true,
                share: 0,
            },
            Creator {
                address: pubkey!("pEsKYABNARLiDFYrjbjHDieD5h6gHrvYf9Vru62NX9k"),
                verified: true,
                share: 40,
            },
            Creator {
                address: pubkey!("ppTeamTpw1cbC8ybJpppbnoL7xXD9froJNFb5uvoPvb"),
                verified: false,
                share: 60,
            },
        ];

        let data = Data {
            name: "Pesky Penguins #8060".to_string(),
            symbol: "NOOT".to_string(),
            uri: "https://arweave.net/HzOnfNMWQBHT9v0DW8Er9YFwdiGJX4-uK9RSYAR8fxE".to_string(),
            seller_fee_basis_points: 500,
            creators: Some(creators),
        };

        let mut metadata = Metadata {
            key: Key::MetadataV1,
            update_authority: pubkey!("pEsKYABNARLiDFYrjbjHDieD5h6gHrvYf9Vru62NX9k"),
            mint: pubkey!("DFR3KjTso6PFCyUtq48a2aPZQpMMoaFgtbdxtaLxF2TR"),
            data,
            primary_sale_happened: true,
            is_mutable: true,
            edition_nonce: Some(255),
            token_standard: None,
            collection: None,
            uses: None,
            collection_details: None,
        };

        puff_out_data_fields(&mut metadata);

        metadata
    }

    #[test]
    fn deserialize_corrupted_metadata() {
        let mut buf = pesky_data();
        let metadata = meta_deser_unchecked(&mut buf).unwrap();
        let expected_metadata = expected_pesky_metadata();

        assert_eq!(metadata, expected_metadata);
    }
}
