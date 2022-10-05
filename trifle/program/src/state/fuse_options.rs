use borsh::{BorshDeserialize, BorshSerialize};
use modular_bitfield::{bitfield, specifiers::B12};

#[bitfield]
#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub struct FuseOptions {
    pub track: bool,
    pub burn: bool,
    pub freeze: bool,
    pub freeze_parent: bool,
    pub empty_bytes: B12,
}

impl Default for FuseOptions {
    fn default() -> Self {
        FuseOptions::new().with_track(true)
    }
}
