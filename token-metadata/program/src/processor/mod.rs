pub(crate) mod deprecated;
mod edition;
mod metadata;
use std::cmp;

use arrayref::array_ref;
use borsh::{BorshDeserialize, BorshSerialize};
pub use edition::*;
pub use metadata::*;
use mpl_utils::{
    assert_signer, create_or_allocate_account_raw,
    token::{
        get_mint_decimals, get_mint_supply, spl_token_burn, spl_token_close, TokenBurnParams,
        TokenCloseParams,
    },
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_memory::sol_memset,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::SysvarId,
};
use spl_token::{
    instruction::{approve, freeze_account, revoke, thaw_account},
    state::{Account, Mint},
};

use crate::{
    assertions::{
        assert_delegated_tokens, assert_derivation, assert_freeze_authority_matches_mint,
        assert_initialized, assert_owned_by,
        collection::{assert_collection_verify_is_valid, assert_has_collection_authority},
        metadata::{assert_currently_holding, assert_verified_member_of_collection},
        uses::{
            assert_burner, assert_use_authority_derivation, assert_valid_bump,
            process_use_authority_validation,
        },
    },
    deprecated_processor::{
        process_deprecated_create_metadata_accounts,
        process_deprecated_mint_new_edition_from_master_edition_via_vault_proxy,
        process_deprecated_update_metadata_accounts,
    },
    deser::clean_write_metadata,
    error::MetadataError,
    escrow::{
        process_close_escrow_account, process_create_escrow_account, process_transfer_out_of_escrow,
    },
    instruction::{MetadataInstruction, SetCollectionSizeArgs},
    processor::edition::{
        process_convert_master_edition_v1_to_v2, process_create_master_edition,
        process_mint_new_edition_from_master_edition_via_token,
    },
    state::{
        Collection, CollectionAuthorityRecord, CollectionDetails, Edition, EditionMarker, Key,
        MasterEditionV2, Metadata, TokenMetadataAccount, UseAuthorityRecord, UseMethod, Uses, BURN,
        COLLECTION_AUTHORITY, COLLECTION_AUTHORITY_RECORD_SIZE, EDITION, EDITION_MARKER_BIT_SIZE,
        MAX_METADATA_LEN, PREFIX, USER, USE_AUTHORITY_RECORD_SIZE,
    },
    utils::{
        decrement_collection_size, increment_collection_size, is_master_edition, is_print_edition,
        BUBBLEGUM_ACTIVATED, BUBBLEGUM_SIGNER,
    },
};

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    let instruction = MetadataInstruction::try_from_slice(input)?;
    match instruction {
        MetadataInstruction::CreateMetadataAccount(args) => {
            msg!("(Deprecated as of 1.1.0) Instruction: Create Metadata Accounts");
            process_deprecated_create_metadata_accounts(
                program_id,
                accounts,
                args.data,
                args.is_mutable,
            )
        }
        MetadataInstruction::UpdateMetadataAccount(args) => {
            msg!("(Deprecated as of 1.1.0) Instruction: Update Metadata Accounts");
            process_deprecated_update_metadata_accounts(
                program_id,
                accounts,
                args.data,
                args.update_authority,
                args.primary_sale_happened,
            )
        }
        MetadataInstruction::CreateMetadataAccountV2(args) => {
            msg!("Instruction: Create Metadata Accounts v2");
            process_create_metadata_accounts_v2(program_id, accounts, args.data, args.is_mutable)
        }
        MetadataInstruction::CreateMetadataAccountV3(args) => {
            msg!("Instruction: Create Metadata Accounts v3");
            process_create_metadata_accounts_v3(
                program_id,
                accounts,
                args.data,
                args.is_mutable,
                args.collection_details,
            )
        }
        MetadataInstruction::UpdateMetadataAccountV2(args) => {
            msg!("Instruction: Update Metadata Accounts v2");
            process_update_metadata_accounts_v2(
                program_id,
                accounts,
                args.data,
                args.update_authority,
                args.primary_sale_happened,
                args.is_mutable,
            )
        }
        MetadataInstruction::DeprecatedCreateMasterEdition(_args) => {
            msg!("Instruction: Deprecated Create Master Edition, Removed in 1.1.0");
            Err(MetadataError::Removed.into())
        }
        MetadataInstruction::DeprecatedMintNewEditionFromMasterEditionViaPrintingToken => {
            msg!("Instruction: Deprecated Mint New Edition from Master Edition Via Token, Removed in 1.1.0");
            Err(MetadataError::Removed.into())
        }
        MetadataInstruction::UpdatePrimarySaleHappenedViaToken => {
            msg!("Instruction: Update primary sale via token");
            process_update_primary_sale_happened_via_token(program_id, accounts)
        }
        MetadataInstruction::DeprecatedSetReservationList(_args) => {
            msg!("Instruction: Deprecated Set Reservation List, Removed in 1.1.0");
            Err(MetadataError::Removed.into())
        }
        MetadataInstruction::DeprecatedCreateReservationList => {
            msg!("Instruction: Deprecated Create Reservation List, Removed in 1.1.0");
            Err(MetadataError::Removed.into())
        }
        MetadataInstruction::SignMetadata => {
            msg!("Instruction: Sign Metadata");
            process_sign_metadata(program_id, accounts)
        }
        MetadataInstruction::RemoveCreatorVerification => {
            msg!("Instruction: Remove Creator Verification");
            process_remove_creator_verification(program_id, accounts)
        }
        MetadataInstruction::DeprecatedMintPrintingTokensViaToken(_args) => {
            msg!("Instruction: Deprecated Mint Printing Tokens Via Token, Removed in 1.1.0");
            Err(MetadataError::Removed.into())
        }
        MetadataInstruction::DeprecatedMintPrintingTokens(_args) => {
            msg!("Instruction: Deprecated Mint Printing Tokens, Removed in 1.1.0");
            Err(MetadataError::Removed.into())
        }
        MetadataInstruction::CreateMasterEdition(args) => {
            msg!("(Deprecated as of 1.1.0, please use V3 Create Master Edition)\n V2 Create Master Edition");
            process_create_master_edition(program_id, accounts, args.max_supply)
        }
        MetadataInstruction::CreateMasterEditionV3(args) => {
            msg!("V3 Create Master Edition");
            process_create_master_edition(program_id, accounts, args.max_supply)
        }
        MetadataInstruction::MintNewEditionFromMasterEditionViaToken(args) => {
            msg!("Instruction: Mint New Edition from Master Edition Via Token");
            process_mint_new_edition_from_master_edition_via_token(
                program_id,
                accounts,
                args.edition,
                false,
            )
        }
        MetadataInstruction::ConvertMasterEditionV1ToV2 => {
            msg!("Instruction: Convert Master Edition V1 to V2");
            process_convert_master_edition_v1_to_v2(program_id, accounts)
        }
        MetadataInstruction::MintNewEditionFromMasterEditionViaVaultProxy(args) => {
            msg!("Instruction: Mint New Edition from Master Edition Via Vault Proxy, deprecated as of 1.4.0.");
            process_deprecated_mint_new_edition_from_master_edition_via_vault_proxy(
                program_id,
                accounts,
                args.edition,
            )
        }
        MetadataInstruction::PuffMetadata => {
            msg!("Instruction: Puff Metadata");
            process_puff_metadata_account(program_id, accounts)
        }
        MetadataInstruction::VerifyCollection => {
            msg!("Instruction: Verify Collection");
            verify_collection(program_id, accounts)
        }
        MetadataInstruction::SetAndVerifyCollection => {
            msg!("Instruction: Set and Verify Collection");
            set_and_verify_collection(program_id, accounts)
        }
        MetadataInstruction::UnverifyCollection => {
            msg!("Instruction: Unverify Collection");
            unverify_collection(program_id, accounts)
        }
        MetadataInstruction::Utilize(args) => {
            msg!("Instruction: Use/Utilize Token");
            process_utilize(program_id, accounts, args.number_of_uses)
        }
        MetadataInstruction::ApproveUseAuthority(args) => {
            msg!("Instruction: Approve Use Authority");
            process_approve_use_authority(program_id, accounts, args.number_of_uses)
        }
        MetadataInstruction::RevokeUseAuthority => {
            msg!("Instruction: Revoke Use Authority");
            process_revoke_use_authority(program_id, accounts)
        }
        MetadataInstruction::ApproveCollectionAuthority => {
            msg!("Instruction: Approve Collection Authority");
            process_approve_collection_authority(program_id, accounts)
        }
        MetadataInstruction::RevokeCollectionAuthority => {
            msg!("Instruction: Revoke Collection Authority");
            process_revoke_collection_authority(program_id, accounts)
        }
        MetadataInstruction::FreezeDelegatedAccount => {
            msg!("Instruction: Freeze Delegated Account");
            process_freeze_delegated_account(program_id, accounts)
        }
        MetadataInstruction::ThawDelegatedAccount => {
            msg!("Instruction: Thaw Delegated Account");
            process_thaw_delegated_account(program_id, accounts)
        }
        MetadataInstruction::BurnNft => {
            msg!("Instruction: Burn NFT");
            process_burn_nft(program_id, accounts)
        }
        MetadataInstruction::BurnEditionNft => {
            msg!("Instruction: Burn Edition NFT");
            process_burn_edition_nft(program_id, accounts)
        }
        MetadataInstruction::VerifySizedCollectionItem => {
            msg!("Instruction: Verify Collection V2");
            verify_sized_collection_item(program_id, accounts)
        }
        MetadataInstruction::SetAndVerifySizedCollectionItem => {
            msg!("Instruction: Set and Verify Collection");
            set_and_verify_sized_collection_item(program_id, accounts)
        }
        MetadataInstruction::UnverifySizedCollectionItem => {
            msg!("Instruction: Unverify Collection");
            unverify_sized_collection_item(program_id, accounts)
        }
        MetadataInstruction::SetCollectionSize(args) => {
            msg!("Instruction: Set Collection Size");
            set_collection_size(program_id, accounts, args)
        }
        MetadataInstruction::SetTokenStandard => {
            msg!("Instruction: Set Token Standard");
            process_set_token_standard(program_id, accounts)
        }
        MetadataInstruction::BubblegumSetCollectionSize(args) => {
            msg!("Instruction: Bubblegum Program Set Collection Size");
            bubblegum_set_collection_size(program_id, accounts, args)
        }
        MetadataInstruction::CreateEscrowAccount => {
            msg!("Instruction: Create Escrow Account");
            process_create_escrow_account(program_id, accounts)
        }
        MetadataInstruction::CloseEscrowAccount => {
            msg!("Instruction: Close Escrow Account");
            process_close_escrow_account(program_id, accounts)
        }
        MetadataInstruction::TransferOutOfEscrow(args) => {
            msg!("Instruction: Transfer Out Of Escrow");
            process_transfer_out_of_escrow(program_id, accounts, args)
        }
    }
}

