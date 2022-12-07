use mpl_utils::token::TokenTransferParams;
use solana_program::account_info::next_account_info;
// use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::instruction::TransferArgs;
use crate::state::{Metadata, TokenMetadataAccount, TokenStandard};

pub struct AuthorizationPayloadAccounts<'info> {
    authorization_rules: &'info AccountInfo<'info>,
    authorization_rules_program: &'info AccountInfo<'info>,
}

pub fn transfer<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: TransferArgs,
) -> ProgramResult {
    let authorization_payload = match args {
        TransferArgs::V1 {
            authorization_payload,
        } => authorization_payload,
    };
    let account_info_iter = &mut accounts.iter();

    let token_account = next_account_info(account_info_iter)?;
    let metadata = next_account_info(account_info_iter)?;
    let mint = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;
    let destination_token_account = next_account_info(account_info_iter)?;
    let _destination_owner = next_account_info(account_info_iter)?;
    let spl_token_program = next_account_info(account_info_iter)?;

    let _spl_associated_token_program = next_account_info(account_info_iter)?;
    let _system_program = next_account_info(account_info_iter)?;
    let _sysvar_instructions = next_account_info(account_info_iter)?;

    let _authorization_payload = if authorization_payload.is_some() {
        let authorization_rules = next_account_info(account_info_iter)?;
        let authorization_rules_program = next_account_info(account_info_iter)?;
        Some(AuthorizationPayloadAccounts {
            authorization_rules,
            authorization_rules_program,
        })
    } else {
        None
    };
    // do a bunch of checks and stuff. Not "needed" for PoC.

    // Deserialize metadata to determine its type
    let metadata_data = Metadata::from_account_info(metadata)?;
    // If programmable asset:
    if matches!(
        metadata_data.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        // do auth checks and then potentially transfer
    } else {
        msg!("Transferring SPL token normally");
        let token_transfer_params: TokenTransferParams = TokenTransferParams {
            mint: mint.clone(),
            source: token_account.clone(),
            destination: destination_token_account.clone(),
            amount: 1,
            authority: owner.clone(),
            authority_signer_seeds: None,
            token_program: spl_token_program.clone(),
        };
        mpl_utils::token::spl_token_transfer(token_transfer_params).unwrap();
    }
    Ok(())
}
