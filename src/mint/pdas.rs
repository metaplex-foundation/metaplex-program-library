use anchor_client::solana_sdk::pubkey::Pubkey;

use mpl_token_metadata::ID as TOKEN_METADATA_ID;
use spl_associated_token_account::ID as ASSOCIATED_TOKEN_PROGRAM_ID;
use spl_token::ID as TOKEN_PROGRAM_ID;

use crate::candy_machine;

pub fn get_ata_for_mint(mint: &Pubkey, buyer: &Pubkey) -> Pubkey {
    let seeds: &[&[u8]] = &[
        &buyer.to_bytes(),
        &TOKEN_PROGRAM_ID.to_bytes(),
        &mint.to_bytes(),
    ];
    let (pda, _bump) = Pubkey::find_program_address(seeds, &ASSOCIATED_TOKEN_PROGRAM_ID);
    pda
}

pub fn get_metadata_pda(mint: &Pubkey) -> Pubkey {
    // Derive metadata account
    let metadata_seeds = &[
        "metadata".as_bytes(),
        &TOKEN_METADATA_ID.to_bytes(),
        &mint.to_bytes(),
    ];
    let (pda, _bump) = Pubkey::find_program_address(metadata_seeds, &TOKEN_METADATA_ID);

    pda
}

pub fn get_master_edition_pda(mint: &Pubkey) -> Pubkey {
    // Derive Master Edition account
    let master_edition_seeds = &[
        "metadata".as_bytes(),
        &TOKEN_METADATA_ID.to_bytes(),
        &mint.to_bytes(),
        "edition".as_bytes(),
    ];
    let (pda, _bump) = Pubkey::find_program_address(master_edition_seeds, &TOKEN_METADATA_ID);

    pda
}

pub fn get_candy_machine_creator_pda(candy_machine_id: &Pubkey) -> (Pubkey, u8) {
    // Derive metadata account
    let creator_seeds = &["candy_machine".as_bytes(), candy_machine_id.as_ref()];

    Pubkey::find_program_address(creator_seeds, &candy_machine::ID)
}