pub fn verify_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let using_delegated_collection_authority = accounts.len() == 7;
    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::id())?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let collection_metadata = Metadata::from_account_info(collection_info)?;

    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_metadata,
        collection_mint,
        edition_account_info,
    )?;

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            None,
        )?;
    }

    // This handler can only verify non-sized NFTs
    if collection_metadata.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    }

    // If the NFT has collection data, we set it to be verified
    if let Some(collection) = &mut metadata.collection {
        collection.verified = true;
        metadata.serialize(&mut *metadata_info.try_borrow_mut_data()?)?;
    }
    Ok(())
}

pub fn verify_sized_collection_item(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;

    let using_delegated_collection_authority = accounts.len() == 7;

    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::id())?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let mut collection_metadata = Metadata::from_account_info(collection_info)?;

    // Don't verify already verified items, otherwise we end up with invalid size data.
    if let Some(collection) = &metadata.collection {
        if collection.verified {
            return Err(MetadataError::AlreadyVerified.into());
        }
    }

    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_metadata,
        collection_mint,
        edition_account_info,
    )?;

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            None,
        )?;
    }

    // If the NFT has unverified collection data, we set it to be verified and then update the collection
    // size on the Collection Parent.
    if let Some(collection) = &mut metadata.collection {
        msg!("Verifying sized collection item");
        increment_collection_size(&mut collection_metadata, collection_info)?;

        collection.verified = true;
        clean_write_metadata(&mut metadata, metadata_info)?;
    } else {
        return Err(MetadataError::CollectionNotFound.into());
    }
    Ok(())
}

