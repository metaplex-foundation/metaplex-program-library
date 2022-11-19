mod approve_collection_authority;
mod revoke_collection_authority;
mod set_and_verify_collection;
mod set_and_verify_sized_collection_item;
mod set_collection_size;
mod unverify_collection;
mod unverify_sized_collection_item;
mod verify_collection;
mod verify_sized_collection_item;

pub use approve_collection_authority::approve_collection_authority;
pub use revoke_collection_authority::revoke_collection_authority;
pub use set_and_verify_collection::set_and_verify_collection;
pub use set_and_verify_sized_collection_item::set_and_verify_sized_collection_item;
pub use set_collection_size::set_collection_size;
pub use unverify_collection::unverify_collection;
pub use unverify_sized_collection_item::unverify_sized_collection_item;
pub use verify_collection::verify_collection;
pub use verify_sized_collection_item::verify_sized_collection_item;

pub(crate) mod collection_instructions {
    pub use approve_collection_authority::instruction::*;
    pub use revoke_collection_authority::instruction::*;
    pub use set_and_verify_collection::instruction::*;
    pub use set_and_verify_sized_collection_item::instruction::*;
    pub use set_collection_size::instruction::*;
    pub use unverify_collection::instruction::*;
    pub use unverify_sized_collection_item::instruction::*;
    pub use verify_collection::instruction::*;
    pub use verify_sized_collection_item::instruction::*;

    use super::*;
}
