use mpl_token_auth_rules::utils::assert_owned_by;
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, system_program, sysvar,
};
use spl_token::state::{Account, Mint};

use crate::{
    assertions::metadata::assert_metadata_valid,
    error::MetadataError,
    instruction::{Context, Migrate, MigrateArgs},
    state::{Metadata, MigrationType, ProgrammableConfig, TokenMetadataAccount, TokenStandard},
    utils::{assert_edition_valid, assert_initialized, clean_write_metadata, thaw},
};

pub fn migrate<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: MigrateArgs,
) -> ProgramResult {
    let context = Migrate::to_context(accounts)?;

    match args {
        MigrateArgs::V1 { .. } => migrate_v1(program_id, context, args),
    }
}

pub fn migrate_v1(program_id: &Pubkey, ctx: Context<Migrate>, args: MigrateArgs) -> ProgramResult {
    let MigrateArgs::V1 {
        migration_type,
        rule_set,
    } = args;

    let payer_info = ctx.accounts.payer_info;
    let authority_info = ctx.accounts.authority_info;
    let metadata_info = ctx.accounts.metadata_info;
    let edition_info = ctx.accounts.edition_info;
    let mint_info = ctx.accounts.mint_info;
    let collection_metadata_info = ctx.accounts.collection_metadata_info;
    let token_info = ctx.accounts.token_info;
    let system_program_info = ctx.accounts.system_program_info;
    let sysvar_instructions_info = ctx.accounts.sysvar_instructions_info;
    let spl_token_program_info = ctx.accounts.spl_token_program_info;

    // Validate Accounts

    // Check signers
    assert_signer(authority_info)?;
    assert_signer(payer_info)?;

    // Assert program ownership
    assert_owned_by(metadata_info, program_id)?;
    assert_owned_by(edition_info, program_id)?;
    assert_owned_by(mint_info, &spl_token::ID)?;
    assert_owned_by(token_info, &spl_token::ID)?;

    // Check program IDs.
    msg!("Check program IDs");
    if spl_token_program_info.key != &spl_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if system_program_info.key != &system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    msg!("Check auth rules program ID");
    if let Some(auth_rules_program) = ctx.accounts.authorization_rules_program_info {
        if auth_rules_program.key != &mpl_token_auth_rules::ID {
            return Err(ProgramError::IncorrectProgramId);
        }
    }

    // Check derivations.
    assert_edition_valid(program_id, mint_info.key, edition_info)?;
    assert_metadata_valid(program_id, mint_info.key, metadata_info)?;

    // Deserialize metadata.
    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;

    let token_standard = metadata.token_standard.ok_or_else(|| {
        <MetadataError as std::convert::Into<ProgramError>>::into(
            MetadataError::InvalidTokenStandard,
        )
    })?;

    // Can only migrate NFT --> PNFT right now.
    // Do we want to check for TokenStandard None?
    if !matches!(token_standard, TokenStandard::NonFungible) {
        return Err(<MetadataError as std::convert::Into<ProgramError>>::into(
            MetadataError::InvalidTokenStandard,
        ));
    }

    match migration_type {
        MigrationType::CollectionV1 => return Err(MetadataError::FeatureNotSupported.into()),
        MigrationType::ProgrammableV1 => {
            // Assertions about state:
            // Is it escrowed?

            let token: Account = assert_initialized(token_info)?;
            let mint: Mint = assert_initialized(mint_info)?;

            if metadata.update_authority != *authority_info.key {
                return Err(MetadataError::UpdateAuthorityIncorrect.into());
            }

            if token.mint != *mint_info.key {
                return Err(MetadataError::MintMismatch.into());
            }

            if mint.freeze_authority.is_none() {
                return Err(MetadataError::NoFreezeAuthoritySet.into());
            }

            if mint.freeze_authority.unwrap() != *edition_info.key {
                return Err(MetadataError::InvalidFreezeAuthority.into());
            }

            // IDK how this would be possible, but if we have the freeze
            // authority and it's the right kind of token we can just
            // unfreeze it before migration.
            if token.is_frozen() {
                thaw(
                    mint_info.clone(),
                    token_info.clone(),
                    edition_info.clone(),
                    spl_token_program_info.clone(),
                )?;
            }

            // Collection checks
            let collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

            // Is it a verified member of the collection?
            if metadata.collection.is_none() {
                return Err(MetadataError::NotAMemberOfCollection.into());
            }
            let collection = metadata.collection.as_ref().unwrap();
            if collection.key != collection_metadata.mint || !collection.verified {
                return Err(MetadataError::NotVerifiedMemberOfCollection.into());
            }

            // Migrate the token.
            metadata.token_standard = Some(TokenStandard::ProgrammableNonFungible);
            if let Some(rule_set) = rule_set {
                metadata.programmable_config = Some(ProgrammableConfig { rule_set });
            }

            clean_write_metadata(&mut metadata, metadata_info)?;
        }
    }

    Ok(())
}
