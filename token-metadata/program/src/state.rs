use std::io::ErrorKind;

use crate::{
    deser::meta_deser_unchecked,
    error::MetadataError,
    utils::{assert_owned_by, is_correct_account_type, try_from_slice_checked},
    ID,
};
use borsh::{maybestd::io::Error as BorshError, BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

#[cfg(feature = "serde")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

/// prefix used for PDAs to avoid certain collision attacks (https://en.wikipedia.org/wiki/Collision_attack#Chosen-prefix_collision_attack)
pub const PREFIX: &str = "metadata";

/// Used in seeds to make Edition model pda address
pub const EDITION: &str = "edition";

pub const RESERVATION: &str = "reservation";

pub const USER: &str = "user";

pub const BURN: &str = "burn";

pub const COLLECTION_AUTHORITY: &str = "collection_authority";

pub const MAX_NAME_LENGTH: usize = 32;

pub const MAX_SYMBOL_LENGTH: usize = 10;

pub const MAX_URI_LENGTH: usize = 200;

pub const MAX_METADATA_LEN: usize = 1 //key 
+ 32 // update auth pubkey
+ 32 // mint pubkey
+ MAX_DATA_SIZE
+ 1 // primary sale
+ 1 // mutable
+ 9 // nonce (pretty sure this only needs to be 2)
+ 2 // token standard
+ 34 // collection
+ 18 // uses
+ 118; // Padding

pub const MAX_DATA_SIZE: usize = 4
    + MAX_NAME_LENGTH
    + 4
    + MAX_SYMBOL_LENGTH
    + 4
    + MAX_URI_LENGTH
    + 2
    + 1
    + 4
    + MAX_CREATOR_LIMIT * MAX_CREATOR_LEN;

pub const MAX_EDITION_LEN: usize = 1 + 32 + 8 + 200;

// Large buffer because the older master editions have two pubkeys in them,
// need to keep two versions same size because the conversion process actually changes the same account
// by rewriting it.
pub const MAX_MASTER_EDITION_LEN: usize = 1 + 9 + 8 + 264;

pub const MAX_CREATOR_LIMIT: usize = 5;

pub const MAX_CREATOR_LEN: usize = 32 + 1 + 1;

pub const MAX_RESERVATIONS: usize = 200;

// can hold up to 200 keys per reservation, note: the extra 8 is for number of elements in the vec
pub const MAX_RESERVATION_LIST_V1_SIZE: usize = 1 + 32 + 8 + 8 + MAX_RESERVATIONS * 34 + 100;

// can hold up to 200 keys per reservation, note: the extra 8 is for number of elements in the vec
pub const MAX_RESERVATION_LIST_SIZE: usize = 1 + 32 + 8 + 8 + MAX_RESERVATIONS * 48 + 8 + 8 + 84;

pub const MAX_EDITION_MARKER_SIZE: usize = 32;

pub const EDITION_MARKER_BIT_SIZE: u64 = 248;

pub const USE_AUTHORITY_RECORD_SIZE: usize = 18; //8 byte padding

pub const COLLECTION_AUTHORITY_RECORD_SIZE: usize = 11; //10 byte padding

pub trait TokenMetadataAccount {
    fn key() -> Key;

    fn size() -> usize;

    fn pad_length(buf: &mut Vec<u8>) -> Result<(), MetadataError> {
        let padding_length = Self::size()
            .checked_sub(buf.len())
            .ok_or(MetadataError::NumericalOverflowError)?;
        buf.extend(vec![0; padding_length]);
        Ok(())
    }

    fn safe_deserialize<T: BorshDeserialize>(mut data: &[u8]) -> Result<T, BorshError> {
        if !is_correct_account_type(data, Self::key(), Self::size()) {
            return Err(BorshError::new(ErrorKind::Other, "DataTypeMismatch"));
        }

        let result: T = T::deserialize(&mut data)?;

        Ok(result)
    }

    fn from_account_info<T: BorshDeserialize>(a: &AccountInfo) -> Result<T, ProgramError>
where {
        let ua: T = Self::safe_deserialize(&a.data.borrow_mut())
            .map_err(|_| MetadataError::DataTypeMismatch)?;

        // Check that this is a `token-metadata` owned account.
        assert_owned_by(a, &ID)?;

        Ok(ua)
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum Key {
    Uninitialized,
    EditionV1,
    MasterEditionV1,
    ReservationListV1,
    MetadataV1,
    ReservationListV2,
    MasterEditionV2,
    EditionMarker,
    UseAuthorityRecord,
    CollectionAuthorityRecord,
}
#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, Default, PartialEq, Debug, Clone)]
pub struct Data {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<Creator>>,
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct DataV2 {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<Creator>>,
    /// Collection
    pub collection: Option<Collection>,
    /// Uses
    pub uses: Option<Uses>,
}

impl DataV2 {
    pub fn to_v1(&self) -> Data {
        let ns = self.clone();
        Data {
            name: ns.name,
            symbol: ns.symbol,
            uri: ns.uri,
            seller_fee_basis_points: ns.seller_fee_basis_points,
            creators: ns.creators,
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum CollectionDetails {
    V1 { size: u64 },
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Uses {
    // 17 bytes + Option byte
    pub use_method: UseMethod, //1
    pub remaining: u64,        //8
    pub total: u64,            //8
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum TokenStandard {
    NonFungible,        // This is a master edition
    FungibleAsset,      // A token with metadata that can also have attrributes
    Fungible,           // A token with simple metadata
    NonFungibleEdition, // This is a limited edition
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, ShankAccount)]
pub struct UseAuthorityRecord {
    pub key: Key,          //1
    pub allowed_uses: u64, //8
    pub bump: u8,
}

impl Default for UseAuthorityRecord {
    fn default() -> Self {
        UseAuthorityRecord {
            key: Key::UseAuthorityRecord,
            allowed_uses: 0,
            bump: 255,
        }
    }
}

impl TokenMetadataAccount for UseAuthorityRecord {
    fn key() -> Key {
        Key::UseAuthorityRecord
    }

    fn size() -> usize {
        USE_AUTHORITY_RECORD_SIZE
    }
}

impl UseAuthorityRecord {
    pub fn from_bytes(b: &[u8]) -> Result<UseAuthorityRecord, ProgramError> {
        let ua: UseAuthorityRecord =
            try_from_slice_checked(b, Key::UseAuthorityRecord, USE_AUTHORITY_RECORD_SIZE)?;
        Ok(ua)
    }

    pub fn bump_empty(&self) -> bool {
        self.bump == 0 && self.key == Key::UseAuthorityRecord
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, ShankAccount)]
pub struct CollectionAuthorityRecord {
    pub key: Key, //1
    pub bump: u8, //1
}

impl Default for CollectionAuthorityRecord {
    fn default() -> Self {
        CollectionAuthorityRecord {
            key: Key::CollectionAuthorityRecord,
            bump: 255,
        }
    }
}

impl TokenMetadataAccount for CollectionAuthorityRecord {
    fn key() -> Key {
        Key::CollectionAuthorityRecord
    }

    fn size() -> usize {
        COLLECTION_AUTHORITY_RECORD_SIZE
    }
}

impl CollectionAuthorityRecord {
    pub fn from_bytes(b: &[u8]) -> Result<CollectionAuthorityRecord, ProgramError> {
        let ca: CollectionAuthorityRecord = try_from_slice_checked(
            b,
            Key::CollectionAuthorityRecord,
            COLLECTION_AUTHORITY_RECORD_SIZE,
        )?;
        Ok(ca)
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, BorshSerialize, Debug, PartialEq, ShankAccount)]
pub struct Metadata {
    pub key: Key,
    pub update_authority: Pubkey,
    pub mint: Pubkey,
    pub data: Data,
    // Immutable, once flipped, all sales of this metadata are considered secondary.
    pub primary_sale_happened: bool,
    // Whether or not the data struct is mutable, default is not
    pub is_mutable: bool,
    /// nonce for easy calculation of editions, if present
    pub edition_nonce: Option<u8>,
    /// Since we cannot easily change Metadata, we add the new DataV2 fields here at the end.
    pub token_standard: Option<TokenStandard>,
    /// Collection
    pub collection: Option<Collection>,
    /// Uses
    pub uses: Option<Uses>,
    /// Item Details
    pub collection_details: Option<CollectionDetails>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            key: Key::MetadataV1,
            update_authority: Pubkey::default(),
            mint: Pubkey::default(),
            data: Data::default(),
            primary_sale_happened: false,
            is_mutable: false,
            edition_nonce: None,
            token_standard: None,
            collection: None,
            uses: None,
            collection_details: None,
        }
    }
}

impl TokenMetadataAccount for Metadata {
    fn key() -> Key {
        Key::MetadataV1
    }

    fn size() -> usize {
        MAX_METADATA_LEN
    }
}

// We have a custom implementation of BorshDeserialize for Metadata because of corrupted metadata issues
// caused by resizing of the Creators array. We use a custom `meta_deser_unchecked` function
// that has fallback values for corrupted fields.
impl borsh::de::BorshDeserialize for Metadata {
    fn deserialize(buf: &mut &[u8]) -> ::core::result::Result<Self, BorshError> {
        let md = meta_deser_unchecked(buf)?;
        Ok(md)
    }
}

pub trait MasterEdition {
    fn key(&self) -> Key;
    fn supply(&self) -> u64;
    fn set_supply(&mut self, supply: u64);
    fn max_supply(&self) -> Option<u64>;
    fn save(&self, account: &AccountInfo) -> ProgramResult;
}

pub fn get_master_edition(account: &AccountInfo) -> Result<Box<dyn MasterEdition>, ProgramError> {
    let version = account.data.borrow()[0];

    // For some reason when converting Key to u8 here, it becomes unreachable. Use direct constant instead.
    let master_edition_result: Result<Box<dyn MasterEdition>, ProgramError> = match version {
        2 => {
            let me: MasterEditionV1 = MasterEditionV1::from_account_info(account)?;
            Ok(Box::new(me))
        }
        6 => {
            let me: MasterEditionV2 = MasterEditionV2::from_account_info(account)?;
            Ok(Box::new(me))
        }
        _ => Err(MetadataError::DataTypeMismatch.into()),
    };

    master_edition_result
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, ShankAccount)]
pub struct MasterEditionV2 {
    pub key: Key,

    pub supply: u64,

    pub max_supply: Option<u64>,
}

impl Default for MasterEditionV2 {
    fn default() -> Self {
        MasterEditionV2 {
            key: Key::MasterEditionV2,
            supply: 0,
            max_supply: Some(0),
        }
    }
}

impl TokenMetadataAccount for MasterEditionV2 {
    fn key() -> Key {
        Key::MasterEditionV2
    }

    fn size() -> usize {
        MAX_MASTER_EDITION_LEN
    }
}

impl MasterEdition for MasterEditionV2 {
    fn key(&self) -> Key {
        self.key
    }

    fn supply(&self) -> u64 {
        self.supply
    }

    fn set_supply(&mut self, supply: u64) {
        self.supply = supply;
    }

    fn max_supply(&self) -> Option<u64> {
        self.max_supply
    }

    fn save(&self, account: &AccountInfo) -> ProgramResult {
        BorshSerialize::serialize(self, &mut *account.data.borrow_mut())?;
        Ok(())
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, ShankAccount)]
pub struct MasterEditionV1 {
    pub key: Key,

    pub supply: u64,

    pub max_supply: Option<u64>,

    /// Can be used to mint tokens that give one-time permission to mint a single limited edition.
    pub printing_mint: Pubkey,

    /// If you don't know how many printing tokens you are going to need, but you do know
    /// you are going to need some amount in the future, you can use a token from this mint.
    /// Coming back to token metadata with one of these tokens allows you to mint (one time)
    /// any number of printing tokens you want. This is used for instance by Auction Manager
    /// with participation NFTs, where we dont know how many people will bid and need participation
    /// printing tokens to redeem, so we give it ONE of these tokens to use after the auction is over,
    /// because when the auction begins we just dont know how many printing tokens we will need,
    /// but at the end we will. At the end it then burns this token with token-metadata to
    /// get the printing tokens it needs to give to bidders. Each bidder then redeems a printing token
    /// to get their limited editions.
    pub one_time_printing_authorization_mint: Pubkey,
}

impl TokenMetadataAccount for MasterEditionV1 {
    fn key() -> Key {
        Key::MasterEditionV1
    }

    fn size() -> usize {
        MAX_MASTER_EDITION_LEN
    }
}

impl MasterEdition for MasterEditionV1 {
    fn key(&self) -> Key {
        self.key
    }

    fn supply(&self) -> u64 {
        self.supply
    }

    fn max_supply(&self) -> Option<u64> {
        self.max_supply
    }

    fn set_supply(&mut self, supply: u64) {
        self.supply = supply;
    }

    fn save(&self, account: &AccountInfo) -> ProgramResult {
        BorshSerialize::serialize(self, &mut *account.data.borrow_mut())?;
        Ok(())
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, ShankAccount)]
/// All Editions should never have a supply greater than 1.
/// To enforce this, a transfer mint authority instruction will happen when
/// a normal token is turned into an Edition, and in order for a Metadata update authority
/// to do this transaction they will also need to sign the transaction as the Mint authority.
pub struct Edition {
    pub key: Key,

    /// Points at MasterEdition struct
    pub parent: Pubkey,

    /// Starting at 0 for master record, this is incremented for each edition minted.
    pub edition: u64,
}

impl Default for Edition {
    fn default() -> Self {
        Edition {
            key: Key::EditionV1,
            parent: Pubkey::default(),
            edition: 0,
        }
    }
}

impl TokenMetadataAccount for Edition {
    fn key() -> Key {
        Key::EditionV1
    }

    fn size() -> usize {
        MAX_EDITION_LEN
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Creator {
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

pub trait ReservationList {
    fn master_edition(&self) -> Pubkey;
    fn supply_snapshot(&self) -> Option<u64>;
    fn reservations(&self) -> Vec<Reservation>;
    fn total_reservation_spots(&self) -> u64;
    fn current_reservation_spots(&self) -> u64;
    fn set_master_edition(&mut self, key: Pubkey);
    fn set_supply_snapshot(&mut self, supply: Option<u64>);
    fn set_reservations(&mut self, reservations: Vec<Reservation>) -> ProgramResult;
    fn add_reservation(
        &mut self,
        reservation: Reservation,
        offset: u64,
        total_spot_offset: u64,
    ) -> ProgramResult;
    fn set_total_reservation_spots(&mut self, total_reservation_spots: u64);
    fn set_current_reservation_spots(&mut self, current_reservation_spots: u64);
    fn save(&self, account: &AccountInfo) -> ProgramResult;
}

pub fn get_reservation_list(
    account: &AccountInfo,
) -> Result<Box<dyn ReservationList>, ProgramError> {
    let version = account.data.borrow()[0];

    // For some reason when converting Key to u8 here, it becomes unreachable. Use direct constant instead.
    let reservation_list_result: Result<Box<dyn ReservationList>, ProgramError> = match version {
        3 => {
            let reservation_list = Box::new(ReservationListV1::from_account_info::<
                ReservationListV1,
            >(account)?);
            Ok(reservation_list)
        }
        5 => {
            let reservation_list = Box::new(ReservationListV2::from_account_info::<
                ReservationListV2,
            >(account)?);
            Ok(reservation_list)
        }
        _ => Err(MetadataError::DataTypeMismatch.into()),
    };

    reservation_list_result
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, ShankAccount)]
pub struct ReservationListV2 {
    pub key: Key,
    /// Present for reverse lookups
    pub master_edition: Pubkey,

    /// What supply counter was on master_edition when this reservation was created.
    pub supply_snapshot: Option<u64>,
    pub reservations: Vec<Reservation>,
    /// How many reservations there are going to be, given on first set_reservation call
    pub total_reservation_spots: u64,
    /// Cached count of reservation spots in the reservation vec to save on CPU.
    pub current_reservation_spots: u64,
}

impl TokenMetadataAccount for ReservationListV2 {
    fn key() -> Key {
        Key::ReservationListV2
    }

    fn size() -> usize {
        MAX_RESERVATION_LIST_SIZE
    }
}

impl ReservationList for ReservationListV2 {
    fn master_edition(&self) -> Pubkey {
        self.master_edition
    }

    fn supply_snapshot(&self) -> Option<u64> {
        self.supply_snapshot
    }

    fn reservations(&self) -> Vec<Reservation> {
        self.reservations.clone()
    }

    fn set_master_edition(&mut self, key: Pubkey) {
        self.master_edition = key
    }

    fn set_supply_snapshot(&mut self, supply: Option<u64>) {
        self.supply_snapshot = supply;
    }

    fn add_reservation(
        &mut self,
        reservation: Reservation,
        offset: u64,
        total_spot_offset: u64,
    ) -> ProgramResult {
        let usize_offset = offset as usize;
        while self.reservations.len() < usize_offset {
            self.reservations.push(Reservation {
                address: solana_program::system_program::id(),
                spots_remaining: 0,
                total_spots: 0,
            })
        }
        if self.reservations.len() > usize_offset {
            let replaced_addr = self.reservations[usize_offset].address;
            let replaced_spots = self.reservations[usize_offset].total_spots;

            if replaced_addr == reservation.address {
                // Since we will have incremented, decrease in advance so we dont blow the spot check.
                // Super hacky but this code is to be deprecated.
                self.set_current_reservation_spots(
                    self.current_reservation_spots()
                        .checked_sub(replaced_spots)
                        .ok_or(MetadataError::NumericalOverflowError)?,
                );
            } else if replaced_addr != solana_program::system_program::id() {
                return Err(MetadataError::TriedToReplaceAnExistingReservation.into());
            }
            self.reservations[usize_offset] = reservation;
        } else {
            self.reservations.push(reservation)
        }

        if usize_offset != 0
            && self.reservations[usize_offset - 1].address == solana_program::system_program::id()
        {
            // This becomes an anchor then for calculations... put total spot offset in here.
            self.reservations[usize_offset - 1].spots_remaining = total_spot_offset;
            self.reservations[usize_offset - 1].total_spots = total_spot_offset;
        }

        Ok(())
    }

    fn set_reservations(&mut self, reservations: Vec<Reservation>) -> ProgramResult {
        self.reservations = reservations;
        Ok(())
    }

    fn save(&self, account: &AccountInfo) -> ProgramResult {
        BorshSerialize::serialize(self, &mut *account.data.borrow_mut())?;
        Ok(())
    }

    fn total_reservation_spots(&self) -> u64 {
        self.total_reservation_spots
    }

    fn set_total_reservation_spots(&mut self, total_reservation_spots: u64) {
        self.total_reservation_spots = total_reservation_spots;
    }

    fn current_reservation_spots(&self) -> u64 {
        self.current_reservation_spots
    }

    fn set_current_reservation_spots(&mut self, current_reservation_spots: u64) {
        self.current_reservation_spots = current_reservation_spots;
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Reservation {
    pub address: Pubkey,
    pub spots_remaining: u64,
    pub total_spots: u64,
}

// Legacy Reservation List with u8s
#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, ShankAccount)]
pub struct ReservationListV1 {
    pub key: Key,
    /// Present for reverse lookups
    pub master_edition: Pubkey,

    /// What supply counter was on master_edition when this reservation was created.
    pub supply_snapshot: Option<u64>,
    pub reservations: Vec<ReservationV1>,
}

impl TokenMetadataAccount for ReservationListV1 {
    fn key() -> Key {
        Key::ReservationListV1
    }

    fn size() -> usize {
        MAX_RESERVATION_LIST_V1_SIZE
    }
}

impl ReservationList for ReservationListV1 {
    fn master_edition(&self) -> Pubkey {
        self.master_edition
    }

    fn supply_snapshot(&self) -> Option<u64> {
        self.supply_snapshot
    }

    fn reservations(&self) -> Vec<Reservation> {
        self.reservations
            .iter()
            .map(|r| Reservation {
                address: r.address,
                spots_remaining: r.spots_remaining as u64,
                total_spots: r.total_spots as u64,
            })
            .collect()
    }

    fn set_master_edition(&mut self, key: Pubkey) {
        self.master_edition = key
    }

    fn set_supply_snapshot(&mut self, supply: Option<u64>) {
        self.supply_snapshot = supply;
    }

    fn add_reservation(&mut self, reservation: Reservation, _: u64, _: u64) -> ProgramResult {
        self.reservations = vec![ReservationV1 {
            address: reservation.address,
            spots_remaining: reservation.spots_remaining as u8,
            total_spots: reservation.total_spots as u8,
        }];

        Ok(())
    }

    fn set_reservations(&mut self, reservations: Vec<Reservation>) -> ProgramResult {
        self.reservations = reservations
            .iter()
            .map(|r| ReservationV1 {
                address: r.address,
                spots_remaining: r.spots_remaining as u8,
                total_spots: r.total_spots as u8,
            })
            .collect();
        Ok(())
    }

    fn save(&self, account: &AccountInfo) -> ProgramResult {
        BorshSerialize::serialize(self, &mut *account.data.borrow_mut())?;
        Ok(())
    }

    fn total_reservation_spots(&self) -> u64 {
        self.reservations.iter().map(|r| r.total_spots as u64).sum()
    }

    fn set_total_reservation_spots(&mut self, _: u64) {}

    fn current_reservation_spots(&self) -> u64 {
        self.reservations.iter().map(|r| r.total_spots as u64).sum()
    }

    fn set_current_reservation_spots(&mut self, _: u64) {}
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ReservationV1 {
    pub address: Pubkey,
    pub spots_remaining: u8,
    pub total_spots: u8,
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, ShankAccount)]
pub struct EditionMarker {
    pub key: Key,
    pub ledger: [u8; 31],
}

impl Default for EditionMarker {
    fn default() -> Self {
        Self {
            key: Key::EditionMarker,
            ledger: [0; 31],
        }
    }
}

impl TokenMetadataAccount for EditionMarker {
    fn key() -> Key {
        Key::EditionMarker
    }

    fn size() -> usize {
        MAX_EDITION_MARKER_SIZE
    }
}

impl EditionMarker {
    fn get_edition_offset_from_starting_index(edition: u64) -> Result<usize, ProgramError> {
        Ok(edition
            .checked_rem(EDITION_MARKER_BIT_SIZE)
            .ok_or(MetadataError::NumericalOverflowError)? as usize)
    }

    fn get_index(offset_from_start: usize) -> Result<usize, ProgramError> {
        let index = offset_from_start
            .checked_div(8)
            .ok_or(MetadataError::NumericalOverflowError)?;

        // With only EDITION_MARKER_BIT_SIZE bits, or 31 bytes, we have a max constraint here.
        if index > 30 {
            return Err(MetadataError::InvalidEditionIndex.into());
        }

        Ok(index)
    }

    fn get_offset_from_right(offset_from_start: usize) -> Result<u32, ProgramError> {
        // We're saying the left hand side of a u8 is the 0th index so to get a 1 in that 0th index
        // you need to shift a 1 over 8 spots from the right hand side. To do that you actually
        // need not 00000001 but 10000000 which you can get by simply multiplying 1 by 2^7, 128 and then ORing
        // it with the current value.
        Ok(7 - offset_from_start
            .checked_rem(8)
            .ok_or(MetadataError::NumericalOverflowError)? as u32)
    }

    fn get_index_and_mask(edition: u64) -> Result<(usize, u8), ProgramError> {
        // How many editions off we are from edition at 0th index
        let offset_from_start = EditionMarker::get_edition_offset_from_starting_index(edition)?;

        // How many whole u8s we are from the u8 at the 0th index, which basically dividing by 8
        let index = EditionMarker::get_index(offset_from_start)?;

        // what position in the given u8 bitset are we (remainder math)
        let my_position_in_index_starting_from_right =
            EditionMarker::get_offset_from_right(offset_from_start)?;

        Ok((
            index,
            u8::pow(2, my_position_in_index_starting_from_right as u32),
        ))
    }

    pub fn edition_taken(&self, edition: u64) -> Result<bool, ProgramError> {
        let (index, mask) = EditionMarker::get_index_and_mask(edition)?;

        // apply mask with bitwise and with a 1 to determine if it is set or not
        let applied_mask = self.ledger[index] & mask;

        // What remains should not equal 0.
        Ok(applied_mask != 0)
    }

    pub fn insert_edition(&mut self, edition: u64) -> ProgramResult {
        let (index, mask) = EditionMarker::get_index_and_mask(edition)?;
        // bitwise or a 1 into our position in that position
        self.ledger[index] |= mask;
        Ok(())
    }
}
