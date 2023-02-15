use crate::error::HydraError;
use crate::state::{Fanout, MembershipModel};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program_memory::sol_memcmp;
use anchor_lang::solana_program::pubkey::PUBKEY_BYTES;
use anchor_spl::token::TokenAccount;
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
    error: Option<error::Error>,
) -> Result<u8> {
    let (key, bump) = Pubkey::find_program_address(path, program_id);
    if !cmp_pubkeys(&key, account.key) {
        if let Some(err) = error {
            msg!("Derivation {:?}", err);
            return Err(err);
        }
        msg!("DerivedKeyInvalid");
        return Err(HydraError::DerivedKeyInvalid.into());
    }
    Ok(bump)
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<()> {
    if !cmp_pubkeys(account.owner, owner) {
        Err(HydraError::IncorrectOwner.into())
    } else {
        Ok(())
    }
}

pub fn assert_membership_model(fanout: &Account<Fanout>, model: MembershipModel) -> Result<()> {
    if fanout.membership_model != model {
        return Err(HydraError::InvalidMembershipModel.into());
    }
    Ok(())
}

pub fn assert_ata(
    account: &AccountInfo,
    target: &Pubkey,
    mint: &Pubkey,
    err: Option<error::Error>,
) -> Result<u8> {
    assert_derivation(
        &anchor_spl::associated_token::ID,
        &account.to_account_info(),
        &[
            target.as_ref(),
            anchor_spl::token::ID.as_ref(),
            mint.as_ref(),
        ],
        err,
    )
}

pub fn assert_shares_distributed(fanout: &Account<Fanout>) -> Result<()> {
    if fanout.total_available_shares != 0 {
        return Err(HydraError::SharesArentAtMax.into());
    }
    Ok(())
}

pub fn assert_holding(
    owner: &AccountInfo,
    token_account: &Account<TokenAccount>,
    mint_info: &AccountInfo,
) -> Result<()> {
    assert_owned_by(mint_info, &spl_token::id())?;
    let token_account_info = token_account.to_account_info();
    assert_owned_by(&token_account_info, &spl_token::id())?;
    if !cmp_pubkeys(&token_account.owner, owner.key) {
        return Err(HydraError::IncorrectOwner.into());
    }
    if token_account.amount < 1 {
        return Err(HydraError::WalletDoesNotOwnMembershipToken.into());
    }
    if !cmp_pubkeys(&token_account.mint, &mint_info.key()) {
        return Err(HydraError::MintDoesNotMatch.into());
    }
    Ok(())
}

pub fn assert_distributed(
    ix: Instruction,
    subject: &Pubkey,
    membership_model: MembershipModel,
) -> Result<()> {
    if !cmp_pubkeys(&ix.program_id, &crate::id()) {
        return Err(HydraError::MustDistribute.into());
    }
    let instruction_id = match membership_model {
        MembershipModel::Wallet => [252, 168, 167, 66, 40, 201, 182, 163],
        MembershipModel::NFT => [108, 240, 68, 81, 144, 83, 58, 153],
        MembershipModel::Token => [126, 105, 46, 135, 28, 36, 117, 212],
    };
    if sol_memcmp(instruction_id.as_ref(), ix.data[0..8].as_ref(), 8) != 0 {
        return Err(HydraError::MustDistribute.into());
    }
    if !cmp_pubkeys(subject, &ix.accounts[1].pubkey) {
        return Err(HydraError::MustDistribute.into());
    }
    Ok(())
}

pub fn assert_valid_metadata(
    metadata_account: &AccountInfo,
    mint: &AccountInfo,
) -> Result<Metadata> {
    let meta = Metadata::from_account_info(metadata_account)?;
    if !cmp_pubkeys(&meta.mint, mint.key) {
        return Err(HydraError::InvalidMetadata.into());
    }
    Ok(meta)
}

pub fn assert_owned_by_one(account: &AccountInfo, owners: Vec<&Pubkey>) -> Result<()> {
    for o in owners {
        let res = assert_owned_by(account, o);
        if res.is_ok() {
            return res;
        }
    }
    Err(HydraError::IncorrectOwner.into())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_multi_owner_check() {
        let owner = Pubkey::new_unique();
        let owner1 = Pubkey::new_unique();
        let owner2 = Pubkey::new_unique();
        let ad = Pubkey::new_unique();
        let actual_owner = Pubkey::new_unique();
        let lam = &mut 10000;
        let a = AccountInfo::new(&ad, false, false, lam, &mut [0; 0], &actual_owner, false, 0);

        let e = assert_owned_by_one(&a, vec![&owner, &owner2, &owner1]);

        assert!(e.is_err());

        let e = assert_owned_by_one(&a, vec![&owner, &actual_owner, &owner1]);

        assert!(e.is_ok());
    }
}
