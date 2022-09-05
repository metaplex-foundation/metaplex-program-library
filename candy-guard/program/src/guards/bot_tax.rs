use solana_program::{
    program::invoke,
    serialize_utils::{read_pubkey, read_u16},
    system_instruction,
    sysvar::instructions::get_instruction_relative,
};

use super::*;
use crate::{errors::CandyGuardError, utils::cmp_pubkeys};

const A_TOKEN: Pubkey = solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BotTax {
    pub lamports: u64,
    pub last_instruction: bool,
}

impl Guard for BotTax {
    fn size() -> usize {
        8 + 1 // u64 + bool
    }

    fn mask() -> u64 {
        0b1u64
    }
}

impl Condition for BotTax {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &[u8],
        _guard_set: &GuardSet,
        _evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        if self.last_instruction {
            let instruction_sysvar_account = &ctx.accounts.instruction_sysvar_account;
            let instruction_sysvar_account_info = instruction_sysvar_account.to_account_info();
            let instruction_sysvar = instruction_sysvar_account_info.data.borrow();
            // the next instruction after the mint
            let next_ix = get_instruction_relative(1, &instruction_sysvar_account_info);

            if let Ok(ix) = next_ix {
                let discriminator = &ix.data[0..8];
                let after_collection_ix =
                    get_instruction_relative(2, &instruction_sysvar_account_info);

                if !cmp_pubkeys(&ix.program_id, &crate::id())
                    || discriminator != [103, 17, 200, 25, 118, 95, 125, 61]
                    || after_collection_ix.is_ok()
                {
                    // we fail here, it is much cheaper to fail here than to allow a malicious user
                    // to add an ix at the end and then fail
                    msg!("Failing and halting due to an extra unauthorized instruction");
                    return err!(CandyGuardError::MintNotLastTransaction);
                }
            }

            let mut idx = 0;
            let num_instructions = read_u16(&mut idx, &instruction_sysvar)
                .map_err(|_| ProgramError::InvalidAccountData)?;

            for index in 0..num_instructions {
                let mut current = 2 + (index * 2) as usize;
                let start = read_u16(&mut current, &instruction_sysvar).unwrap();

                current = start as usize;
                let num_accounts = read_u16(&mut current, &instruction_sysvar).unwrap();
                current += (num_accounts as usize) * (1 + 32);
                let program_id = read_pubkey(&mut current, &instruction_sysvar).unwrap();

                if !cmp_pubkeys(&program_id, &crate::id())
                    && !cmp_pubkeys(&program_id, &::spl_token::id())
                    && !cmp_pubkeys(
                        &program_id,
                        &anchor_lang::solana_program::system_program::ID,
                    )
                    && !cmp_pubkeys(&program_id, &A_TOKEN)
                {
                    msg!("Transaction had ix with program id {}", program_id);
                    return err!(CandyGuardError::MintNotLastTransaction);
                }
            }
        }

        Ok(())
    }
}

impl BotTax {
    pub fn punish_bots<'info>(
        &self,
        error: Error,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
    ) -> Result<()> {
        let bot_account = ctx.accounts.payer.to_account_info();
        let payment_account = ctx.accounts.candy_machine.to_account_info();
        let system_program = ctx.accounts.system_program.to_account_info();

        msg!(
            "{}, Candy Guard Botting is taxed at {:?} lamports",
            error.to_string(),
            self.lamports
        );

        let final_fee = self.lamports.min(bot_account.lamports());
        invoke(
            &system_instruction::transfer(bot_account.key, payment_account.key, final_fee),
            &[bot_account, payment_account, system_program],
        )?;

        Ok(())
    }
}
