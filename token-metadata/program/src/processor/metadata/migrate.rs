use mpl_token_auth_rules::utils::assert_owned_by;
use mpl_utils::{assert_signer, create_or_allocate_account_raw};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_option::COption, pubkey, pubkey::Pubkey, system_program, sysvar,
};
use spl_token::state::{Account, Mint};

use crate::{
    assertions::{
        collection::assert_is_collection_delegated_authority, metadata::assert_metadata_valid,
    },
    error::MetadataError,
    instruction::{Context, Migrate, MigrateArgs},
    pda::PREFIX,
    state::{
        CollectionAuthorityRecord, Metadata, MigrationType, ProgrammableConfig, Resizable,
        TokenDelegateRole, TokenMetadataAccount, TokenRecord, TokenStandard, TokenState,
        TOKEN_RECORD_SEED, TOKEN_STANDARD_INDEX,
    },
    utils::{
        assert_derivation, assert_edition_valid, assert_initialized, clean_write_metadata, freeze,
    },
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
    let token_record_info = ctx.accounts.token_record_info;
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
    if spl_token_program_info.key != &spl_token::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if system_program_info.key != &system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if sysvar_instructions_info.key != &sysvar::instructions::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if let Some(auth_rules_program) = ctx.accounts.authorization_rules_program_info {
        if auth_rules_program.key != &mpl_token_auth_rules::ID {
            return Err(ProgramError::IncorrectProgramId);
        }
    }

    // Check derivations.
    assert_edition_valid(program_id, mint_info.key, edition_info)?;
    assert_metadata_valid(program_id, mint_info.key, metadata_info)?;

    // Deserialize metadata.
    let mut metadata = Metadata::from_account_info(metadata_info)?;
    let collection_metadata = Metadata::from_account_info(collection_metadata_info)?;

    match migration_type {
        MigrationType::CollectionV1 => return Err(MetadataError::FeatureNotSupported.into()),
        MigrationType::ProgrammableV1 => {
            // NFT --> PNFT migration requires the mpl-migration-validator
            // to CPI into Token Metadata using its program signer key.
            // In addition, the collection must have a delegate record set
            // with the mpl-migration-validator program signer as the delegate.
            let migration_validator_signer =
                pubkey!("4fDQAj27ahBfXw3ZQumg5gJrMRUCzPUW6RxrRPFMC8Av");

            if *authority_info.key != migration_validator_signer {
                return Err(MetadataError::UpdateAuthorityIncorrect.into());
            }

            assert_is_collection_delegated_authority(
                ctx.accounts.delegate_record_info,
                &migration_validator_signer,
                &collection_metadata.mint,
            )?;

            let delegate_record =
                CollectionAuthorityRecord::from_account_info(ctx.accounts.delegate_record_info)?;

            if delegate_record.update_authority != Some(collection_metadata.update_authority) {
                return Err(MetadataError::UpdateAuthorityIncorrect.into());
            }

            let token: Account = assert_initialized(token_info)?;
            let mint: Mint = assert_initialized(mint_info)?;

            if token.mint != *mint_info.key {
                return Err(MetadataError::MintMismatch.into());
            }

            if mint.freeze_authority.is_none() {
                return Err(MetadataError::NoFreezeAuthoritySet.into());
            }

            if mint.freeze_authority.unwrap() != *edition_info.key {
                return Err(MetadataError::InvalidFreezeAuthority.into());
            }

            // NFT --> PNFT migration must maintain the current level of functionality
            // that the token has, but all pNFTs must be frozen. To accomplish this,
            // we assign Migration delegate to any pNFTs that have a SPL token delegate
            // set. This allows the delegate to freeze the token via the Token Metadata
            // Lock abstraction, as well as to transfer it as a normal SPL delegate is
            // able to.
            //
            // Unfrozen tokens are frozen. Already frozen tokens will have the Lock
            // flag set to match the current state in the new abstraction.

            // We create the token record if it does not exist,
            // but we only serialize once at the end to save on compute.

            let mut token_record = TokenRecord::default();

            // We check the derivation regardless of whether the account exists.
            let mut signer_seeds = Vec::from([
                PREFIX.as_bytes(),
                crate::ID.as_ref(),
                mint_info.key.as_ref(),
                TOKEN_RECORD_SEED.as_bytes(),
                token_info.key.as_ref(),
            ]);

            let bump = &[assert_derivation(
                program_id,
                token_record_info,
                &signer_seeds,
            )?];
            signer_seeds.push(bump);

            if ctx.accounts.token_record_info.data.borrow().is_empty() {
                // allocate the delegate account
                create_or_allocate_account_raw(
                    *program_id,
                    token_record_info,
                    system_program_info,
                    payer_info,
                    TokenRecord::size(),
                    &signer_seeds,
                )?;

                token_record.bump = bump[0];
            }

            // Only freeze if the token is not already frozen, otherwise the call will fail.
            // If the token is frozen already AND it has a SPL delegate set, then we
            // set the state to Locked.
            if !token.is_frozen() {
                freeze(
                    mint_info.clone(),
                    token_info.clone(),
                    edition_info.clone(),
                    spl_token_program_info.clone(),
                )?;
            } else if token.delegate.is_some() {
                token_record.state = TokenState::Locked;
            }

            // Set Migration delegate if SPL delegate is set.
            if let COption::Some(current_delegate) = token.delegate {
                token_record.delegate = Some(current_delegate);
                token_record.delegate_role = Some(TokenDelegateRole::Migration);
            }

            token_record.save(
                ctx.accounts.token_record_info,
                payer_info,
                system_program_info,
            )?;

            // Migrate the token.
            metadata.token_standard = Some(TokenStandard::ProgrammableNonFungible);
            metadata.programmable_config = Some(ProgrammableConfig::V1 { rule_set });
            edition_info.data.borrow_mut()[TOKEN_STANDARD_INDEX] =
                TokenStandard::ProgrammableNonFungible as u8;

            clean_write_metadata(&mut metadata, metadata_info)?;
        }
    }

    Ok(())
}
