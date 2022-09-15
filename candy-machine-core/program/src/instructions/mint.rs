use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use arrayref::array_ref;
use mpl_token_metadata::{
    instruction::{
        create_master_edition_v3, create_metadata_accounts_v2, set_and_verify_collection,
        set_and_verify_sized_collection_item, update_metadata_accounts_v2,
    },
    state::{Metadata, TokenMetadataAccount},
};
use solana_program::{program::invoke_signed, sysvar};

use crate::{
    constants::{AUTHORITY_SEED, EMPTY_STR, HIDDEN_SECTION},
    utils::*,
    CandyError, CandyMachine, ConfigLine,
};

pub fn mint<'info>(ctx: Context<'_, '_, '_, 'info, Mint<'info>>) -> Result<()> {
    // (1) validation

    if !ctx.accounts.nft_metadata.data_is_empty() {
        return err!(CandyError::MetadataAccountMustBeEmpty);
    }

    let candy_machine = &mut ctx.accounts.candy_machine;
    // are there items to be minted?
    if candy_machine.items_redeemed >= candy_machine.data.items_available {
        return err!(CandyError::CandyMachineEmpty);
    }

    if !cmp_pubkeys(
        &ctx.accounts.collection_mint.key(),
        &candy_machine.collection_mint,
    ) {
        return err!(CandyError::CollectionKeyMismatch);
    }

    if !cmp_pubkeys(
        ctx.accounts.collection_metadata.owner,
        &mpl_token_metadata::id(),
    ) {
        return err!(CandyError::IncorrectOwner);
    }

    let collection_metadata = &ctx.accounts.collection_metadata;
    let collection_data: Metadata =
        Metadata::from_account_info(&collection_metadata.to_account_info())?;

    if !cmp_pubkeys(
        &collection_data.update_authority,
        &ctx.accounts.collection_update_authority.key(),
    ) {
        return err!(CandyError::IncorrectCollectionAuthority);
    }

    // (2) selecting an item to mint

    let recent_slothashes = &ctx.accounts.recent_slothashes;
    let data = recent_slothashes.data.borrow();
    let most_recent = array_ref![data, 12, 8];

    let numerator = u64::from_le_bytes(*most_recent);
    let remainder: usize = numerator
        .checked_rem(candy_machine.data.items_available - candy_machine.items_redeemed)
        .ok_or(CandyError::NumericalOverflowError)? as usize;

    let config_line = get_config_line(candy_machine, remainder, candy_machine.items_redeemed)?;

    candy_machine.items_redeemed = candy_machine
        .items_redeemed
        .checked_add(1)
        .ok_or(CandyError::NumericalOverflowError)?;

    // (3) minting

    let mut creators: Vec<mpl_token_metadata::state::Creator> =
        vec![mpl_token_metadata::state::Creator {
            address: ctx.accounts.authority_pda.key(),
            verified: true,
            share: 0,
        }];

    for c in &candy_machine.data.creators {
        creators.push(mpl_token_metadata::state::Creator {
            address: c.address,
            verified: false,
            share: c.percentage_share,
        });
    }

    let metadata_infos = vec![
        ctx.accounts.nft_metadata.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_mint_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.authority_pda.to_account_info(),
    ];

    let master_edition_infos = vec![
        ctx.accounts.nft_master_edition.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_mint_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.nft_metadata.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.authority_pda.to_account_info(),
    ];

    let cm_key = candy_machine.key();
    let authority_seeds = [
        AUTHORITY_SEED.as_bytes(),
        cm_key.as_ref(),
        &[*ctx.bumps.get("authority_pda").unwrap()],
    ];

    invoke_signed(
        &create_metadata_accounts_v2(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.nft_metadata.key(),
            ctx.accounts.nft_mint.key(),
            ctx.accounts.nft_mint_authority.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.authority_pda.key(),
            config_line.name,
            candy_machine.data.symbol.clone(),
            config_line.uri,
            Some(creators),
            candy_machine.data.seller_fee_basis_points,
            true,
            candy_machine.data.is_mutable,
            None,
            None,
        ),
        metadata_infos.as_slice(),
        &[&authority_seeds],
    )?;

    invoke_signed(
        &create_master_edition_v3(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.nft_master_edition.key(),
            ctx.accounts.nft_mint.key(),
            ctx.accounts.authority_pda.key(),
            ctx.accounts.nft_mint_authority.key(),
            ctx.accounts.nft_metadata.key(),
            ctx.accounts.payer.key(),
            Some(candy_machine.data.max_supply),
        ),
        master_edition_infos.as_slice(),
        &[&authority_seeds],
    )?;

    invoke_signed(
        &update_metadata_accounts_v2(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.nft_metadata.key(),
            ctx.accounts.authority_pda.key(),
            Some(collection_data.update_authority),
            None,
            Some(true),
            if !candy_machine.data.is_mutable {
                Some(false)
            } else {
                None
            },
        ),
        &[
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.nft_metadata.to_account_info(),
            ctx.accounts.authority_pda.to_account_info(),
        ],
        &[&authority_seeds],
    )?;

    let collection_authority_record = &ctx.accounts.collection_authority_record;
    let collection_mint = &ctx.accounts.collection_mint;
    let collection_master_edition = &ctx.accounts.collection_master_edition;
    let set_collection_ix = if collection_data.collection_details.is_some() {
        set_and_verify_sized_collection_item(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.nft_metadata.key(),
            ctx.accounts.authority_pda.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.collection_update_authority.key(),
            collection_mint.key(),
            collection_metadata.key(),
            collection_master_edition.key(),
            Some(collection_authority_record.key()),
        )
    } else {
        set_and_verify_collection(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.nft_metadata.key(),
            ctx.accounts.authority_pda.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.collection_update_authority.key(),
            collection_mint.key(),
            collection_metadata.key(),
            collection_master_edition.key(),
            Some(collection_authority_record.key()),
        )
    };

    let set_collection_infos = vec![
        ctx.accounts.nft_metadata.to_account_info(),
        ctx.accounts.authority_pda.to_account_info(),
        ctx.accounts.collection_update_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        collection_mint.to_account_info(),
        collection_metadata.to_account_info(),
        collection_master_edition.to_account_info(),
        collection_authority_record.to_account_info(),
    ];

    invoke_signed(
        &set_collection_ix,
        set_collection_infos.as_slice(),
        &[&authority_seeds],
    )?;

    Ok(())
}

