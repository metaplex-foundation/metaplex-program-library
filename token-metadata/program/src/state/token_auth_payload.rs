use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct SeedsVec {
    pub seeds: Vec<Vec<u8>>,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct LeafInfo {
    pub leaf: [u8; 32],
    pub proof: Vec<[u8; 32]>,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum PayloadType {
    Pubkey(Pubkey),
    Seeds(SeedsVec),
    MerkleProof(LeafInfo),
    Number(u64),
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct Payload {
    map: HashMap<PayloadKey, PayloadType>,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialOrd, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PayloadKey {
    Target,
    Holder,
    Authority,
    Amount,
}
