use super::*;

pub const RESERVATION: &str = "reservation";

pub const MAX_RESERVATIONS: usize = 200;

// can hold up to 200 keys per reservation, note: the extra 8 is for number of elements in the vec
pub const MAX_RESERVATION_LIST_V1_SIZE: usize = 1 + 32 + 8 + 8 + MAX_RESERVATIONS * 34 + 100;

// can hold up to 200 keys per reservation, note: the extra 8 is for number of elements in the vec
pub const MAX_RESERVATION_LIST_SIZE: usize = 1 + 32 + 8 + 8 + MAX_RESERVATIONS * 48 + 8 + 8 + 84;

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
            let reservation_list = Box::new(ReservationListV1::from_account_info(account)?);
            Ok(reservation_list)
        }
        5 => {
            let reservation_list = Box::new(ReservationListV2::from_account_info(account)?);
            Ok(reservation_list)
        }
        _ => Err(MetadataError::DataTypeMismatch.into()),
    };

    reservation_list_result
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
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
                address: solana_program::system_program::ID,
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
            } else if replaced_addr != solana_program::system_program::ID {
                return Err(MetadataError::TriedToReplaceAnExistingReservation.into());
            }
            self.reservations[usize_offset] = reservation;
        } else {
            self.reservations.push(reservation)
        }

        if usize_offset != 0
            && self.reservations[usize_offset - 1].address == solana_program::system_program::ID
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
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Reservation {
    pub address: Pubkey,
    pub spots_remaining: u64,
    pub total_spots: u64,
}

// Legacy Reservation List with u8s
#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
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
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct ReservationV1 {
    pub address: Pubkey,
    pub spots_remaining: u8,
    pub total_spots: u8,
}
