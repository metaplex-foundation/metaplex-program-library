use mpl_utils::assert_signer;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    assertions::{assert_owned_by, metadata::assert_currently_holding},
    instruction::{Burn, Context},
    state::{Metadata, TokenMetadataAccount},
};

use super::nonfungible::{burn_nonfungible, BurnNonFungibleArgs};

pub fn process_burn_nft<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;

    let collection_nft_provided = accounts.len() == 7;
    let collection_metadata_info = if collection_nft_provided {
        Some(next_account_info(account_info_iter)?)
    } else {
        None
    };

    // Validate accounts

    // Assert signer
    assert_signer(owner_info)?;

    // Assert program ownership.
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(edition_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::ID)?;
    assert_owned_by(token_info, &spl_token::ID)?;

    // Check program IDs.
    if spl_token_program_info.key != &spl_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize accounts.
    let metadata = Metadata::from_account_info(metadata_info)?;

    // Checks:
    // * Metadata is owned by the token-metadata program
    // * Mint is owned by the spl-token program
    // * Token is owned by the spl-token program
    // * Token account is initialized
    // * Token account data owner is 'owner'
    // * Token account belongs to mint
    // * Token account has 1 or more tokens
    // * Mint matches metadata.mint
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_info,
    )?;

    // Contruct our new Burn handler context so we can re-use the same code for both.
    let accounts = Burn {
        authority_info: owner_info,
        collection_metadata_info,
        metadata_info,
        edition_info: Some(edition_info),
        mint_info,
        token_info,
        parent_edition_info: None,
        parent_mint_info: None,
        parent_token_info: None,
        edition_marker_info: None,
        token_record_info: None,
        // This handler doesn't get system program and sysvars instructions
        // but we need them to create the Burn struct. They are not used in the burn_nonfungible handler.
        system_program_info: spl_token_program_info,
        sysvar_instructions_info: spl_token_program_info,
        spl_token_program_info,
    };
    let context = Context {
        accounts,
        remaining_accounts: vec![],
    };

    let args = BurnNonFungibleArgs { metadata };
    burn_nonfungible(&context, args)
}
