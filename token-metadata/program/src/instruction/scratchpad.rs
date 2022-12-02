// This is a module that isn't used anywhere and shouldn't stay in the final version. It is just a
// "scratchpad" for ideas and stuff.

#[non_exhaustive]
pub struct UpdateAccounts {
    metadata_account: Pubkey,
    master_edition_account: Pubkey,
    mint_account: Pubkey,
    new_update_authority: Option<Pubkey>,
    authority: AuthorityType,
    authorization_rules: Option<AuthorizationRules>,
}

impl UpdateAccounts {
    pub fn to_account_metas(&self) -> Vec<AccountMeta> {
        let mut infos: Vec<AccountMeta> = vec![
            AccountMeta::new(self.metadata_account, false),
            AccountMeta::new(self.master_edition_account, false),
            AccountMeta::new(self.mint_account, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
        ];
        if let Some(new_update_authority) = self.new_update_authority {
            infos.push(AccountMeta::new_readonly(new_update_authority, false));
        }

        match &self.authority {
            AuthorityType::UpdateAuthority(authority) => {
                infos.push(AccountMeta::new_readonly(*authority, true))
            }
            AuthorityType::Holder {
                holder,
                token_account,
            } => {
                infos.push(AccountMeta::new_readonly(*holder, true));
                infos.push(AccountMeta::new_readonly(*token_account, false));
            }
        }
        if let Some(rules) = &self.authorization_rules {
            infos.push(AccountMeta::new_readonly(rules.authorization_rules, false));
            infos.push(AccountMeta::new_readonly(rules.program_id, false));
        }
        infos
    }
}
