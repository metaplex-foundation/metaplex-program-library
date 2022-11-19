//! A Token Metadata program for the Solana blockchain.

pub mod assertions;

pub mod deprecated_processor {
    pub use crate::processor::deprecated::*;
}

pub mod instruction {
    pub use crate::{
        instruction_old::*,
        processor::{
            collection_instructions::*, edition_instructions::*, metadata_instructions::*,
            operation_instructions::*,
        },
    };
}

pub mod deprecated_instruction {
    pub use crate::processor::deprecated::deprecated_instructions::*;
}

// pub mod deprecated_instructions {
//     // pub use crate::processor::
// }

mod deser;
pub mod entrypoint;
pub mod error;
mod instruction_old;
pub mod pda;
pub mod processor;
pub mod state;
pub mod state_test;
pub mod utils;
pub mod utils_test;
// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
