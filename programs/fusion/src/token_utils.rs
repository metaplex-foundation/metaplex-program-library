use anchor_lang::prelude::*;

pub fn create_master_token_account<'info>(
  formula_key: &Pubkey,
  item_mint: &Pubkey,
  payer: AccountInfo<'info>,
  master_token_account: AccountInfo<'info>,
  master_token_mint: AccountInfo<'info>,
  output_mint_authority: AccountInfo<'info>,
  token_program: AccountInfo<'info>,
  rent: AccountInfo<'info>,
  system_program: AccountInfo<'info>,
) -> ProgramResult {
  // derive account address from seeds
  let (master_token_acct, master_token_acct_nonce) = Pubkey::find_program_address(
    &[
      &formula_key.to_bytes(),
      &item_mint.to_bytes(),
      b"masterTokenAcct",
    ],
    &super::ID,
  );
  if *master_token_account.key != master_token_acct {
    return Err(super::ErrorCode::MasterTokenAccountMismatch.into());
  }

  // Create account with System program
  let space = anchor_spl::token::TokenAccount::LEN as u64;
  let lamports = Rent::get()?.minimum_balance(space as usize);

  let ix = anchor_lang::solana_program::system_instruction::create_account(
    payer.key,
    master_token_account.key,
    lamports,
    space,
    &anchor_spl::token::ID,
  );

  anchor_lang::solana_program::program::invoke_signed(
    &ix,
    &[payer, master_token_account.clone(), system_program],
    &[&[
      &formula_key.to_bytes(),
      &item_mint.to_bytes(),
      b"masterTokenAcct",
      &[master_token_acct_nonce],
    ]],
  )?;
  // Create the TokenAccount with SPL Token CPI
  let cpi_accounts = anchor_spl::token::InitializeAccount {
    account: master_token_account,
    mint: master_token_mint,
    authority: output_mint_authority,
    rent: rent,
  };
  let cpi_ctx = CpiContext::new(token_program, cpi_accounts);
  anchor_spl::token::initialize_account(cpi_ctx)
}