pub fn unverify_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let using_delegated_collection_authority = accounts.len() == 6;

    assert_signer(collection_authority_info)?;
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::id())?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let collection_data = Metadata::from_account_info(collection_info)?;

    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_data,
        collection_mint,
        edition_account_info,
    )?;
    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_data,
            collection_mint.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_data,
            collection_mint.key,
            None,
        )?;
    }

    // This handler can only unverify non-sized NFTs
    if collection_data.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    }

    // If the NFT has collection data, we set it to be unverified and then update the collection
    // size on the Collection Parent.
    if let Some(collection) = &mut metadata.collection {
        collection.verified = false;
    }
    metadata.serialize(&mut *metadata_info.try_borrow_mut_data()?)?;
    Ok(())
}

pub fn unverify_sized_collection_item(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;

    let using_delegated_collection_authority = accounts.len() == 7;

    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::id())?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let mut collection_metadata = Metadata::from_account_info(collection_info)?;

    // Don't unverify already unverified items, otherwise we end up with invalid size data.
    if let Some(collection) = &metadata.collection {
        if !collection.verified {
            return Err(MetadataError::AlreadyUnverified.into());
        }
    }

    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_metadata,
        collection_mint,
        edition_account_info,
    )?;
    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            None,
        )?;
    }

    // If the NFT has collection data, we set it to be unverified and then update the collection
    // size on the Collection Parent.
    if let Some(collection) = &mut metadata.collection {
        decrement_collection_size(&mut collection_metadata, collection_info)?;

        collection.verified = false;
        clean_write_metadata(&mut metadata, metadata_info)?;
    } else {
        return Err(MetadataError::CollectionNotFound.into());
    }
    Ok(())
}

pub fn process_approve_use_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    number_of_uses: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let use_authority_record_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let user_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let program_as_burner = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let metadata: Metadata = Metadata::from_account_info(metadata_info)?;

    if metadata.uses.is_none() {
        return Err(MetadataError::Unusable.into());
    }
    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }
    assert_signer(owner_info)?;
    assert_signer(payer)?;
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_account_info,
    )?;
    let metadata_uses = metadata.uses.unwrap();
    let bump_seed = assert_use_authority_derivation(
        program_id,
        use_authority_record_info,
        user_info,
        mint_info,
    )?;
    let use_authority_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        USER.as_bytes(),
        user_info.key.as_ref(),
        &[bump_seed],
    ];
    process_use_authority_validation(use_authority_record_info.data_len(), true)?;
    create_or_allocate_account_raw(
        *program_id,
        use_authority_record_info,
        system_account_info,
        payer,
        USE_AUTHORITY_RECORD_SIZE,
        use_authority_seeds,
    )?;
    if number_of_uses > metadata_uses.remaining {
        return Err(MetadataError::NotEnoughUses.into());
    }
    if metadata_uses.use_method == UseMethod::Burn {
        assert_burner(program_as_burner.key)?;
        invoke(
            &approve(
                token_program_account_info.key,
                token_account_info.key,
                program_as_burner.key,
                owner_info.key,
                &[],
                1,
            )
            .unwrap(),
            &[
                token_program_account_info.clone(),
                token_account_info.clone(),
                program_as_burner.clone(),
                owner_info.clone(),
            ],
        )?;
    }
    let mutable_data = &mut *use_authority_record_info.try_borrow_mut_data()?;
    let mut record = UseAuthorityRecord::from_bytes(*mutable_data)?;
    record.key = Key::UseAuthorityRecord;
    record.allowed_uses = number_of_uses;
    record.bump = bump_seed;
    record.serialize(mutable_data)?;
    Ok(())
}

pub fn process_revoke_use_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let use_authority_record_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let user_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let metadata = Metadata::from_account_info(metadata_info)?;
    if metadata.uses.is_none() {
        return Err(MetadataError::Unusable.into());
    }
    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }
    assert_signer(owner_info)?;
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_account_info,
    )?;
    let data = &mut use_authority_record_info.try_borrow_mut_data()?;
    process_use_authority_validation(data.len(), false)?;
    assert_owned_by(use_authority_record_info, program_id)?;
    let canonical_bump = assert_use_authority_derivation(
        program_id,
        use_authority_record_info,
        user_info,
        mint_info,
    )?;
    let mut record = UseAuthorityRecord::from_bytes(data)?;
    if record.bump_empty() {
        record.bump = canonical_bump;
    }
    assert_valid_bump(canonical_bump, &record)?;
    let metadata_uses = metadata.uses.unwrap();
    if metadata_uses.use_method == UseMethod::Burn {
        invoke(
            &revoke(
                token_program_account_info.key,
                token_account_info.key,
                owner_info.key,
                &[],
            )
            .unwrap(),
            &[
                token_program_account_info.clone(),
                token_account_info.clone(),
                owner_info.clone(),
            ],
        )?;
    }
    let lamports = use_authority_record_info.lamports();
    **use_authority_record_info.try_borrow_mut_lamports()? = 0;
    **owner_info.try_borrow_mut_lamports()? = owner_info
        .lamports()
        .checked_add(lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;
    sol_memset(data, 0, USE_AUTHORITY_RECORD_SIZE);
    Ok(())
}

