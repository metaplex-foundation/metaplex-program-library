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
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
            AccountMeta::new_readonly(sysvar::instructions::ID, false),
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

pub struct AuthorizationRules {
    authorization_rules: Pubkey,
    program_id: Pubkey,
}

// #[non_exhaustive]
// pub struct TransferAccounts<'info> {
//     token_account: &'info AccountInfo<'info>,
//     metadata: &'info AccountInfo<'info>,
//     mint: &'info AccountInfo<'info>,
//     owner: &'info AccountInfo<'info>,
//     destination_token_account: &'info AccountInfo<'info>,
//     destination_owner: &'info AccountInfo<'info>,
//     spl_token_program: &'info AccountInfo<'info>,
//     spl_associated_token_program: &'info AccountInfo<'info>,
//     system_program: &'info AccountInfo<'info>,
//     sysvar_instructions: &'info AccountInfo<'info>,
//     authorization_payload: Option<AuthorizationPayloadAccounts<'info>>,
// }
//
// impl<'info> TransferAccounts<'info> {
//     pub fn new(
//         token_account: &'info AccountInfo<'info>,
//         metadata: &'info AccountInfo<'info>,
//         mint: &'info AccountInfo<'info>,
//         owner: &'info AccountInfo<'info>,
//         destination_token_account: &'info AccountInfo<'info>,
//         destination_owner: &'info AccountInfo<'info>,
//         spl_token_program: &'info AccountInfo<'info>,
//         spl_associated_token_program: &'info AccountInfo<'info>,
//         system_program: &'info AccountInfo<'info>,
//         sysvar_instructions: &'info AccountInfo<'info>,
//         authorization_payload: Option<AuthorizationPayloadAccounts<'info>>,
//     ) -> Self {
//         Self {
//             token_account,
//             metadata,
//             mint,
//             owner,
//             destination_token_account,
//             destination_owner,
//             spl_token_program,
//             spl_associated_token_program,
//             system_program,
//             sysvar_instructions,
//             authorization_payload,
//         }
//     }
//
//     pub fn from_args<'a>(
//         args: TransferArgs,
//         accounts: &'a [AccountInfo<'a>],
//     ) -> Result<Self, ProgramError> {
//         // match args {
//         // TransferArgs::V1 {
//         //     authorization_payload,
//         // } => {
//         //     let token_account = accounts.get(0);
//         //     let metadata = next_account_info(account_info_iter)?;
//         //     let mint = next_account_info(account_info_iter)?;
//         //     let owner = next_account_info(account_info_iter)?;
//         //     let destination_token_account = next_account_info(account_info_iter)?;
//         //     let destination_owner = next_account_info(account_info_iter)?;
//         //     let spl_token_program = next_account_info(account_info_iter)?;
//         //     let spl_associated_token_program = next_account_info(account_info_iter)?;
//         //     let system_program = next_account_info(account_info_iter)?;
//         //     let sysvar_instructions = next_account_info(account_info_iter)?;
//         //     let authorization_payload = if authorization_payload.is_some() {
//         //         let authorization_rules = next_account_info(account_info_iter)?;
//         //         let authorization_rules_program = next_account_info(account_info_iter)?;
//         //         Some(AuthorizationPayloadAccounts {
//         //             authorization_rules,
//         //             authorization_rules_program,
//         //         })
//         //     } else {
//         //         None
//         //     };
//         //     asso
//         //     Ok(Self {
//         //         token_account,
//         //         metadata,
//         //         mint,
//         //         owner,
//         //         destination_token_account,
//         //         destination_owner,
//         //         spl_token_program,
//         //         spl_associated_token_program,
//         //         system_program,
//         //         sysvar_instructions,
//         //         authorization_payload,
//         //     })
//         // }
//         // }
//         todo!()
//     }
//
//     pub fn validate(&self) -> ProgramResult {
//         todo!()
//     }
// }
