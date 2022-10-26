use arrayref::{array_mut_ref, array_ref, mut_array_refs};
use borsh::BorshSerialize;
use mpl_utils::{
    assert_signer, create_or_allocate_account_raw,
    token::{get_mint_authority, get_mint_decimals, get_mint_freeze_authority, get_mint_supply},
};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, program_option::COption, pubkey::Pubkey,
};
use spl_token::{
    instruction::{set_authority, AuthorityType},
    state::Account,
};

use crate::{
    assertions::{
        assert_derivation, assert_initialized, assert_mint_authority_matches_mint, assert_owned_by,
        assert_token_program_matches_package,
        collection::assert_collection_update_is_valid,
        edition::{assert_edition_is_not_mint_authority, assert_edition_valid},
        metadata::assert_data_valid,
        uses::assert_valid_use,
    },
    deser::clean_write_metadata,
    error::MetadataError,
    state::{
        get_reservation_list, CollectionDetails, DataV2, Edition, EditionMarker, Key,
        MasterEditionV2, Metadata, TokenMetadataAccount, TokenStandard, Uses, EDITION,
        EDITION_MARKER_BIT_SIZE, MAX_EDITION_LEN, MAX_EDITION_MARKER_SIZE, MAX_MASTER_EDITION_LEN,
        MAX_METADATA_LEN, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH, PREFIX,
    },
};

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

pub fn extract_edition_number_from_deprecated_reservation_list(
    account: &AccountInfo,
    mint_authority_info: &AccountInfo,
) -> Result<u64, ProgramError> {
    let mut reservation_list = get_reservation_list(account)?;

    if let Some(supply_snapshot) = reservation_list.supply_snapshot() {
        let mut prev_total_offsets: u64 = 0;
        let mut offset: Option<u64> = None;
        let mut reservations = reservation_list.reservations();
        for i in 0..reservations.len() {
            let mut reservation = &mut reservations[i];

            if reservation.address == *mint_authority_info.key {
                offset = Some(
                    prev_total_offsets
                        .checked_add(reservation.spots_remaining)
                        .ok_or(MetadataError::NumericalOverflowError)?,
                );
                // You get your editions in reverse order but who cares, saves a byte
                reservation.spots_remaining = reservation
                    .spots_remaining
                    .checked_sub(1)
                    .ok_or(MetadataError::NumericalOverflowError)?;

                reservation_list.set_reservations(reservations)?;
                reservation_list.save(account)?;
                break;
            }

            if reservation.address == solana_program::system_program::id() {
                // This is an anchor point in the array...it means we reset our math to
                // this offset because we may be missing information in between this point and
                // the points before it.
                prev_total_offsets = reservation.total_spots;
            } else {
                prev_total_offsets = prev_total_offsets
                    .checked_add(reservation.total_spots)
                    .ok_or(MetadataError::NumericalOverflowError)?;
            }
        }

        match offset {
            Some(val) => Ok(supply_snapshot
                .checked_add(val)
                .ok_or(MetadataError::NumericalOverflowError)?),
            None => Err(MetadataError::AddressNotInReservation.into()),
        }
    } else {
        Err(MetadataError::ReservationNotSet.into())
    }
}

pub fn calculate_edition_number(
    mint_authority_info: &AccountInfo,
    reservation_list_info: Option<&AccountInfo>,
    edition_override: Option<u64>,
    me_supply: u64,
) -> Result<u64, ProgramError> {
    let edition = match reservation_list_info {
        Some(account) => {
            extract_edition_number_from_deprecated_reservation_list(account, mint_authority_info)?
        }
        None => {
            if let Some(edit) = edition_override {
                edit
            } else {
                me_supply
                    .checked_add(1)
                    .ok_or(MetadataError::NumericalOverflowError)?
            }
        }
    };

    Ok(edition)
}

fn get_max_supply_off_master_edition(
    master_edition_account_info: &AccountInfo,
) -> Result<Option<u64>, ProgramError> {
    let data = master_edition_account_info.try_borrow_data()?;
    // this is an option, 9 bytes, first is 0 means is none
    if data[9] == 0 {
        Ok(None)
    } else {
        let amount_data = array_ref![data, 10, 8];
        Ok(Some(u64::from_le_bytes(*amount_data)))
    }
}

pub fn get_supply_off_master_edition(
    master_edition_account_info: &AccountInfo,
) -> Result<u64, ProgramError> {
    let data = master_edition_account_info.try_borrow_data()?;
    // this is an option, 9 bytes, first is 0 means is none

    let amount_data = array_ref![data, 1, 8];
    Ok(u64::from_le_bytes(*amount_data))
}

