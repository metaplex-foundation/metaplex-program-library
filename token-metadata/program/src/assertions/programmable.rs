use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MetadataError, state::ProgrammableConfig};

/// If a programmable rule set is present, then we need:
///   1. authorization rules and data
///   2. edition account
///   3. rule_set passed in by the user to match that stored in the metadata
pub(crate) fn assert_valid_authorization<'info>(
    authorization_rules: Option<&AccountInfo<'info>>,
    config: &ProgrammableConfig,
) -> ProgramResult {
    // Only check if the metadata has a programmable config set.
    if authorization_rules.is_none() {
        return Err(MetadataError::MissingAuthorizationRules.into());
    }
    let rules = authorization_rules.unwrap();

    if let Some(rule_set) = config.rule_set {
        if rule_set != *rules.key {
            return Err(MetadataError::InvalidAuthorizationRules.into());
        }
    }

    Ok(())
}
