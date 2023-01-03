use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::instruction::MetadataInstruction;

pub fn freeze_delegated_account(
    program_id: Pubkey,
    delegate: Pubkey,
    token_account: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
) -> Instruction {
    freeze_delegated_account_with_token_program(
        program_id,
        delegate,
        token_account,
        edition,
        mint,
        spl_token::id(),
    )
}

///# Freeze delegated account
///
///Allow freezing of an NFT if this user is the delegate of the NFT
///
///### Accounts:
///   0. `[signer]` Delegate
///   1. `[writable]` Token account to freeze
///   2. `[]` Edition
///   3. `[]` Token mint
///   4. `[]` Token program
pub fn freeze_delegated_account_with_token_program(
    program_id: Pubkey,
    delegate: Pubkey,
    token_account: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
    token_program_id: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(delegate, true),
            AccountMeta::new(token_account, false),
            AccountMeta::new_readonly(edition, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(token_program_id, false),
        ],
        data: MetadataInstruction::FreezeDelegatedAccount
            .try_to_vec()
            .unwrap(),
    }
}

pub fn thaw_delegated_account(
    program_id: Pubkey,
    delegate: Pubkey,
    token_account: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
) -> Instruction {
    thaw_delegated_account_with_token_program(
        program_id,
        delegate,
        token_account,
        edition,
        mint,
        spl_token::id(),
    )
}

///# Thaw delegated account
///
///Allow thawing of an NFT if this user is the delegate of the NFT
///
///### Accounts:
///   0. `[signer]` Delegate
///   1. `[writable]` Token account to thaw
///   2. `[]` Edition
///   3. `[]` Token mint
///   4. `[]` Token program
pub fn thaw_delegated_account_with_token_program(
    program_id: Pubkey,
    delegate: Pubkey,
    token_account: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
    token_program_id: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(delegate, true),
            AccountMeta::new(token_account, false),
            AccountMeta::new_readonly(edition, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(token_program_id, false),
        ],
        data: MetadataInstruction::ThawDelegatedAccount
            .try_to_vec()
            .unwrap(),
    }
}
