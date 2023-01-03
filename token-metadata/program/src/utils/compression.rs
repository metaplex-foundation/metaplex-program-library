use mpl_utils::cmp_pubkeys;
use solana_program::{account_info::AccountInfo, pubkey, pubkey::Pubkey};

pub const BUBBLEGUM_PROGRAM_ADDRESS: Pubkey =
    pubkey!("BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY");

pub const BUBBLEGUM_SIGNER: Pubkey = pubkey!("4ewWZC5gT6TGpm5LZNDs9wVonfUT2q5PP5sc9kVbwMAK");

// This flag activates certain program authority features of the Bubblegum program.
pub const BUBBLEGUM_ACTIVATED: bool = true;

pub fn find_compression_mint_authority(mint: &Pubkey) -> (Pubkey, u8) {
    let seeds = &[mint.as_ref()];
    Pubkey::find_program_address(seeds, &BUBBLEGUM_PROGRAM_ADDRESS)
}

pub fn is_decompression(mint: &AccountInfo, mint_authority_info: &AccountInfo) -> bool {
    if BUBBLEGUM_ACTIVATED
        && mint_authority_info.is_signer
        && cmp_pubkeys(mint_authority_info.owner, &BUBBLEGUM_PROGRAM_ADDRESS)
    {
        let (expected, _) = find_compression_mint_authority(mint.key);
        return cmp_pubkeys(mint_authority_info.key, &expected);
    }
    false
}
