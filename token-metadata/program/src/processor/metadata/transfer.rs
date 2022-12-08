use mpl_utils::token::TokenTransferParams;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    instruction::TransferArgs,
    state::{Metadata, TokenMetadataAccount, TokenStandard},
};

pub fn transfer<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: TransferArgs,
) -> ProgramResult {
    match args {
        TransferArgs::V1 { .. } => transfer_v1(program_id, accounts, args),
    }
}

fn transfer_v1<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: TransferArgs,
) -> ProgramResult {
    let TransferAccounts::V1 {
        token_account,
        metadata,
        mint,
        owner,
        destination_token_account,
        destination_owner,
        spl_token_program,
        spl_associated_token_program,
        system_program,
        sysvar_instructions,
        authorization_rules,
    } = args.get_accounts(accounts)?;

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

enum TransferAccounts<'a> {
    V1 {
        token_account: &'a AccountInfo<'a>,
        metadata: &'a AccountInfo<'a>,
        mint: &'a AccountInfo<'a>,
        owner: &'a AccountInfo<'a>,
        destination_token_account: &'a AccountInfo<'a>,
        destination_owner: &'a AccountInfo<'a>,
        spl_token_program: &'a AccountInfo<'a>,
        spl_associated_token_program: &'a AccountInfo<'a>,
        system_program: &'a AccountInfo<'a>,
        sysvar_instructions: &'a AccountInfo<'a>,
        authorization_rules: Option<&'a AccountInfo<'a>>,
    },
}

impl TransferArgs {
    fn get_accounts<'a>(
        &self,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<TransferAccounts<'a>, ProgramError> {
        let account_info_iter = &mut accounts.iter().peekable();

        match self {
            TransferArgs::V1 { .. } => {
                let token_account = next_account_info(account_info_iter)?;
                let metadata = next_account_info(account_info_iter)?;
                let mint = next_account_info(account_info_iter)?;
                let owner = next_account_info(account_info_iter)?;
                let destination_token_account = next_account_info(account_info_iter)?;
                let destination_owner = next_account_info(account_info_iter)?;
                let spl_token_program = next_account_info(account_info_iter)?;

                let spl_associated_token_program = next_account_info(account_info_iter)?;
                let system_program = next_account_info(account_info_iter)?;
                let sysvar_instructions = next_account_info(account_info_iter)?;

                // let maybe_next_account = account_info_iter.peek();

                // let authorization_rules = if maybe_next_account.is_some()
                //     && maybe_next_account.unwrap().key == &mpl_token_auth_rules::ID
                // {
                //     let _ = next_account_info(account_info_iter)?;
                //     Some(next_account_info(account_info_iter)?)
                // } else {
                //     None
                // };

                // If the next account is the mpl_token_auth_rules ID, then we consume it
                // and read the next account which will be the authorization rules account.
                let authorization_rules = if account_info_iter
                    .next_if(|a| a.key == &mpl_token_auth_rules::ID)
                    .is_some()
                {
                    // Auth rules account
                    Some(next_account_info(account_info_iter)?)
                } else {
                    None
                };

                Ok(TransferAccounts::V1 {
                    token_account,
                    metadata,
                    mint,
                    owner,
                    destination_token_account,
                    destination_owner,
                    spl_token_program,
                    spl_associated_token_program,
                    system_program,
                    sysvar_instructions,
                    authorization_rules,
                })
            }
        }
    }
}
