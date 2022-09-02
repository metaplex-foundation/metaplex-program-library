use console::style;
use solana_program::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;

use crate::utils::{FreezeInfo, TokenInfo};
use mpl_candy_machine::{constants::PREFIX as CANDY_PREFIX, CollectionPDA, ConfigLine};

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
        &[CollectionPDA::PREFIX.as_bytes(), candy_machine_key.as_ref()],
        &mpl_candy_machine::id(),
    )
}

pub fn find_freeze_ata(freeze_info: &FreezeInfo, token_info: &TokenInfo) -> Pubkey {
    get_associated_token_address(&freeze_info.pda, &token_info.mint)
}

pub fn sol(amount: f64) -> u64 {
    (amount * LAMPORTS_PER_SOL as f64) as u64
}

pub struct CandyTestLogger {
    test_name: String,
}

impl CandyTestLogger {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
        }
    }

    pub fn new_start(test_name: &str) -> Self {
        let new = Self {
            test_name: test_name.to_string(),
        };
        new.start();
        new
    }

    pub fn start(&self) {
        println!(
            "{}",
            style(format!("\n{} start.", self.test_name)).bold().cyan()
        )
    }

    pub fn end(&self) {
        println!(
            "{}",
            style(format!("{} finished!\n", self.test_name))
                .bold()
                .green()
        )
    }
}

pub fn test_start(input: &str) {
    println!("\n{}", style(input).magenta().bold().underlined());
}