pub fn calculate_supply_change<'a>(
    master_edition_account_info: &AccountInfo<'a>,
    reservation_list_info: Option<&AccountInfo<'a>>,
    edition_override: Option<u64>,
    current_supply: u64,
) -> ProgramResult {
    // Reservation lists are deprecated.
    if reservation_list_info.is_some() {
        return Err(MetadataError::ReservationListDeprecated.into());
    }

    // This function requires passing in the edition number.
    if edition_override.is_none() {
        return Err(MetadataError::EditionOverrideCannotBeZero.into());
    }

    let edition = edition_override.unwrap();

    if edition == 0 {
        return Err(MetadataError::EditionOverrideCannotBeZero.into());
    }

    let max_supply = get_max_supply_off_master_edition(master_edition_account_info)?;

    // Previously, the code used edition override to set the supply to the highest edition number minted,
    // instead of properly tracking the supply.
    // Now, we increment this by one if the edition number is less than the max supply.
    // This allows users to mint out missing edition numbers that are less than the supply, but
    // tracks the supply correctly for new Master Editions.
    let new_supply = if let Some(max_supply) = max_supply {
        // We should never be able to mint an edition number that is greater than the max supply.
        if edition > max_supply {
            return Err(MetadataError::EditionNumberGreaterThanMaxSupply.into());
        }

        // If the current supply is less than the max supply, then we can mint another addition so we increment the supply.
        if current_supply < max_supply {
            current_supply
                .checked_add(1)
                .ok_or(MetadataError::NumericalOverflowError)?
        }
        // If it's the same as max supply, we don't increment, but we return the supply
        // so we can mint out missing edition numbers in old editions that use the previous
        // edition override logic.
        //
        // The EditionMarker bitmask ensures we don't remint the same number twice.
        else {
            current_supply
        }
    }
    // With no max supply we can increment each time.
    else {
        current_supply
            .checked_add(1)
            .ok_or(MetadataError::NumericalOverflowError)?
    };

    // Doing old school serialization to protect CPU credits.
    let edition_data = &mut master_edition_account_info.data.borrow_mut();
    let output = array_mut_ref![edition_data, 0, MAX_MASTER_EDITION_LEN];

    let (_key, supply, _the_rest) = mut_array_refs![output, 1, 8, 273];
    *supply = new_supply.to_le_bytes();

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn mint_limited_edition<'a>(
    program_id: &'a Pubkey,
    master_metadata: Metadata,
    new_metadata_account_info: &'a AccountInfo<'a>,
    new_edition_account_info: &'a AccountInfo<'a>,
    master_edition_account_info: &'a AccountInfo<'a>,
    mint_info: &'a AccountInfo<'a>,
    mint_authority_info: &'a AccountInfo<'a>,
    payer_account_info: &'a AccountInfo<'a>,
    update_authority_info: &'a AccountInfo<'a>,
    token_program_account_info: &'a AccountInfo<'a>,
    system_account_info: &'a AccountInfo<'a>,
    // Only present with MasterEditionV1 calls, if present, use edition based off address in res list,
    // otherwise, pull off the top
    reservation_list_info: Option<&'a AccountInfo<'a>>,
    // Only present with MasterEditionV2 calls, if present, means
    // directing to a specific version, otherwise just pull off the top
    edition_override: Option<u64>,
) -> ProgramResult {
    let me_supply = get_supply_off_master_edition(master_edition_account_info)?;
    let mint_authority = get_mint_authority(mint_info)?;
    let mint_supply = get_mint_supply(mint_info)?;
    assert_mint_authority_matches_mint(&mint_authority, mint_authority_info)?;

    assert_edition_valid(
        program_id,
        &master_metadata.mint,
        master_edition_account_info,
    )?;

    let edition_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ];
    let (edition_key, bump_seed) = Pubkey::find_program_address(edition_seeds, program_id);
    if edition_key != *new_edition_account_info.key {
        return Err(MetadataError::InvalidEditionKey.into());
    }

    if reservation_list_info.is_some() && edition_override.is_some() {
        return Err(MetadataError::InvalidOperation.into());
    }
    calculate_supply_change(
        master_edition_account_info,
        reservation_list_info,
        edition_override,
        me_supply,
    )?;

    if mint_supply != 1 {
        return Err(MetadataError::EditionsMustHaveExactlyOneToken.into());
    }
    let master_data = master_metadata.data;
    // bundle data into v2
    let data_v2 = DataV2 {
        name: master_data.name,
        symbol: master_data.symbol,
        uri: master_data.uri,
        seller_fee_basis_points: master_data.seller_fee_basis_points,
        creators: master_data.creators,
        collection: master_metadata.collection,
        uses: master_metadata.uses.map(|u| Uses {
            use_method: u.use_method,
            remaining: u.total, // reset remaining uses per edition for extra fun
            total: u.total,
        }),
    };
    // create the metadata the normal way, except `allow_direct_creator_writes` is set to true
    // because we are directly copying from the Master Edition metadata.

    process_create_metadata_accounts_logic(
        program_id,
        CreateMetadataAccountsLogicArgs {
            metadata_account_info: new_metadata_account_info,
            mint_info,
            mint_authority_info,
            payer_account_info,
            update_authority_info,
            system_account_info,
        },
        data_v2,
        true,
        false,
        true,
        true,
        None, // Not a collection parent
    )?;
    let edition_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
        &[bump_seed],
    ];

    create_or_allocate_account_raw(
        *program_id,
        new_edition_account_info,
        system_account_info,
        payer_account_info,
        MAX_EDITION_LEN,
        edition_authority_seeds,
    )?;

    // Doing old school serialization to protect CPU credits.
    let edition_data = &mut new_edition_account_info.data.borrow_mut();
    let output = array_mut_ref![edition_data, 0, MAX_EDITION_LEN];

    let (key, parent, edition, _padding) = mut_array_refs![output, 1, 32, 8, 200];

    *key = [Key::EditionV1 as u8];
    parent.copy_from_slice(master_edition_account_info.key.as_ref());

    *edition = calculate_edition_number(
        mint_authority_info,
        reservation_list_info,
        edition_override,
        me_supply,
    )?
    .to_le_bytes();

    // Now make sure this mint can never be used by anybody else.
    transfer_mint_authority(
        &edition_key,
        new_edition_account_info,
        mint_info,
        mint_authority_info,
        token_program_account_info,
    )?;

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

