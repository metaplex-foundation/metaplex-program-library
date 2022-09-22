pub mod escrow_constraints;
pub mod trifle;

use borsh::{BorshDeserialize, BorshSerialize};

pub const ESCROW_PREFIX: &str = "escrow";

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Key {
    EscrowConstraintModel,
    Trifle,
}
