use crate::{
    instruction::{Unverify, VerificationArgs, Verify},
    processor::verification::{
        collection::{unverify_collection_v1, verify_collection_v1},
        creator::{unverify_creator_v1, verify_creator_v1},
    },
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn verify<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: VerificationArgs,
) -> ProgramResult {
    let context = Verify::to_context(accounts)?;

    match args {
        VerificationArgs::CreatorV1 => verify_creator_v1(program_id, context),
        VerificationArgs::CollectionV1 => verify_collection_v1(program_id, context),
    }
}

pub fn unverify<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: VerificationArgs,
) -> ProgramResult {
    let context = Unverify::to_context(accounts)?;

    match args {
        VerificationArgs::CreatorV1 => unverify_creator_v1(program_id, context),
        VerificationArgs::CollectionV1 => unverify_collection_v1(program_id, context),
    }
}
