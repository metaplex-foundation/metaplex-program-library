use solana_program::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};

use mpl_candy_machine::{constants::PREFIX as CANDY_PREFIX, ConfigLine};

pub fn make_config_lines(start_index: u32, total: u8) -> Vec<ConfigLine> {
    let mut config_lines = Vec::with_capacity(total as usize);
    for i in 0..total {
        config_lines.push(ConfigLine {
            name: format!("Item #{}", i as u32 + start_index),
            uri: format!("Item #{} URI", i as u32 + start_index),
        })
    }
    config_lines
}

pub fn find_candy_creator(candy_machine_key: &Pubkey) -> (Pubkey, u8) {
    let seeds = &[CANDY_PREFIX.as_bytes(), candy_machine_key.as_ref()];
    Pubkey::find_program_address(seeds, &mpl_candy_machine::id())
}

pub fn find_collection_pda(candy_machine_key: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"collection".as_ref(), candy_machine_key.as_ref()],
        &mpl_candy_machine::id(),
    )
}

pub fn sol(amount: f64) -> u64 {
    (amount * LAMPORTS_PER_SOL as f64) as u64
}