pub fn process_utilize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    number_of_uses: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().peekable();
    let metadata_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let user_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    let _ata_program_account_info = next_account_info(account_info_iter)?;
    let _system_account_info = next_account_info(account_info_iter)?;
    // consume the next account only if it is Rent
    let approved_authority_is_using = if account_info_iter
        .next_if(|info| info.key == &Rent::id())
        .is_some()
    {
        // rent was passed in
        accounts.len() == 11
    } else {
        // necessary accounts is one less if rent isn't passed in.
        accounts.len() == 10
    };

    let metadata: Metadata = Metadata::from_account_info(metadata_info)?;
    if metadata.uses.is_none() {
        return Err(MetadataError::Unusable.into());
    }
    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }
    assert_signer(user_info)?;
    assert_currently_holding(
        program_id,
        owner_info,
        metadata_info,
        &metadata,
        mint_info,
        token_account_info,
    )?;
    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let metadata_uses = metadata.uses.unwrap();
    let must_burn = metadata_uses.use_method == UseMethod::Burn;
    if number_of_uses > metadata_uses.total || number_of_uses > metadata_uses.remaining {
        return Err(MetadataError::NotEnoughUses.into());
    }
    let remaining_uses = metadata_uses
        .remaining
        .checked_sub(number_of_uses)
        .ok_or(MetadataError::NotEnoughUses)?;
    metadata.uses = Some(Uses {
        use_method: metadata_uses.use_method,
        total: metadata_uses.total,
        remaining: remaining_uses,
    });
    if approved_authority_is_using {
        let use_authority_record_info = next_account_info(account_info_iter)?;
        let data = &mut *use_authority_record_info.try_borrow_mut_data()?;
        process_use_authority_validation(data.len(), false)?;
        assert_owned_by(use_authority_record_info, program_id)?;
        let canonical_bump = assert_use_authority_derivation(
            program_id,
            use_authority_record_info,
            user_info,
            mint_info,
        )?;
        let mut record = UseAuthorityRecord::from_bytes(data)?;
        // Migrates old UARs to having the bump stored
        if record.bump_empty() {
            record.bump = canonical_bump;
        }
        assert_valid_bump(canonical_bump, &record)?;
        record.allowed_uses = record
            .allowed_uses
            .checked_sub(number_of_uses)
            .ok_or(MetadataError::NotEnoughUses)?;
        record.serialize(data)?;
    } else if user_info.key != owner_info.key {
        return Err(MetadataError::InvalidUser.into());
    }
    metadata.serialize(&mut *metadata_info.try_borrow_mut_data()?)?;
    if remaining_uses == 0 && must_burn {
        if approved_authority_is_using {
            let burn_authority_info = next_account_info(account_info_iter)?;
            let seed = assert_burner(burn_authority_info.key)?;
            let burn_bump_ref = &[
                PREFIX.as_bytes(),
                program_id.as_ref(),
                BURN.as_bytes(),
                &[seed],
            ];
            spl_token_burn(TokenBurnParams {
                mint: mint_info.clone(),
                amount: 1,
                authority: burn_authority_info.clone(),
                token_program: token_program_account_info.clone(),
                source: token_account_info.clone(),
                authority_signer_seeds: Some(burn_bump_ref),
            })?;
        } else {
            spl_token_burn(TokenBurnParams {
                mint: mint_info.clone(),
                amount: 1,
                authority: owner_info.clone(),
                token_program: token_program_account_info.clone(),
                source: token_account_info.clone(),
                authority_signer_seeds: None,
            })?;
        }
    }
    Ok(())
}

pub fn process_approve_collection_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let collection_authority_record = next_account_info(account_info_iter)?;
    let new_collection_authority = next_account_info(account_info_iter)?;
    let update_authority = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    let metadata = Metadata::from_account_info(metadata_info)?;
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    assert_signer(update_authority)?;
    assert_signer(payer)?;
    if metadata.update_authority != *update_authority.key {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }
    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }
    let collection_authority_info_empty = collection_authority_record.try_data_is_empty()?;
    if !collection_authority_info_empty {
        return Err(MetadataError::CollectionAuthorityRecordAlreadyExists.into());
    }
    let collection_authority_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        COLLECTION_AUTHORITY.as_bytes(),
        new_collection_authority.key.as_ref(),
    ]);
    let collection_authority_bump_seed = &[assert_derivation(
        program_id,
        collection_authority_record,
        &collection_authority_path,
    )?];
    let mut collection_authority_seeds = collection_authority_path.clone();
    collection_authority_seeds.push(collection_authority_bump_seed);
    create_or_allocate_account_raw(
        *program_id,
        collection_authority_record,
        system_account_info,
        payer,
        COLLECTION_AUTHORITY_RECORD_SIZE,
        &collection_authority_seeds,
    )?;

    let mut record = CollectionAuthorityRecord::from_account_info(collection_authority_record)?;
    record.key = Key::CollectionAuthorityRecord;
    record.bump = collection_authority_bump_seed[0];
    record.serialize(&mut *collection_authority_record.try_borrow_mut_data()?)?;
    Ok(())
}

