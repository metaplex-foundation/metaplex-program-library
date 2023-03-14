use mpl_utils::cmp_pubkeys;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{error::MetadataError, state::ProgrammableConfig};

/// If a programmable rule set is present, then we need:
///   1. authorization rules and data
///   2. edition account
///   3. rule_set passed in by the user to match that stored in the metadata
pub(crate) fn assert_valid_authorization(
    authorization_rules: Option<&AccountInfo>,
    config: &ProgrammableConfig,
) -> ProgramResult {
    let rules = match authorization_rules {
        Some(rules) => rules,
        None => {
            return Err(MetadataError::MissingAuthorizationRules.into());
        }
    };

    if let ProgrammableConfig::V1 {
        rule_set: Some(rule_set),
    } = config
    {
        if cmp_pubkeys(rule_set, rules.key) {
            return Ok(());
        }
    }

    Err(MetadataError::InvalidAuthorizationRules.into())
}
