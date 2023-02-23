use crate::{
    instruction::{Verify, VerifyArgs},
    processor::verification::{
        collection::collection_verification_v1, creator::creator_verification_v1,
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
        VerifyArgs::CreatorV1 => creator_verification_v1(program_id, context, true),
        VerifyArgs::CollectionV1 => collection_verification_v1(program_id, context, true),
    }
}

pub fn unverify<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: VerifyArgs,
) -> ProgramResult {
    let context = Verify::to_context(accounts)?;

    match args {
        VerifyArgs::CreatorV1 => creator_verification_v1(program_id, context, false),
        VerifyArgs::CollectionV1 => collection_verification_v1(program_id, context, false),
    }
}
