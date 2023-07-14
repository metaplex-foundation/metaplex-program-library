use arrayref::{array_mut_ref, array_ref, mut_array_refs};
use borsh::BorshSerialize;
use mpl_utils::{
    assert_signer, create_or_allocate_account_raw,
    token::{get_mint_authority, get_mint_supply},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};
use spl_token::state::{Account, Mint};

use super::*;
use crate::{
    assertions::{
        assert_derivation, assert_initialized, assert_mint_authority_matches_mint, assert_owned_by,
        assert_token_program_matches_package, edition::assert_edition_valid,
        metadata::assert_update_authority_is_correct,
    },
    error::MetadataError,
    pda::MARKER,
    state::{
        get_reservation_list, DataV2, EditionMarker, EditionMarkerV2, Key, MasterEdition, Metadata,
        TokenMetadataAccount, Uses, EDITION, EDITION_MARKER_BIT_SIZE, MAX_EDITION_LEN,
        MAX_EDITION_MARKER_SIZE, MAX_MASTER_EDITION_LEN, PREFIX,
    },
};

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
    assert_owned_by(mint_info, &spl_token::ID)?;
    assert_owned_by(token_account_info, &spl_token::ID)?;
    assert_owned_by(master_edition_account_info, program_id)?;
    assert_owned_by(master_metadata_account_info, program_id)?;
    assert_signer(payer_account_info)?;

    if system_account_info.key != &system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    let master_metadata = Metadata::from_account_info(master_metadata_account_info)?;
    let token_account: Account = assert_initialized(token_account_info)?;

    assert_signer(owner_account_info)?;

    if token_account.owner != *owner_account_info.key {
        return Err(MetadataError::InvalidOwner.into());
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

    // Check that the edition we're printing from actually is a master edition.
    // We're not passing in the master edition mint so we can't fetch the actual supply and decimals
    // but we can safely assume that the account was only created if those checks passed.
    if !is_master_edition(master_edition_account_info, 0, 1) {
        return Err(MetadataError::InvalidMasterEdition.into());
    };

    let token_standard = master_metadata
        .token_standard
        .unwrap_or(TokenStandard::NonFungible);
    match token_standard {
        TokenStandard::NonFungible => {
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
        }
        TokenStandard::ProgrammableNonFungible => {
            let bump = assert_derivation(
                program_id,
                edition_marker_info,
                &[
                    PREFIX.as_bytes(),
                    program_id.as_ref(),
                    master_metadata.mint.as_ref(),
                    EDITION.as_bytes(),
                    MARKER.as_bytes(),
                ],
            )?;

            let mut edition_marker = if edition_marker_info.data_is_empty() {
                let seeds = &[
                    PREFIX.as_bytes(),
                    program_id.as_ref(),
                    master_metadata.mint.as_ref(),
                    EDITION.as_bytes(),
                    MARKER.as_bytes(),
                    &[bump],
                ];

                let marker = EditionMarkerV2::default();
                let serialized_data = marker.try_to_vec()?;

                create_or_allocate_account_raw(
                    *program_id,
                    edition_marker_info,
                    system_account_info,
                    payer_account_info,
                    serialized_data.len(),
                    seeds,
                )?;

                marker
            } else {
                EditionMarkerV2::from_account_info(edition_marker_info)?
            };

            edition_marker.key = Key::EditionMarkerV2;
            if edition_marker.edition_taken(edition)? {
                return Err(MetadataError::AlreadyInitialized.into());
            } else {
                edition_marker.insert_edition(edition)?
            }
            edition_marker.save(edition_marker_info, payer_account_info, system_account_info)?;
        }
        _ => return Err(MetadataError::InvalidTokenStandard.into()),
    };

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

            if reservation.address == solana_program::system_program::ID {
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
    let mint_decimals = get_mint_decimals(mint_info)?;
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
    if mint_decimals != 0 {
        return Err(MetadataError::EditionMintDecimalsShouldBeZero.into());
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

    // I hate this but can't think of a better way until we refactor setting
    // token_standard everywhere.
    let token_standard_override = match master_metadata.token_standard {
        Some(TokenStandard::NonFungible) => Some(TokenStandard::NonFungibleEdition),
        Some(TokenStandard::ProgrammableNonFungible) => {
            Some(TokenStandard::ProgrammableNonFungibleEdition)
        }
        _ => None,
    };

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
        token_standard_override,
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

/// Creates a new master edition account for the specified `edition_account_info` and
/// `mint_info`. Master editions only exist for non-fungible assets, therefore the supply
/// of the mint must thei either 0 or 1; any value higher than that will generate an
/// error.
///
/// After a master edition is created, it becomes the mint authority of the mint account.
pub fn create_master_edition<'a>(
    program_id: &Pubkey,
    edition_account_info: &'a AccountInfo<'a>,
    mint_info: &'a AccountInfo<'a>,
    update_authority_info: &'a AccountInfo<'a>,
    mint_authority_info: &'a AccountInfo<'a>,
    payer_account_info: &'a AccountInfo<'a>,
    metadata_account_info: &'a AccountInfo<'a>,
    token_program_info: &'a AccountInfo<'a>,
    system_account_info: &'a AccountInfo<'a>,
    max_supply: Option<u64>,
) -> ProgramResult {
    let metadata = Metadata::from_account_info(metadata_account_info)?;
    let mint: Mint = assert_initialized(mint_info)?;

    let bump_seed = assert_derivation(
        program_id,
        edition_account_info,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            mint_info.key.as_ref(),
            EDITION.as_bytes(),
        ],
    )?;

    assert_token_program_matches_package(token_program_info)?;
    assert_mint_authority_matches_mint(&mint.mint_authority, mint_authority_info)?;
    assert_owned_by(metadata_account_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::ID)?;

    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if mint.decimals != 0 {
        return Err(MetadataError::EditionMintDecimalsShouldBeZero.into());
    }

    assert_update_authority_is_correct(&metadata, update_authority_info)?;

    if mint.supply > 1 {
        return Err(MetadataError::EditionsMustHaveExactlyOneToken.into());
    }

    let edition_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
        &[bump_seed],
    ];

    create_or_allocate_account_raw(
        *program_id,
        edition_account_info,
        system_account_info,
        payer_account_info,
        MAX_MASTER_EDITION_LEN,
        edition_authority_seeds,
    )?;

    let mut edition = MasterEditionV2::from_account_info(edition_account_info)?;

    edition.key = Key::MasterEditionV2;
    edition.supply = 0;
    edition.max_supply = max_supply;
    edition.save(edition_account_info)?;

    if metadata_account_info.is_writable {
        let mut metadata_mut = Metadata::from_account_info(metadata_account_info)?;
        metadata_mut.token_standard = Some(TokenStandard::NonFungible);
        metadata_mut.save(&mut metadata_account_info.try_borrow_mut_data()?)?;
    }

    // while you can't mint only mint 1 token from your master record, you can
    // mint as many limited editions as you like within your max supply
    transfer_mint_authority(
        edition_account_info.key,
        edition_account_info,
        mint_info,
        mint_authority_info,
        token_program_info,
    )
}
