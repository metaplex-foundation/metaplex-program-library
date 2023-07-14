use mpl_utils::token::get_mint_supply;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{self, Sysvar},
};
use spl_token::state::Mint;

use crate::{
    assertions::assert_keys_equal,
    error::MetadataError,
    instruction::{Context, Print, PrintArgs},
    pda::find_token_record_account,
    state::{
        Metadata, TokenMetadataAccount, TokenStandard, MAX_EDITION_LEN,
        TOKEN_STANDARD_INDEX_EDITION,
    },
    utils::{
        assert_initialized, assert_owned_by, create_token_record_account,
        fee::{levy, set_fee_flag, LevyArgs},
        freeze, process_mint_new_edition_from_master_edition_via_token_logic,
        MintNewEditionFromMasterEditionViaTokenLogicArgs,
    },
};

pub fn print<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: PrintArgs,
) -> ProgramResult {
    let context = Print::to_context(accounts)?;

    match args {
        PrintArgs::V1 { .. } => print_v1(program_id, context, args),
    }
}

fn print_v1(_program_id: &Pubkey, ctx: Context<Print>, args: PrintArgs) -> ProgramResult {
    // Get the args for the instruction
    let PrintArgs::V1 { edition } = args;

    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let edition_metadata_info = ctx.accounts.edition_metadata_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let edition_account_info = ctx.accounts.edition_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let edition_mint_info = ctx.accounts.edition_mint_info;
    let edition_token_account_owner_info = ctx.accounts.edition_token_account_owner_info;
    let edition_token_account_info = ctx.accounts.edition_token_account_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let edition_mint_authority_info = ctx.accounts.edition_mint_authority_info;
    let edition_token_record_info = ctx.accounts.edition_token_record_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let master_edition_info = ctx.accounts.master_edition_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let edition_marker_pda_info = ctx.accounts.edition_marker_pda_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let payer_info = ctx.accounts.payer_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let master_token_account_owner_info = ctx.accounts.master_token_account_owner_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let master_token_account_info = ctx.accounts.master_token_account_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let master_metadata_info = ctx.accounts.master_metadata_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let update_authority_info = ctx.accounts.update_authority_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let token_program = ctx.accounts.spl_token_program_info;
    let ata_program = ctx.accounts.spl_ata_program_info;
    let sysvar_instructions = ctx.accounts.sysvar_instructions_info;
    // CHECK: Checked in process_mint_new_edition_from_master_edition_via_token_logic
    let system_program = ctx.accounts.system_program_info;

    // Levy fees first, to fund the metadata account with rent + fee amount.
    levy(LevyArgs {
        payer_account_info: payer_info,
        token_metadata_pda_info: edition_metadata_info,
    })?;

    // if the account does not exist, we will allocate a new mint
    if edition_mint_info.data_is_empty() {
        // mint account must be a signer in the transaction
        if !edition_mint_info.is_signer {
            return Err(MetadataError::MintIsNotSigner.into());
        }

        invoke(
            &system_instruction::create_account(
                payer_info.key,
                edition_mint_info.key,
                Rent::get()?.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::ID,
            ),
            &[payer_info.clone(), edition_mint_info.clone()],
        )?;

        // initializing the mint account
        invoke(
            &spl_token::instruction::initialize_mint2(
                token_program.key,
                edition_mint_info.key,
                edition_account_info.key,
                Some(edition_account_info.key),
                0,
            )?,
            &[edition_mint_info.clone(), edition_account_info.clone()],
        )?;
    } else {
        // validates the existing mint account

        let mint: Mint = assert_initialized(edition_mint_info)?;
        // NonFungible assets must have decimals == 0 and supply no greater than 1
        if mint.decimals > 0 || mint.supply > 1 {
            return Err(MetadataError::InvalidMintForTokenStandard.into());
        }
    }

    // If the edition token account isn't already initialized, create it.
    // If it does exist, validate it.
    if edition_token_account_info.data_is_empty() {
        // If the token account is empty, we need to double check the token isn't just in another account.
        // We do this by checking supply == 0
        let mint_supply = get_mint_supply(edition_mint_info)?;
        if mint_supply > 0 {
            return Err(MetadataError::MintSupplyMustBeZero.into());
        }

        // creating the associated token account
        invoke(
            &spl_associated_token_account::instruction::create_associated_token_account(
                payer_info.key,
                edition_token_account_owner_info.key,
                edition_mint_info.key,
                &spl_token::id(),
            ),
            &[
                payer_info.clone(),
                edition_token_account_owner_info.clone(),
                edition_mint_info.clone(),
                edition_token_account_info.clone(),
            ],
        )?;
    } else {
        assert_owned_by(edition_token_account_info, &spl_token::id())?;
        let edition_token_account: spl_token::state::Account =
            assert_initialized(edition_token_account_info)?;
        if edition_token_account.amount < 1 {
            return Err(MetadataError::NotEnoughTokens.into());
        }
    }

    if ata_program.key != &spl_associated_token_account::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if sysvar_instructions.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize the master edition's metadata so we can determine token type
    let master_metadata = Metadata::from_account_info(master_metadata_info)?;
    let token_standard = master_metadata
        .token_standard
        .unwrap_or(TokenStandard::NonFungible);

    match token_standard {
        TokenStandard::NonFungible => {}
        TokenStandard::ProgrammableNonFungible => {
            // Validate that the token record was passed in for pNFTs.
            let token_record_info =
                edition_token_record_info.ok_or(MetadataError::MissingTokenRecord)?;
            let (pda_key, _) = find_token_record_account(
                ctx.accounts.edition_mint_info.key,
                ctx.accounts.edition_token_account_info.key,
            );
            // validates the derivation
            assert_keys_equal(&pda_key, token_record_info.key)?;

            if token_record_info.data_is_empty() {
                create_token_record_account(
                    &crate::ID,
                    token_record_info,
                    edition_mint_info,
                    edition_token_account_info,
                    payer_info,
                    system_program,
                )?;
            } else {
                assert_owned_by(token_record_info, &crate::ID)?;
            }
        }
        _ => return Err(MetadataError::InvalidTokenStandard.into()),
    };

    // Check that the new update authority is the same as the master edition.
    if update_authority_info.key != &master_metadata.update_authority {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }

    process_mint_new_edition_from_master_edition_via_token_logic(
        &crate::ID,
        MintNewEditionFromMasterEditionViaTokenLogicArgs {
            new_metadata_account_info: edition_metadata_info,
            new_edition_account_info: edition_account_info,
            master_edition_account_info: master_edition_info,
            mint_info: edition_mint_info,
            edition_marker_info: edition_marker_pda_info,
            mint_authority_info: edition_mint_authority_info,
            payer_account_info: payer_info,
            owner_account_info: master_token_account_owner_info,
            token_account_info: master_token_account_info,
            update_authority_info,
            master_metadata_account_info: master_metadata_info,
            token_program_account_info: token_program,
            system_account_info: system_program,
        },
        edition,
    )?;

    if token_standard == TokenStandard::ProgrammableNonFungible {
        freeze(
            edition_mint_info.clone(),
            edition_token_account_info.clone(),
            edition_account_info.clone(),
            token_program.clone(),
        )?;

        // for pNFTs, we store the token standard value at the end of the
        // master edition account
        let mut data = edition_account_info.data.borrow_mut();

        if data.len() < MAX_EDITION_LEN {
            return Err(MetadataError::InvalidMasterEditionAccountLength.into());
        }

        data[TOKEN_STANDARD_INDEX_EDITION] = TokenStandard::ProgrammableNonFungible as u8;
    }

    // Set fee flag after metadata account is created.
    set_fee_flag(edition_metadata_info)
}
