
use crate::{
    instruction::*,
    state::*,
    error::*,
    pod::*,
    equality_proof,
    ID,
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{Sysvar},
};

use spl_token_metadata::{
    state::MAX_METADATA_LEN,
};

use spl_zk_token_sdk::{
};

use arrayref::{array_ref};

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    match decode_instruction_type(input)? {
        PrivateMetadataInstruction::ConfigureMetadata => {
            msg!("ConfigureMetadata!");
            process_configure_metadata(
                accounts,
                decode_instruction_data::<ConfigureMetadataData>(input)?)
        }
    }
}

fn process_configure_metadata(
    accounts: &[AccountInfo],
    data: &ConfigureMetadataData
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let metadata_update_authority_info = next_account_info(account_info_iter)?;
    let private_metadata_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_sysvar_info = next_account_info(account_info_iter)?;

    validate_account_owner(mint_info, &spl_token::ID)?;
    validate_account_owner(metadata_info, &spl_token_metadata::ID)?;


    // check metadata matches mint
    let metadata_seeds = &[
        spl_token_metadata::state::PREFIX.as_bytes(),
        spl_token_metadata::ID.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (metadata_key, _metadata_bump_seed) =
        Pubkey::find_program_address(metadata_seeds, &spl_token_metadata::ID);

    if metadata_key != *metadata_info.key {
        return Err(PrivateMetadataError::InvalidMetadataKey.into());
    }


    // check that metadata authority matches and that metadata is mutable (adding private metadata
    // and not acting on a limited edition). TODO?
    let metadata_data = metadata_info.try_borrow_data()?;

    let authority_data = array_ref![metadata_data, 1, 32];
    let authority_pubkey = Pubkey::new_from_array(*authority_data);

    if authority_pubkey != *metadata_update_authority_info.key {
        return Err(PrivateMetadataError::InvalidUpdateAuthority.into());
    }

    let is_mutable = metadata_data[MAX_METADATA_LEN - 172 - 9 - 1];
    if is_mutable != 0x01 {
        return Err(PrivateMetadataError::MetadataIsImmutable.into());
    }


    // check that private metadata matches mint
    let private_metadata_seeds = &[
        PREFIX.as_bytes(),
        mint_info.key.as_ref(),
    ];
    let (private_metadata_key, private_metadata_bump_seed) =
        Pubkey::find_program_address(private_metadata_seeds, &ID);

    if private_metadata_key != *private_metadata_info.key {
        return Err(PrivateMetadataError::InvalidPrivateMetadataKey.into());
    }


    // create and initialize PDA
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            private_metadata_info.key,
            rent.minimum_balance(PrivateMetadataAccount::get_packed_len()).max(1),
            PrivateMetadataAccount::get_packed_len() as u64,
            &ID,
        ),
        &[
            payer_info.clone(),
            private_metadata_info.clone(),
            system_program_info.clone(),
        ],
        &[
            &[
                PREFIX.as_bytes(),
                mint_info.key.as_ref(),
                &[private_metadata_bump_seed],
            ],
        ],
    )?;

    let mut private_metadata = PrivateMetadataAccount::from_account_info(
        &private_metadata_info, &ID)?.into_mut();

    private_metadata.mint = *mint_info.key;
    private_metadata.elgamal_pk = data.elgamal_pk;
    private_metadata.uri = data.uri;

    Ok(())
}

fn validate_account_owner(account_info: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account_info.owner == owner {
        Ok(())
    } else {
        Err(ProgramError::InvalidArgument)
    }
}

// TODO: how do we ensure that the encryption key is actually re-encrypted on transfers? can we
// lock some sol and poke the auditor?
//
// so something like the person selling the nft w/ private metadata transfers the token to our
// contract and then we do an auction and then the seller needs to do encrypt the data at that
// point with the pubkey of the winner and then the winner does a transaction where they answer
// the challenge...