pub fn process_revoke_collection_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let collection_authority_record = next_account_info(account_info_iter)?;
    let delegate_authority = next_account_info(account_info_iter)?;
    let revoke_authority = next_account_info(account_info_iter)?;
    let metadata_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let metadata = Metadata::from_account_info(metadata_info)?;
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    assert_signer(revoke_authority)?;
    if metadata.update_authority != *revoke_authority.key
        && *delegate_authority.key != *revoke_authority.key
    {
        return Err(MetadataError::RevokeCollectionAuthoritySignerIncorrect.into());
    }
    if metadata.mint != *mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }
    let collection_authority_info_empty = collection_authority_record.try_data_is_empty()?;
    if collection_authority_info_empty {
        return Err(MetadataError::CollectionAuthorityDoesNotExist.into());
    }
    assert_has_collection_authority(
        delegate_authority,
        &metadata,
        mint_info.key,
        Some(collection_authority_record),
    )?;
    let lamports = collection_authority_record.lamports();
    **collection_authority_record.try_borrow_mut_lamports()? = 0;
    **revoke_authority.try_borrow_mut_lamports()? = revoke_authority
        .lamports()
        .checked_add(lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;
    sol_memset(
        *collection_authority_record.try_borrow_mut_data()?,
        0,
        USE_AUTHORITY_RECORD_SIZE,
    );

    Ok(())
}

pub fn set_and_verify_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let update_authority = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let using_delegated_collection_authority = accounts.len() == 8;
    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::id())?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let collection_data = Metadata::from_account_info(collection_info)?;
    if metadata.update_authority != *update_authority.key
        || metadata.update_authority != collection_data.update_authority
    {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }

    // If it's a verified item and the user is trying to move it to a new collection,
    // they must unverify first, in case it belongs to a sized collection.
    if let Some(collection) = metadata.collection {
        if collection.key != *collection_mint.key && collection.verified {
            return Err(MetadataError::MustUnverify.into());
        }
    }

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_data,
            collection_mint.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_data,
            collection_mint.key,
            None,
        )?;
    }
    metadata.collection = Some(Collection {
        key: *collection_mint.key,
        verified: true,
    });
    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_data,
        collection_mint,
        edition_account_info,
    )?;

    // This handler can only verify non-sized NFTs
    if collection_data.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    }

    metadata.serialize(&mut *metadata_info.try_borrow_mut_data()?)?;
    Ok(())
}

pub fn set_and_verify_sized_collection_item(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let metadata_info = next_account_info(account_info_iter)?;
    let collection_authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let update_authority = next_account_info(account_info_iter)?;
    let collection_mint = next_account_info(account_info_iter)?;
    let collection_info = next_account_info(account_info_iter)?;
    let edition_account_info = next_account_info(account_info_iter)?;
    let using_delegated_collection_authority = accounts.len() == 8;

    assert_signer(collection_authority_info)?;
    assert_signer(payer_info)?;

    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(collection_info, program_id)?;
    assert_owned_by(collection_mint, &spl_token::id())?;
    assert_owned_by(edition_account_info, program_id)?;

    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let mut collection_metadata = Metadata::from_account_info(collection_info)?;

    // Don't verify already verified items, otherwise we end up with invalid size data.
    if let Some(collection) = metadata.collection {
        if collection.verified {
            return Err(MetadataError::MustUnverify.into());
        }
    }

    if metadata.update_authority != *update_authority.key
        || metadata.update_authority != collection_metadata.update_authority
    {
        return Err(MetadataError::UpdateAuthorityIncorrect.into());
    }

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_authority_info,
            &collection_metadata,
            collection_mint.key,
            None,
        )?;
    }
    metadata.collection = Some(Collection {
        key: *collection_mint.key,
        verified: true,
    });
    assert_collection_verify_is_valid(
        &metadata.collection,
        &collection_metadata,
        collection_mint,
        edition_account_info,
    )?;

    // Update the collection size if this is a valid parent collection NFT.
    increment_collection_size(&mut collection_metadata, collection_info)?;

    clean_write_metadata(&mut metadata, metadata_info)?;

    Ok(())
}

pub fn process_freeze_delegated_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let delegate_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;

    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }

    // assert that edition pda is the freeze authority of this mint
    let mint: Mint = assert_initialized(mint_info)?;
    assert_owned_by(edition_info, program_id)?;
    assert_freeze_authority_matches_mint(&mint.freeze_authority, edition_info)?;

    // assert delegate is signer and delegated tokens
    assert_signer(delegate_info)?;
    assert_delegated_tokens(delegate_info, mint_info, token_account_info)?;

    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let edition_info_path_bump_seed = &[assert_derivation(
        program_id,
        edition_info,
        &edition_info_path,
    )?];
    let mut edition_info_seeds = edition_info_path.clone();
    edition_info_seeds.push(edition_info_path_bump_seed);
    invoke_signed(
        &freeze_account(
            token_program_account_info.key,
            token_account_info.key,
            mint_info.key,
            edition_info.key,
            &[],
        )
        .unwrap(),
        &[
            token_account_info.clone(),
            mint_info.clone(),
            edition_info.clone(),
        ],
        &[&edition_info_seeds],
    )?;
    Ok(())
}