pub struct CreateMetadataAccountsLogicArgs<'a> {
    pub metadata_account_info: &'a AccountInfo<'a>,
    pub mint_info: &'a AccountInfo<'a>,
    pub mint_authority_info: &'a AccountInfo<'a>,
    pub payer_account_info: &'a AccountInfo<'a>,
    pub update_authority_info: &'a AccountInfo<'a>,
    pub system_account_info: &'a AccountInfo<'a>,
}

// This equals the program address of the metadata program:
// AqH29mZfQFgRpfwaPoTMWSKJ5kqauoc1FwVBRksZyQrt
// IMPORTANT NOTE
// This allows the upgrade authority of the Token Metadata program to create metadata for SPL tokens.
// This only allows the upgrade authority to do create general metadata for the SPL token, it does not
// allow the upgrade authority to add or change creators.
pub const SEED_AUTHORITY: Pubkey = Pubkey::new_from_array([
    0x92, 0x17, 0x2c, 0xc4, 0x72, 0x5d, 0xc0, 0x41, 0xf9, 0xdd, 0x8c, 0x51, 0x52, 0x60, 0x04, 0x26,
    0x00, 0x93, 0xa3, 0x0b, 0x02, 0x73, 0xdc, 0xfa, 0x74, 0x92, 0x17, 0xfc, 0x94, 0xa2, 0x40, 0x49,
]);

// This equals the program address of the Bubblegum program:
// "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY"
// This allows the Bubblegum program to add verified creators since they were verified as part of
// the Bubblegum program.
pub const BUBBLEGUM_PROGRAM_ADDRESS: Pubkey = Pubkey::new_from_array([
    0x98, 0x8b, 0x80, 0xeb, 0x79, 0x35, 0x28, 0x69, 0xb2, 0x24, 0x74, 0x5f, 0x59, 0xdd, 0xbf, 0x8a,
    0x26, 0x58, 0xca, 0x13, 0xdc, 0x68, 0x81, 0x21, 0x26, 0x35, 0x1c, 0xae, 0x07, 0xc1, 0xa5, 0xa5,
]);
// This flag activates certain program authority features of the Bubblegum program.
pub const BUBBLEGUM_ACTIVATED: bool = false;