pub fn get_config_line(
    candy_machine: &Account<'_, CandyMachine>,
    index: usize,
    mint_number: u64,
) -> Result<ConfigLine> {
    if let Some(hs) = &candy_machine.data.hidden_settings {
        return Ok(ConfigLine {
            name: replace_patterns(hs.name.clone(), mint_number as usize),
            uri: replace_patterns(hs.uri.clone(), mint_number as usize),
        });
    }
    let settings = if let Some(settings) = &candy_machine.data.config_line_settings {
        settings
    } else {
        return err!(CandyError::MissingConfigLinesSettings);
    };

    let account_info = candy_machine.to_account_info();
    let mut account_data = account_info.data.borrow_mut();

    // (1) determine the mint index (index is a random index on the available indices array)

    let value_to_use = if settings.is_sequential {
        mint_number as usize
    } else {
        let items_available = candy_machine.data.items_available as u64;
        let indices_start = HIDDEN_SECTION
            + 4
            + (items_available as usize) * candy_machine.data.get_config_line_size()
            + 4
            + ((items_available
                .checked_div(8)
                .ok_or(CandyError::NumericalOverflowError)?
                + 1) as usize)
            + 4;
        // calculates the mint index and retrieves the value at that position
        let mint_index = indices_start + index * 4;
        let value_to_use = u32::from_le_bytes(*array_ref![account_data, mint_index, 4]) as usize;
        // calculates the last available index and retrieves the value at that position
        let last_index = indices_start + ((items_available - mint_number - 1) * 4) as usize;
        let last_value = u32::from_le_bytes(*array_ref![account_data, last_index, 4]);
        // swap-remove: this guarantees that we remove the used mint index from the available array
        // in a constant time O(1) no matter how big the indices array is
        account_data[mint_index..mint_index + 4].copy_from_slice(&u32::to_le_bytes(last_value));

        value_to_use
    };

    // (2) retrieve the config line at the mint_index position

    let mut position =
        HIDDEN_SECTION + 4 + value_to_use * candy_machine.data.get_config_line_size();
    let name_length = settings.name_length as usize;
    let uri_length = settings.uri_length as usize;

    let name = if name_length > 0 {
        let name_slice: &mut [u8] = &mut account_data[position..position + name_length];
        String::from_utf8(name_slice.to_vec())
            .map_err(|_| CandyError::CouldNotRetrieveConfigLineData)?
    } else {
        EMPTY_STR.to_string()
    };

    position += name_length;
    let uri = if uri_length > 0 {
        let uri_slice: &mut [u8] = &mut account_data[position..position + uri_length];
        String::from_utf8(uri_slice.to_vec())
            .map_err(|_| CandyError::CouldNotRetrieveConfigLineData)?
    } else {
        EMPTY_STR.to_string()
    };

    let complete_name = replace_patterns(settings.prefix_name.clone(), value_to_use) + &name;
    let complete_uri = replace_patterns(settings.prefix_uri.clone(), value_to_use) + &uri;

    Ok(ConfigLine {
        name: complete_name,
        uri: complete_uri,
    })
}

/// Mint a new NFT pseudo-randomly from the config array.
#[derive(Accounts)]
pub struct Mint<'info> {
    #[account(mut, has_one = mint_authority)]
    candy_machine: Box<Account<'info, CandyMachine>>,
    /// CHECK: account constraints checked in account trait
    #[account(
        mut,
        seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.key().as_ref()],
        bump
    )]
    authority_pda: UncheckedAccount<'info>,
    // candy machine mint_authority (mint only allowed for the mint_authority)
    mint_authority: Signer<'info>,
    #[account(mut)]
    payer: Signer<'info>,
    // the following accounts aren't using anchor macros because they are CPI'd
    // through to token-metadata which will do all the validations we need on them
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_mint: UncheckedAccount<'info>,
    // authority of the mint account
    nft_mint_authority: Signer<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_update_authority: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::slot_hashes::id())]
    recent_slothashes: UncheckedAccount<'info>,
}
