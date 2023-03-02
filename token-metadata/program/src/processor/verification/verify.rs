use crate::{
    instruction::{Unverify, Verify, VerifyArgs},
    processor::verification::{
        collection::{unverify_collection_v1, verify_collection_v1},
        creator::{unverify_creator_v1, verify_creator_v1},
    },
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn verify<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: VerifyArgs,
) -> ProgramResult {
    let context = Verify::to_context(accounts)?;

    match args {
        VerifyArgs::CreatorV1 => verify_creator_v1(program_id, context),
        VerifyArgs::CollectionV1 => verify_collection_v1(program_id, context),
    }
}

pub fn unverify<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: VerifyArgs,
) -> ProgramResult {
    let context = Unverify::to_context(accounts)?;

    match args {
        VerifyArgs::CreatorV1 => unverify_creator_v1(program_id, context),
        VerifyArgs::CollectionV1 => unverify_collection_v1(program_id, context),
    }
}
