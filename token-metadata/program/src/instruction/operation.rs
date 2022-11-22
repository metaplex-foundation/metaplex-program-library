use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
#[cfg(feature = "serde-feature")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

use crate::instruction::MetadataInstruction;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct ApproveUseAuthorityArgs {
    pub number_of_uses: u64,
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
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: MetadataInstruction::ApproveUseAuthority(ApproveUseAuthorityArgs { number_of_uses })
            .try_to_vec()
            .unwrap(),
    }
}

///# Burn Edition NFT
///
/// Burn an Edition NFT, closing its token, metadata and edition accounts, and reducing the Master Edition supply.
///
/// ### Accounts:
///
///   0. [writable] Print NFT Metadata Account
///   1. [writable, signer]</code> Owner of Print NFT
///   2. [writable] Mint of Print Edition NFT
///   3. [writable] Mint of Master Edition NFT
///   4. [writable] Print Edition Token Account
///   5. [writable] Master Edition Token Account
///   6. [writable] Master Edition PDA Account
///   7. [writable] Print Edition PDA Account
///   8. [writable] Edition Marker PDA Account
///   9. [] SPL Token program.
pub fn burn_edition_nft(
    program_id: Pubkey,
    metadata: Pubkey,
    owner: Pubkey,
    print_edition_mint: Pubkey,
    master_edition_mint: Pubkey,
    print_edition_token: Pubkey,
    master_edition_token: Pubkey,
    master_edition: Pubkey,
    print_edition: Pubkey,
    edition_marker: Pubkey,
    spl_token: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(owner, true),
        AccountMeta::new(print_edition_mint, false),
        AccountMeta::new_readonly(master_edition_mint, false),
        AccountMeta::new(print_edition_token, false),
        AccountMeta::new_readonly(master_edition_token, false),
        AccountMeta::new(master_edition, false),
        AccountMeta::new(print_edition, false),
        AccountMeta::new(edition_marker, false),
        AccountMeta::new_readonly(spl_token, false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::BurnEditionNft.try_to_vec().unwrap(),
    }
}

///# Burn NFT
///
/// Burn an NFT, closing its token, metadata and edition accounts.
///
/// 0. `[writable]` NFT metadata
/// 1. `[writable, signer]` Owner of NFT
/// 2. `[writable]` Mint of NFT
/// 3. `[writable]` NFT token account
/// 4. `[writable]` NFT edition account
/// 5. `[]` SPL Token program.
/// 6. Optional `[writable]` Collection metadata account
pub fn burn_nft(
    program_id: Pubkey,
    metadata: Pubkey,
    owner: Pubkey,
    mint: Pubkey,
    token: Pubkey,
    edition: Pubkey,
    spl_token: Pubkey,
    collection_metadata: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(owner, true),
        AccountMeta::new(mint, false),
        AccountMeta::new(token, false),
        AccountMeta::new(edition, false),
        AccountMeta::new_readonly(spl_token, false),
    ];

    if let Some(collection_metadata) = collection_metadata {
        accounts.push(AccountMeta::new(collection_metadata, false));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::BurnNft.try_to_vec().unwrap(),
    }
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
#[allow(clippy::too_many_arguments)]
pub fn freeze_delegated_account(
    program_id: Pubkey,
    delegate: Pubkey,
    token_account: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(delegate, true),
            AccountMeta::new(token_account, false),
            AccountMeta::new_readonly(edition, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: MetadataInstruction::FreezeDelegatedAccount
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
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: MetadataInstruction::RevokeUseAuthority
            .try_to_vec()
            .unwrap(),
    }
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
#[allow(clippy::too_many_arguments)]
pub fn thaw_delegated_account(
    program_id: Pubkey,
    delegate: Pubkey,
    token_account: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(delegate, true),
            AccountMeta::new(token_account, false),
            AccountMeta::new_readonly(edition, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: MetadataInstruction::ThawDelegatedAccount
            .try_to_vec()
            .unwrap(),
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct UtilizeArgs {
    pub number_of_uses: u64,
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
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
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
