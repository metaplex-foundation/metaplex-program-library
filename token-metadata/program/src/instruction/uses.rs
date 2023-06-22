use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::{instruction::MetadataInstruction, processor::AuthorizationData};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct ApproveUseAuthorityArgs {
    pub number_of_uses: u64,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct UtilizeArgs {
    pub number_of_uses: u64,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UseArgs {
    V1 {
        /// Required authorization data to validate the request.
        authorization_data: Option<AuthorizationData>,
    },
}

///# Approve Use Authority
///
///Approve another account to call [utilize] on this NFT
///
///### Args:
///
///See: [ApproveUseAuthorityArgs]
///
///### Accounts:
///
///   0. `[writable]` Use Authority Record PDA
///   1. `[writable]` Owned Token Account Of Mint
///   2. `[signer]` Owner
///   3. `[signer]` Payer
///   4. `[]` A Use Authority
///   5. `[]` Metadata account
///   6. `[]` Mint of Metadata
///   7. `[]` Program As Signer (Burner)
///   8. `[]` Token program
///   9. `[]` System program
///   10. Optional `[]` Rent info
#[allow(clippy::too_many_arguments)]
pub fn approve_use_authority(
    program_id: Pubkey,
    use_authority_record: Pubkey,
    user: Pubkey,
    owner: Pubkey,
    payer: Pubkey,
    owner_token_account: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
    burner: Pubkey,
    number_of_uses: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(use_authority_record, false),
            AccountMeta::new(owner, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(user, false),
            AccountMeta::new(owner_token_account, false),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(burner, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: MetadataInstruction::ApproveUseAuthority(ApproveUseAuthorityArgs { number_of_uses })
            .try_to_vec()
            .unwrap(),
    }
}

//# Revoke Use Authority
///
///Revoke account to call [utilize] on this NFT
///
///### Accounts:
///
///   0. `[writable]` Use Authority Record PDA
///   1. `[writable]` Owned Token Account Of Mint
///   2. `[signer]` Owner
///   3. `[signer]` Payer
///   4. `[]` A Use Authority
///   5. `[]` Metadata account
///   6. `[]` Mint of Metadata
///   7. `[]` Token program
///   8. `[]` System program
///   9. Optional `[]` Rent info
#[allow(clippy::too_many_arguments)]
pub fn revoke_use_authority(
    program_id: Pubkey,
    use_authority_record: Pubkey,
    user: Pubkey,
    owner: Pubkey,
    owner_token_account: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(use_authority_record, false),
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(user, false),
            AccountMeta::new(owner_token_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: MetadataInstruction::RevokeUseAuthority
            .try_to_vec()
            .unwrap(),
    }
}

///# Utilize
///
///Utilize or Use an NFT , burns the NFT and returns the lamports to the update authority if the use method is burn and its out of uses.
///Use Authority can be the Holder of the NFT, or a Delegated Use Authority.
///
///### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[writable]` Token Account Of NFT
///   2. `[writable]` Mint of the Metadata
///   2. `[signer]` A Use Authority / Can be the current Owner of the NFT
///   3. `[signer]` Payer
///   4. `[]` Owner
///   5. `[]` Token program
///   6. `[]` Associated Token program
///   7. `[]` System program
///   8. Optional `[]` Rent info
///   9. Optional `[writable]` Use Authority Record PDA If present the program Assumes a delegated use authority
#[allow(clippy::too_many_arguments)]
pub fn utilize(
    program_id: Pubkey,
    metadata: Pubkey,
    token_account: Pubkey,
    mint: Pubkey,
    use_authority_record_pda: Option<Pubkey>,
    use_authority: Pubkey,
    owner: Pubkey,
    burner: Option<Pubkey>,
    number_of_uses: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(token_account, false),
        AccountMeta::new(mint, false),
        AccountMeta::new(use_authority, true),
        AccountMeta::new_readonly(owner, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    if let Some(use_authority_record_pda) = use_authority_record_pda {
        accounts.push(AccountMeta::new(use_authority_record_pda, false));
    }

    if let Some(burner) = burner {
        accounts.push(AccountMeta::new_readonly(burner, false));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::Utilize(UtilizeArgs { number_of_uses })
            .try_to_vec()
            .unwrap(),
    }
}
