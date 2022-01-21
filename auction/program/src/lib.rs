#![allow(warnings)]

mod utils;

pub mod entrypoint;
pub mod errors;
pub mod instruction;
pub mod processor;

/// Prefix used in PDA derivations to avoid collisions with other programs.
pub const PREFIX: &str = "auction";
pub const EXTENDED: &str = "extended";
pub const BIDDER_POT_TOKEN: &str = "bidder_pot_token";
solana_program::declare_id!("NH1kx3qaCqEMABLf3H5Dh3XKzFXd6LgoSmTyZj9p3L8");
