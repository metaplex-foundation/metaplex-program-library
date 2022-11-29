mod bubblegum;
mod burn;
mod collection;
pub(crate) mod deprecated;
mod edition;
pub(crate) mod escrow;
mod freeze;
mod metadata;
mod uses;

use borsh::BorshDeserialize;
pub use bubblegum::*;
// Have to reexport for backwards compatibility
pub use burn::*;
pub use collection::*;
pub use deprecated::{
    process_create_metadata_accounts_v2,
    process_deprecated_mint_new_edition_from_master_edition_via_vault_proxy,
};
pub use edition::*;
pub use escrow::*;
pub use freeze::*;
pub use metadata::*;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
pub use uses::*;

use crate::{
    deprecated_processor::{
        process_deprecated_create_metadata_accounts, process_deprecated_update_metadata_accounts,
    },
    error::MetadataError,
    instruction::MetadataInstruction,
    processor::{
        edition::{
            process_convert_master_edition_v1_to_v2, process_create_master_edition,
            process_mint_new_edition_from_master_edition_via_token,
        },
        escrow::process_transfer_out_of_escrow,
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
