pub(crate) mod collection;
pub(crate) mod compression;
pub(crate) mod master_edition;
pub(crate) mod metadata;

pub use crate::assertions::{
    assert_delegated_tokens, assert_derivation, assert_freeze_authority_matches_mint,
    assert_initialized, assert_mint_authority_matches_mint, assert_owned_by, assert_rent_exempt,
    assert_token_program_matches_package,
    edition::{
        assert_edition_is_not_mint_authority, assert_edition_valid, assert_supply_invariance,
    },
    metadata::{
        assert_currently_holding, assert_data_valid, assert_update_authority_is_correct,
        assert_verified_member_of_collection,
    },
};
pub use collection::*;
pub use compression::*;
pub use master_edition::*;
pub use metadata::*;
pub use mpl_utils::{
    assert_signer, close_account_raw, create_or_allocate_account_raw,
    resize_or_reallocate_account_raw,
    token::{
        get_mint_authority, get_mint_decimals, get_mint_freeze_authority, get_mint_supply,
        get_owner_from_token_account, spl_token_burn, spl_token_close, spl_token_mint_to,
    },
};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
};
use spl_token::instruction::{set_authority, AuthorityType};

use crate::{
    error::MetadataError,
    state::{
        Edition, Key, MasterEditionV2, Metadata, TokenMetadataAccount, TokenStandard,
        MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH,
    },
};

pub fn check_token_standard(
    mint_info: &AccountInfo,
    edition_account_info: Option<&AccountInfo>,
) -> Result<TokenStandard, ProgramError> {
    let mint_decimals = get_mint_decimals(mint_info)?;
    let mint_supply = get_mint_supply(mint_info)?;

    match edition_account_info {
        Some(edition) => {
            if is_master_edition(edition, mint_decimals, mint_supply) {
                Ok(TokenStandard::NonFungible)
            } else if is_print_edition(edition, mint_decimals, mint_supply) {
                Ok(TokenStandard::NonFungibleEdition)
            } else {
                Err(MetadataError::CouldNotDetermineTokenStandard.into())
            }
        }
        None => {
            assert_edition_is_not_mint_authority(mint_info)?;
            if mint_decimals == 0 {
                Ok(TokenStandard::FungibleAsset)
            } else {
                Ok(TokenStandard::Fungible)
            }
        }
    }
}

pub fn is_master_edition(
    edition_account_info: &AccountInfo,
    mint_decimals: u8,
    mint_supply: u64,
) -> bool {
    let is_correct_type = MasterEditionV2::from_account_info(edition_account_info).is_ok();

    is_correct_type && mint_decimals == 0 && mint_supply == 1
}

pub fn is_print_edition(
    edition_account_info: &AccountInfo,
    mint_decimals: u8,
    mint_supply: u64,
) -> bool {
    let is_correct_type = Edition::from_account_info(edition_account_info).is_ok();

    is_correct_type && mint_decimals == 0 && mint_supply == 1
}

/// Strings need to be appended with `\0`s in order to have a deterministic length.
/// This supports the `memcmp` filter  on get program account calls.
/// NOTE: it is assumed that the metadata fields are never larger than the respective MAX_LENGTH
pub fn puff_out_data_fields(metadata: &mut Metadata) {
    metadata.data.name = puffed_out_string(&metadata.data.name, MAX_NAME_LENGTH);
    metadata.data.symbol = puffed_out_string(&metadata.data.symbol, MAX_SYMBOL_LENGTH);
    metadata.data.uri = puffed_out_string(&metadata.data.uri, MAX_URI_LENGTH);
}

/// Pads the string to the desired size with `0u8`s.
/// NOTE: it is assumed that the string's size is never larger than the given size.
pub fn puffed_out_string(s: &str, size: usize) -> String {
    let mut array_of_zeroes = vec![];
    let puff_amount = size - s.len();
    while array_of_zeroes.len() < puff_amount {
        array_of_zeroes.push(0u8);
    }
    s.to_owned() + std::str::from_utf8(&array_of_zeroes).unwrap()
}

pub fn transfer_mint_authority<'a>(
    edition_key: &Pubkey,
    edition_account_info: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    mint_authority_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    msg!("Setting mint authority");
    let accounts = &[
        mint_authority_info.clone(),
        mint_info.clone(),
        token_program_info.clone(),
        edition_account_info.clone(),
    ];
    invoke_signed(
        &set_authority(
            token_program_info.key,
            mint_info.key,
            Some(edition_key),
            AuthorityType::MintTokens,
            mint_authority_info.key,
            &[mint_authority_info.key],
        )
        .unwrap(),
        accounts,
        &[],
    )?;
    msg!("Setting freeze authority");
    let freeze_authority = get_mint_freeze_authority(mint_info)?;
    if freeze_authority.is_some() {
        invoke_signed(
            &set_authority(
                token_program_info.key,
                mint_info.key,
                Some(edition_key),
                AuthorityType::FreezeAccount,
                mint_authority_info.key,
                &[mint_authority_info.key],
            )
            .unwrap(),
            accounts,
            &[],
        )?;
        msg!("Finished setting freeze authority");
    } else {
        return Err(MetadataError::NoFreezeAuthoritySet.into());
    }

    Ok(())
}

pub fn try_from_slice_checked<T: TokenMetadataAccount>(
    data: &[u8],
    data_type: Key,
    data_size: usize,
) -> Result<T, ProgramError> {
    if !T::is_correct_account_type(data, data_type, data_size) {
        return Err(MetadataError::DataTypeMismatch.into());
    }

    let result: T = try_from_slice_unchecked(data)?;

    Ok(result)
}

/// Pads the string to the desired size with `0u8`s.
/// NOTE: it is assumed that the string's size is never larger than the given size.
pub fn zero_account(s: &str, size: usize) -> String {
    let mut array_of_zeroes = vec![];
    let puff_amount = size - s.len();
    while array_of_zeroes.len() < puff_amount {
        array_of_zeroes.push(0u8);
    }
    s.to_owned() + std::str::from_utf8(&array_of_zeroes).unwrap()
}

#[cfg(test)]
mod tests {
    pub use solana_program::pubkey::Pubkey;

    use crate::{
        state::MAX_METADATA_LEN,
        utils::{
            metadata::tests::{expected_pesky_metadata, pesky_data},
            try_from_slice_checked,
        },
    };
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