pub fn process_thaw_delegated_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let delegate_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_program_account_info = next_account_info(account_info_iter)?;
    if *token_program_account_info.key != spl_token::id() {
        return Err(MetadataError::InvalidTokenProgram.into());
    }

    // assert that edition pda is the freeze authority of this mint
    let mint: Mint = assert_initialized(mint_info)?;
    assert_owned_by(edition_info, program_id)?;
    assert_freeze_authority_matches_mint(&mint.freeze_authority, edition_info)?;

    // assert delegate is signer and delegated tokens
    assert_signer(delegate_info)?;
    assert_delegated_tokens(delegate_info, mint_info, token_account_info)?;

    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let edition_info_path_bump_seed = &[assert_derivation(
        program_id,
        edition_info,
        &edition_info_path,
    )?];
    let mut edition_info_seeds = edition_info_path.clone();
    edition_info_seeds.push(edition_info_path_bump_seed);
    invoke_signed(
        &thaw_account(
            token_program_account_info.key,
            token_account_info.key,
            mint_info.key,
            edition_info.key,
            &[],
        )
        .unwrap(),
        &[
            token_account_info.clone(),
            mint_info.clone(),
            edition_info.clone(),
        ],
        &[&edition_info_seeds],
    )?;
    Ok(())
}

pub fn process_burn_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let token_info = next_account_info(account_info_iter)?;
    let edition_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;

    let collection_nft_provided = accounts.len() == 7;

    let metadata = Metadata::from_account_info(metadata_info)?;

    // If the NFT is a verified part of a collection but the user has not provided the collection
    // metadata account, we cannot burn it because we need to check if we need to decrement the collection size.
    if !collection_nft_provided
        && metadata.collection.is_some()
        && metadata.collection.as_ref().unwrap().verified
    {
        return Err(MetadataError::MissingCollectionMetadata.into());
    }

    // Ensure this is a Master Edition and not a Print.

    // Scope this so the borrow gets dropped and doesn't conflict with the mut borrow
    // later in the handler when overwriting data.
    {
        let edition_account_data = edition_info.try_borrow_data()?;

        // First byte is the object key.
        let key = edition_account_data
            .first()
            .ok_or(MetadataError::InvalidMasterEdition)?;
        if *key != Key::MasterEditionV1 as u8 && *key != Key::MasterEditionV2 as u8 {
            return Err(MetadataError::NotAMasterEdition.into());
        }

        // Next eight bytes are the supply, which must be converted to a u64.
        let supply_bytes = array_ref![edition_account_data, 1, 8];
        let supply = u64::from_le_bytes(*supply_bytes);

        // Cannot burn Master Editions with existing prints in this handler.
        if supply > 0 {
            return Err(MetadataError::MasterEditionHasPrints.into());
        }
    }

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

    // Owned by token-metadata program.
    assert_owned_by(edition_info, program_id)?;

    // Owner is a signer.
    assert_signer(owner_info)?;

    // Has a valid Master Edition or Print Edition.
    let edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    assert_derivation(program_id, edition_info, &edition_info_path)?;

    // Burn the SPL token
    let params = TokenBurnParams {
        mint: mint_info.clone(),
        source: token_info.clone(),
        authority: owner_info.clone(),
        token_program: spl_token_program_info.clone(),
        amount: 1,
        authority_signer_seeds: None,
    };
    spl_token_burn(params)?;

    // Close token account.
    let params = TokenCloseParams {
        token_program: spl_token_program_info.clone(),
        account: token_info.clone(),
        destination: owner_info.clone(),
        owner: owner_info.clone(),
        authority_signer_seeds: None,
    };
    spl_token_close(params)?;

    // Close metadata and edition accounts by transferring rent funds to owner and
    // zeroing out the data.
    let metadata_lamports = metadata_info.lamports();
    **metadata_info.try_borrow_mut_lamports()? = 0;
    **owner_info.try_borrow_mut_lamports()? = owner_info
        .lamports()
        .checked_add(metadata_lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;

    let edition_lamports = edition_info.lamports();
    **edition_info.try_borrow_mut_lamports()? = 0;
    **owner_info.try_borrow_mut_lamports()? = owner_info
        .lamports()
        .checked_add(edition_lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;

    let metadata_data = &mut metadata_info.try_borrow_mut_data()?;
    let edition_data = &mut edition_info.try_borrow_mut_data()?;
    let edition_data_len = edition_data.len();

    // Use MAX_METADATA_LEN because it has unused padding, making it longer than current metadata len.
    sol_memset(metadata_data, 0, MAX_METADATA_LEN);
    sol_memset(edition_data, 0, edition_data_len);

    if collection_nft_provided {
        let collection_metadata_info = next_account_info(account_info_iter)?;

        // Get our collections metadata into a Rust type so we can update the collection size after burning.
        let mut collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

        // Owned by token metadata program.
        assert_owned_by(collection_metadata_info, program_id)?;

        // NFT is actually a verified member of the specified collection.
        assert_verified_member_of_collection(&metadata, &collection_metadata)?;

        // Update collection size if it's sized.
        if let Some(ref details) = collection_metadata.collection_details {
            match details {
                CollectionDetails::V1 { size } => {
                    collection_metadata.collection_details = Some(CollectionDetails::V1 {
                        size: size
                            .checked_sub(1)
                            .ok_or(MetadataError::NumericalOverflowError)?,
                    });
                    clean_write_metadata(&mut collection_metadata, collection_metadata_info)?;
                }
            }
        }
    }

    Ok(())
}

pub fn process_burn_edition_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let metadata_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let print_edition_mint_info = next_account_info(account_info_iter)?;
    let master_edition_mint_info = next_account_info(account_info_iter)?;
    let print_edition_token_info = next_account_info(account_info_iter)?;
    let master_edition_token_info = next_account_info(account_info_iter)?;
    let master_edition_info = next_account_info(account_info_iter)?;
    let print_edition_info = next_account_info(account_info_iter)?;
    let edition_marker_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;

    //    **CHECKS**

    // Ensure the master edition is actually a master edition.
    let master_edition_mint_decimals = get_mint_decimals(master_edition_mint_info)?;
    let master_edition_mint_supply = get_mint_supply(master_edition_mint_info)?;

    if !is_master_edition(
        master_edition_info,
        master_edition_mint_decimals,
        master_edition_mint_supply,
    ) {
        return Err(MetadataError::NotAMasterEdition.into());
    }

    // Ensure the print edition is actually a print edition.
    let print_edition_mint_decimals = get_mint_decimals(print_edition_mint_info)?;
    let print_edition_mint_supply = get_mint_supply(print_edition_mint_info)?;

    if !is_print_edition(
        print_edition_info,
        print_edition_mint_decimals,
        print_edition_mint_supply,
    ) {
        return Err(MetadataError::NotAPrintEdition.into());
    }

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
        print_edition_mint_info,
        print_edition_token_info,
    )?;

    // Owned by token-metadata program.
    assert_owned_by(master_edition_info, program_id)?;
    assert_owned_by(print_edition_info, program_id)?;
    assert_owned_by(edition_marker_info, program_id)?;

    // Owned by spl-token program.
    assert_owned_by(master_edition_mint_info, &spl_token::id())?;
    assert_owned_by(master_edition_token_info, &spl_token::id())?;

    // Master Edition token account checks.
    let master_edition_token_account: Account = assert_initialized(master_edition_token_info)?;

    if master_edition_token_account.mint != *master_edition_mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if master_edition_token_account.amount < 1 {
        return Err(MetadataError::NotEnoughTokens.into());
    }

    // Owner is a signer.
    assert_signer(owner_info)?;

    // Master and Print editions are valid PDAs for their given mints.
    let master_edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        master_edition_mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    assert_derivation(program_id, master_edition_info, &master_edition_info_path)
        .map_err(|_| MetadataError::InvalidMasterEdition)?;

    let print_edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        print_edition_mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    assert_derivation(program_id, print_edition_info, &print_edition_info_path)
        .map_err(|_| MetadataError::InvalidPrintEdition)?;

    let print_edition = Edition::from_account_info(print_edition_info)?;

    // Print Edition actually belongs to the master edition.
    if print_edition.parent != *master_edition_info.key {
        return Err(MetadataError::PrintEditionDoesNotMatchMasterEdition.into());
    }

    // Which edition marker is this edition in
    let edition_marker_number = print_edition
        .edition
        .checked_div(EDITION_MARKER_BIT_SIZE)
        .ok_or(MetadataError::NumericalOverflowError)?;
    let edition_marker_number_str = edition_marker_number.to_string();

    // Ensure we were passed the correct edition marker PDA.
    let edition_marker_info_path = Vec::from([
        PREFIX.as_bytes(),
        program_id.as_ref(),
        master_edition_mint_info.key.as_ref(),
        EDITION.as_bytes(),
        edition_marker_number_str.as_bytes(),
    ]);
    assert_derivation(program_id, edition_marker_info, &edition_marker_info_path)
        .map_err(|_| MetadataError::InvalidEditionMarker)?;

    //      **BURN**
    // Burn the SPL token
    let params = TokenBurnParams {
        mint: print_edition_mint_info.clone(),
        source: print_edition_token_info.clone(),
        authority: owner_info.clone(),
        token_program: spl_token_program_info.clone(),
        amount: 1,
        authority_signer_seeds: None,
    };
    spl_token_burn(params)?;

    // Close token account.
    let params = TokenCloseParams {
        token_program: spl_token_program_info.clone(),
        account: print_edition_token_info.clone(),
        destination: owner_info.clone(),
        owner: owner_info.clone(),
        authority_signer_seeds: None,
    };
    spl_token_close(params)?;

    // Close metadata and edition accounts by transferring rent funds to owner and
    // zeroing out the data.
    let metadata_lamports = metadata_info.lamports();
    **metadata_info.try_borrow_mut_lamports()? = 0;
    **owner_info.try_borrow_mut_lamports()? = owner_info
        .lamports()
        .checked_add(metadata_lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;

    let edition_lamports = print_edition_info.lamports();
    **print_edition_info.try_borrow_mut_lamports()? = 0;
    **owner_info.try_borrow_mut_lamports()? = owner_info
        .lamports()
        .checked_add(edition_lamports)
        .ok_or(MetadataError::NumericalOverflowError)?;

    let metadata_data = &mut metadata_info.try_borrow_mut_data()?;
    let edition_data = &mut print_edition_info.try_borrow_mut_data()?;
    let edition_data_len = edition_data.len();

    // Use MAX_METADATA_LEN because it has unused padding, making it longer than current metadata len.
    sol_memset(metadata_data, 0, MAX_METADATA_LEN);
    sol_memset(edition_data, 0, edition_data_len);

    //       **EDITION HOUSEKEEPING**
    // Set the particular bit for this edition to 0 to allow reprinting,
    // IF the print edition owner is also the master edition owner.
    // Otherwise leave the bit set to 1 to disallow reprinting.
    let mut edition_marker: EditionMarker = EditionMarker::from_account_info(edition_marker_info)?;
    let owner_is_the_same = *owner_info.key == master_edition_token_account.owner;

    if owner_is_the_same {
        let (index, mask) = EditionMarker::get_index_and_mask(print_edition.edition)?;
        edition_marker.ledger[index] ^= mask;
    }

    // If the entire edition marker is empty, then we can close the account.
    // Otherwise, serialize the new edition marker and update the account data.
    if edition_marker.ledger.iter().all(|i| *i == 0) {
        let edition_marker_lamports = edition_marker_info.lamports();
        **edition_marker_info.try_borrow_mut_lamports()? = 0;
        **owner_info.try_borrow_mut_lamports()? = owner_info
            .lamports()
            .checked_add(edition_marker_lamports)
            .ok_or(MetadataError::NumericalOverflowError)?;

        let edition_marker_data = &mut edition_marker_info.try_borrow_mut_data()?;
        let edition_marker_data_len = edition_marker_data.len();

        sol_memset(edition_marker_data, 0, edition_marker_data_len);
    } else {
        let mut edition_marker_info_data = edition_marker_info.try_borrow_mut_data()?;
        edition_marker_info_data[0..].fill(0);
        edition_marker.serialize(&mut *edition_marker_info_data)?;
    }

    // Decrement the suppply on the master edition now that we've successfully burned a print.
    // Decrement max_supply if Master Edition owner is not the same as Print Edition owner.
    let mut master_edition: MasterEditionV2 =
        MasterEditionV2::from_account_info(master_edition_info)?;
    master_edition.supply = master_edition
        .supply
        .checked_sub(1)
        .ok_or(MetadataError::NumericalOverflowError)?;

    if let Some(max_supply) = master_edition.max_supply {
        if !owner_is_the_same {
            master_edition.max_supply = Some(
                max_supply
                    .checked_sub(1)
                    .ok_or(MetadataError::NumericalOverflowError)?,
            );
        }
    }
    master_edition.serialize(&mut *master_edition_info.try_borrow_mut_data()?)?;

    Ok(())
}

