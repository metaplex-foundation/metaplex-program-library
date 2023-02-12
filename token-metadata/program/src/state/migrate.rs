use super::*;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum MigrationType {
    CollectionV1,
    ProgrammableV1,
}
