/// These types exist to give Shank a way to create the Payload type as it
/// cannnot create it from the remote type from mpl-token-auth-rules.
/// Care will need to be taken to ensure they stay synced with any changes in
/// mpl-token-auth-rules.
use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
struct SeedsVec {
    seeds: Vec<Vec<u8>>,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
struct LeafInfo {
    leaf: [u8; 32],
    proof: Vec<[u8; 32]>,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
enum PayloadType {
    Pubkey(Pubkey),
    Seeds(SeedsVec),
    MerkleProof(LeafInfo),
    Number(u64),
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
struct Payload {
    map: HashMap<String, PayloadType>,
}