pub fn set_collection_size(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetCollectionSizeArgs,
) -> ProgramResult {
    let size = args.size;

    let account_info_iter = &mut accounts.iter();

    let parent_nft_metadata_account_info = next_account_info(account_info_iter)?;
    let collection_update_authority_account_info = next_account_info(account_info_iter)?;
    let collection_mint_account_info = next_account_info(account_info_iter)?;

    let using_delegated_collection_authority = accounts.len() == 4;

    // Owned by token-metadata program.
    assert_owned_by(parent_nft_metadata_account_info, program_id)?;

    // Mint owned by spl token program.
    assert_owned_by(collection_mint_account_info, &spl_token::id())?;

    let mut metadata = Metadata::from_account_info(parent_nft_metadata_account_info)?;

    // Check that the update authority or delegate is a signer.
    if !collection_update_authority_account_info.is_signer {
        return Err(MetadataError::UpdateAuthorityIsNotSigner.into());
    }

    if using_delegated_collection_authority {
        let collection_authority_record = next_account_info(account_info_iter)?;
        assert_has_collection_authority(
            collection_update_authority_account_info,
            &metadata,
            collection_mint_account_info.key,
            Some(collection_authority_record),
        )?;
    } else {
        assert_has_collection_authority(
            collection_update_authority_account_info,
            &metadata,
            collection_mint_account_info.key,
            None,
        )?;
    }

    // Only unsized collections can have the size set, and only once.
    if metadata.collection_details.is_some() {
        return Err(MetadataError::SizedCollection.into());
    } else {
        metadata.collection_details = Some(CollectionDetails::V1 { size });
    }

    clean_write_metadata(&mut metadata, parent_nft_metadata_account_info)?;
    Ok(())
}

