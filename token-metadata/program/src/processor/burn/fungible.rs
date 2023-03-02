use super::*;

pub(crate) fn burn_fungible(ctx: &Context<Burn>, amount: u64) -> ProgramResult {
    let token = TokenAccount::unpack(&ctx.accounts.token_info.data.borrow())?;

    if amount > token.amount {
        return Err(MetadataError::InsufficientTokenBalance.into());
    }

    // Burn the SPL tokens
    let params = TokenBurnParams {
        mint: ctx.accounts.mint_info.clone(),
        source: ctx.accounts.token_info.clone(),
        authority: ctx.accounts.authority_info.clone(),
        token_program: ctx.accounts.spl_token_program_info.clone(),
        amount,
        authority_signer_seeds: None,
    };
    spl_token_burn(params)?;

    if amount == token.amount {
        // Close token account.
        let params = TokenCloseParams {
            token_program: ctx.accounts.spl_token_program_info.clone(),
            account: ctx.accounts.token_info.clone(),
            destination: ctx.accounts.authority_info.clone(),
            owner: ctx.accounts.authority_info.clone(),
            authority_signer_seeds: None,
        };
        spl_token_close(params)?;
    }

    Ok(())
}
