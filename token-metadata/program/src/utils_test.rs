#![cfg(test)]

mod puff_out_test {
    pub use solana_program::pubkey::Pubkey;

    pub use crate::{
        state::{Data, Key, Metadata},
        utils::{puff_out_data_fields, puffed_out_string},
    };

    #[test]
    fn puffed_out_string_test() {
        let cases = &[
            ("hello", 5, "hello"),
            ("hello", 6, "hello\u{0}"),
            ("hello", 10, "hello\u{0}\u{0}\u{0}\u{0}\u{0}"),
            (
                "hello",
                20,
                "hello\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}",
            ),
        ];
        for (s, size, puffed_out) in cases {
            let result = puffed_out_string(s, *size);
            assert_eq!(result, puffed_out.to_string(), "s: {:?}, size: {}", s, size,);
        }
    }

    #[test]
    fn puffed_out_metadata_test() {
        let mut metadata = Metadata {
            key: Key::MetadataV1,
            update_authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            data: Data {
                name: "Garfield".to_string(),
                symbol: "GARF".to_string(),
                uri: "https://garfiel.de".to_string(),
                seller_fee_basis_points: 0,
                creators: None,
            },
            primary_sale_happened: false,
            is_mutable: false,
            edition_nonce: None,
            collection: None,
            uses: None,
            token_standard: None,
            collection_details: None,
        };

        puff_out_data_fields(&mut metadata);

        let Data {
            name,
            symbol,
            uri,
            seller_fee_basis_points,
            creators,
        } = metadata.data;

        assert_eq!(name.as_str(), "Garfield\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}");
        assert_eq!(symbol.as_str(), "GARF\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}");
        assert_eq!(uri.as_str(), "https://garfiel.de\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}");
        assert_eq!(seller_fee_basis_points, 0);
        assert_eq!(creators, None);
    }
}

mod try_from_slice_checked {
    use crate::{
        deser::tests::{expected_pesky_metadata, pesky_data},
        state::{Key, Metadata, MAX_METADATA_LEN},
        utils::try_from_slice_checked,
    };

    #[test]
    fn deserialize_corrupted_metadata_ok() {
        // This should be able to deserialize the corrupted metadata account successfully due to the custom BorshDeserilization
        // implementation for the Metadata struct.
        let expected_metadata = expected_pesky_metadata();
        let corrupted_data = pesky_data();

        let metadata: Metadata =
            try_from_slice_checked(corrupted_data, Key::MetadataV1, MAX_METADATA_LEN).unwrap();

        assert_eq!(metadata, expected_metadata);
    }
}