/// Create a new account instruction
pub fn process_create_metadata_accounts_logic(
    program_id: &Pubkey,
    accounts: CreateMetadataAccountsLogicArgs,
    data: DataV2,
    allow_direct_creator_writes: bool,
    mut is_mutable: bool,
    is_edition: bool,
    add_token_standard: bool,
    collection_details: Option<CollectionDetails>,
) -> ProgramResult {
    let CreateMetadataAccountsLogicArgs {
        metadata_account_info,
        mint_info,
        mint_authority_info,
        payer_account_info,
        update_authority_info,
        system_account_info,
    } = accounts;

    let mut update_authority_key = *update_authority_info.key;
    let existing_mint_authority = get_mint_authority(mint_info)?;
    // IMPORTANT NOTE
    // This allows the Metaplex Foundation to Create but not update metadata for SPL tokens that have not populated their metadata.
    assert_mint_authority_matches_mint(&existing_mint_authority, mint_authority_info).or_else(
        |e| {
            // Allow seeding by the authority seed populator
            if mint_authority_info.key == &SEED_AUTHORITY && mint_authority_info.is_signer {
                // When metadata is seeded, the mint authority should be able to change it
                if let COption::Some(auth) = existing_mint_authority {
                    update_authority_key = auth;
                    is_mutable = true;
                }
                Ok(())
            } else {
                Err(e)
            }
        },
    )?;
    assert_owned_by(mint_info, &spl_token::id())?;

    let metadata_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (metadata_key, metadata_bump_seed) =
        Pubkey::find_program_address(metadata_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[metadata_bump_seed],
    ];

    if metadata_account_info.key != &metadata_key {
        return Err(MetadataError::InvalidMetadataKey.into());
    }

    create_or_allocate_account_raw(
        *program_id,
        metadata_account_info,
        system_account_info,
        payer_account_info,
        MAX_METADATA_LEN,
        metadata_authority_signer_seeds,
    )?;

    let mut metadata = Metadata::from_account_info(metadata_account_info)?;
    let compatible_data = data.to_v1();

    // This allows the Bubblegum program to create metadata with verified creators since they were
    // verified already by the Bubblegum program.
    let allow_direct_creator_writes = if BUBBLEGUM_ACTIVATED
        && mint_authority_info.owner == &BUBBLEGUM_PROGRAM_ADDRESS
        && mint_authority_info.is_signer
    {
        true
    } else {
        allow_direct_creator_writes
    };

    assert_data_valid(
        &compatible_data,
        &update_authority_key,
        &metadata,
        allow_direct_creator_writes,
        update_authority_info.is_signer,
    )?;

    let mint_decimals = get_mint_decimals(mint_info)?;

    metadata.mint = *mint_info.key;
    metadata.key = Key::MetadataV1;
    metadata.data = data.to_v1();
    metadata.is_mutable = is_mutable;
    metadata.update_authority = update_authority_key;

    assert_valid_use(&data.uses, &None)?;
    metadata.uses = data.uses;

    assert_collection_update_is_valid(is_edition, &None, &data.collection)?;
    metadata.collection = data.collection;

    // We want to create new collections with a size of zero but we use the
    // collection details enum for forward compatibility.
    if let Some(details) = collection_details {
        match details {
            CollectionDetails::V1 { size: _size } => {
                metadata.collection_details = Some(CollectionDetails::V1 { size: 0 });
            }
        }
    } else {
        metadata.collection_details = None;
    }

    if add_token_standard {
        let token_standard = if is_edition {
            TokenStandard::NonFungibleEdition
        } else if mint_decimals == 0 {
            TokenStandard::FungibleAsset
        } else {
            TokenStandard::Fungible
        };
        metadata.token_standard = Some(token_standard);
    } else {
        metadata.token_standard = None;
    }
    puff_out_data_fields(&mut metadata);

    let edition_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        metadata.mint.as_ref(),
        EDITION.as_bytes(),
    ];
    let (_, edition_bump_seed) = Pubkey::find_program_address(edition_seeds, program_id);
    metadata.edition_nonce = Some(edition_bump_seed);
    metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;

    Ok(())
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

pub struct MintNewEditionFromMasterEditionViaTokenLogicArgs<'a> {
    pub new_metadata_account_info: &'a AccountInfo<'a>,
    pub new_edition_account_info: &'a AccountInfo<'a>,
    pub master_edition_account_info: &'a AccountInfo<'a>,
    pub mint_info: &'a AccountInfo<'a>,
    pub edition_marker_info: &'a AccountInfo<'a>,
    pub mint_authority_info: &'a AccountInfo<'a>,
    pub payer_account_info: &'a AccountInfo<'a>,
    pub owner_account_info: &'a AccountInfo<'a>,
    pub token_account_info: &'a AccountInfo<'a>,
    pub update_authority_info: &'a AccountInfo<'a>,
    pub master_metadata_account_info: &'a AccountInfo<'a>,
    pub token_program_account_info: &'a AccountInfo<'a>,
    pub system_account_info: &'a AccountInfo<'a>,
}

