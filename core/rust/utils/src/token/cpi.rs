use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
};

pub fn spl_token_burn(params: TokenBurnParams<'_, '_>) -> ProgramResult {
    let TokenBurnParams {
        mint,
        source,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let mut seeds: Vec<&[&[u8]]> = vec![];
    if let Some(seed) = authority_signer_seeds {
        seeds.push(seed);
    }
    invoke_signed(
        &spl_token_2022::instruction::burn(
            token_program.key,
            source.key,
            mint.key,
            authority.key,
            &[authority.key],
            amount,
        )?,
        &[source, mint, authority],
        seeds.as_slice(),
    )
}

/// TokenBurnParams
pub struct TokenBurnParams<'a: 'b, 'b> {
    /// mint
    pub mint: AccountInfo<'a>,
    /// source
    pub source: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: Option<&'b [&'b [u8]]>,
    /// token_program
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_close(params: TokenCloseParams<'_, '_>) -> ProgramResult {
    let TokenCloseParams {
        account,
        destination,
        owner,
        authority_signer_seeds,
        token_program,
    } = params;
    let mut seeds: Vec<&[&[u8]]> = vec![];
    if let Some(seed) = authority_signer_seeds {
        seeds.push(seed);
    }
    invoke_signed(
        &spl_token_2022::instruction::close_account(
            token_program.key,
            account.key,
            destination.key,
            owner.key,
            &[],
        )?,
        &[account, destination, owner, token_program],
        seeds.as_slice(),
    )
}

/// TokenCloseParams
pub struct TokenCloseParams<'a: 'b, 'b> {
    /// Token account
    pub account: AccountInfo<'a>,
    /// Destination for redeemed SOL.
    pub destination: AccountInfo<'a>,
    /// Owner of the token account.
    pub owner: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: Option<&'b [&'b [u8]]>,
    /// token_program
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_mint_to(params: TokenMintToParams<'_, '_>) -> ProgramResult {
    let TokenMintToParams {
        mint,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let mut seeds: Vec<&[&[u8]]> = vec![];
    if let Some(seed) = authority_signer_seeds {
        seeds.push(seed);
    }
    invoke_signed(
        &spl_token_2022::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[mint, destination, authority, token_program],
        seeds.as_slice(),
    )
}

/// TokenMintToParams
pub struct TokenMintToParams<'a: 'b, 'b> {
    /// mint
    pub mint: AccountInfo<'a>,
    /// destination
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: Option<&'b [&'b [u8]]>,
    /// token_program
    pub token_program: AccountInfo<'a>,
}

#[allow(deprecated)]
pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        amount,
        authority,
        token_program,
        authority_signer_seeds,
        ..
    } = params;
    let seeds = if let Some(seeds) = authority_signer_seeds {
        seeds
    } else {
        &[]
    };

    invoke_signed(
        &spl_token_2022::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[authority.key],
            amount,
        )?,
        &[source, destination, authority],
        &[seeds],
    )
}

/// TokenTransferParams
#[derive(Debug)]
pub struct TokenTransferParams<'a: 'b, 'b> {
    /// mint
    pub mint: AccountInfo<'a>,
    /// source
    pub source: AccountInfo<'a>,
    /// destination
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: Option<&'b [&'b [u8]]>,
    /// token_program
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_transfer_checked(params: TokenTransferCheckedParams<'_, '_>) -> ProgramResult {
    let TokenTransferCheckedParams {
        mint,
        source,
        destination,
        amount,
        authority,
        token_program,
        authority_signer_seeds,
        decimals,
    } = params;
    let seeds = if let Some(seeds) = authority_signer_seeds {
        seeds
    } else {
        &[]
    };

    invoke_signed(
        &spl_token_2022::instruction::transfer_checked(
            token_program.key,
            source.key,
            mint.key,
            destination.key,
            authority.key,
            &[authority.key],
            amount,
            decimals,
        )?,
        &[source, mint, destination, authority],
        &[seeds],
    )
}

/// TokenTransferParams
#[derive(Debug)]
pub struct TokenTransferCheckedParams<'a: 'b, 'b> {
    /// mint
    pub mint: AccountInfo<'a>,
    /// source
    pub source: AccountInfo<'a>,
    /// destination
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: Option<&'b [&'b [u8]]>,
    /// token_program
    pub token_program: AccountInfo<'a>,
    /// decimals
    pub decimals: u8,
}