pub fn bubblegum_set_collection_size(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetCollectionSizeArgs,
) -> ProgramResult {
    let size = args.size;

    let account_info_iter = &mut accounts.iter();

    let parent_nft_metadata_account_info = next_account_info(account_info_iter)?;
    let collection_update_authority_account_info = next_account_info(account_info_iter)?;
    let collection_mint_account_info = next_account_info(account_info_iter)?;
    let bubblegum_signer_info = next_account_info(account_info_iter)?;

    let delegated_collection_auth_opt = if accounts.len() == 5 {
        Some(next_account_info(account_info_iter)?)
    } else {
        None
    };

    if !BUBBLEGUM_ACTIVATED {
        return Err(MetadataError::InvalidOperation.into());
    }

    // This instruction can only be called by the Bubblegum program.
    if *bubblegum_signer_info.key != BUBBLEGUM_SIGNER {
        return Err(MetadataError::InvalidBubblegumSigner.into());
    }
    assert_signer(bubblegum_signer_info)?;

    // Owned by token-metadata program.
    assert_owned_by(parent_nft_metadata_account_info, program_id)?;

    // Mint owned by spl token program.
    assert_owned_by(collection_mint_account_info, &spl_token::id())?;

    let mut metadata = Metadata::from_account_info(parent_nft_metadata_account_info)?;

    // Check that the update authority or delegate is a signer.
    if !collection_update_authority_account_info.is_signer {
        return Err(MetadataError::UpdateAuthorityIsNotSigner.into());
    }

    assert_has_collection_authority(
        collection_update_authority_account_info,
        &metadata,
        collection_mint_account_info.key,
        delegated_collection_auth_opt,
    )?;

    // Ensure new size is + or - 1 of the current size.
    let current_size = if let Some(details) = metadata.collection_details {
        match details {
            CollectionDetails::V1 { size } => size,
        }
    } else {
        return Err(MetadataError::NotACollectionParent.into());
    };

    let diff = cmp::max(current_size, size)
        .checked_sub(cmp::min(current_size, size))
        .ok_or(MetadataError::InvalidCollectionSizeChange)?;

    if diff != 1 {
        return Err(MetadataError::InvalidCollectionSizeChange.into());
    }

    // The Bubblegum program has authority to manage the collection details.
    metadata.collection_details = Some(CollectionDetails::V1 { size });

    clean_write_metadata(&mut metadata, parent_nft_metadata_account_info)?;
    Ok(())
}