pub fn process_mint_new_edition_from_master_edition_via_token_logic<'a>(
    program_id: &'a Pubkey,
    accounts: MintNewEditionFromMasterEditionViaTokenLogicArgs<'a>,
    edition: u64,
    ignore_owner_signer: bool,
) -> ProgramResult {
    let MintNewEditionFromMasterEditionViaTokenLogicArgs {
        new_metadata_account_info,
        new_edition_account_info,
        master_edition_account_info,
        mint_info,
        edition_marker_info,
        mint_authority_info,
        payer_account_info,
        owner_account_info,
        token_account_info,
        update_authority_info,
        master_metadata_account_info,
        token_program_account_info,
        system_account_info,
    } = accounts;

    assert_token_program_matches_package(token_program_account_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    assert_owned_by(token_account_info, &spl_token::id())?;
    assert_owned_by(master_edition_account_info, program_id)?;
    assert_owned_by(master_metadata_account_info, program_id)?;

    let master_metadata = Metadata::from_account_info(master_metadata_account_info)?;
    let token_account: Account = assert_initialized(token_account_info)?;

    if !ignore_owner_signer {
        assert_signer(owner_account_info)?;

        if token_account.owner != *owner_account_info.key {
            return Err(MetadataError::InvalidOwner.into());
        }
    }

    if token_account.mint != master_metadata.mint {
        return Err(MetadataError::TokenAccountMintMismatchV2.into());
    }

    if token_account.amount < 1 {
        return Err(MetadataError::NotEnoughTokens.into());
    }

    if !new_metadata_account_info.data_is_empty() {
        return Err(MetadataError::AlreadyInitialized.into());
    }

    if !new_edition_account_info.data_is_empty() {
        return Err(MetadataError::AlreadyInitialized.into());
    }

    let edition_number = edition.checked_div(EDITION_MARKER_BIT_SIZE).unwrap();
    let as_string = edition_number.to_string();

    let bump = assert_derivation(
        program_id,
        edition_marker_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            master_metadata.mint.as_ref(),
            EDITION.as_bytes(),
            as_string.as_bytes(),
        ],
    )?;

    if edition_marker_info.data_is_empty() {
        let seeds = &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            master_metadata.mint.as_ref(),
            EDITION.as_bytes(),
            as_string.as_bytes(),
            &[bump],
        ];

        create_or_allocate_account_raw(
            *program_id,
            edition_marker_info,
            system_account_info,
            payer_account_info,
            MAX_EDITION_MARKER_SIZE,
            seeds,
        )?;
    }

    let mut edition_marker = EditionMarker::from_account_info(edition_marker_info)?;
    edition_marker.key = Key::EditionMarker;
    if edition_marker.edition_taken(edition)? {
        return Err(MetadataError::AlreadyInitialized.into());
    } else {
        edition_marker.insert_edition(edition)?
    }
    edition_marker.serialize(&mut *edition_marker_info.data.borrow_mut())?;

    mint_limited_edition(
        program_id,
        master_metadata,
        new_metadata_account_info,
        new_edition_account_info,
        master_edition_account_info,
        mint_info,
        mint_authority_info,
        payer_account_info,
        update_authority_info,
        token_program_account_info,
        system_account_info,
        None,
        Some(edition),
    )?;
    Ok(())
}

pub fn increment_collection_size(
    metadata: &mut Metadata,
    metadata_info: &AccountInfo,
) -> ProgramResult {
    if let Some(ref details) = metadata.collection_details {
        match details {
            CollectionDetails::V1 { size } => {
                metadata.collection_details = Some(CollectionDetails::V1 {
                    size: size
                        .checked_add(1)
                        .ok_or(MetadataError::NumericalOverflowError)?,
                });
                msg!("Clean writing collection parent metadata");
                clean_write_metadata(metadata, metadata_info)?;
                Ok(())
            }
        }
    } else {
        msg!("No collection details found. Cannot increment collection size.");
        Err(MetadataError::UnsizedCollection.into())
    }
}

pub fn decrement_collection_size(
    metadata: &mut Metadata,
    metadata_info: &AccountInfo,
) -> ProgramResult {
    if let Some(ref details) = metadata.collection_details {
        match details {
            CollectionDetails::V1 { size } => {
                metadata.collection_details = Some(CollectionDetails::V1 {
                    size: size
                        .checked_sub(1)
                        .ok_or(MetadataError::NumericalOverflowError)?,
                });
                clean_write_metadata(metadata, metadata_info)?;
                Ok(())
            }
        }
    } else {
        msg!("No collection details found. Cannot decrement collection size.");
        Err(MetadataError::UnsizedCollection.into())
    }
}

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
