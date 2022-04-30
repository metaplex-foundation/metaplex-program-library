use crate::{
    candy_machine,
    constants::{
        COLLECTIONS_FEATURE_INDEX, CONFIG_ARRAY_START, CONFIG_LINE_SIZE, EXPIRE_OFFSET, PREFIX,
    },
    utils::*,
    CandyError, CandyMachine, ConfigLine, EndSettingType, WhitelistMintMode,
};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use arrayref::array_ref;
use mpl_token_metadata::{
    instruction::{
        create_master_edition_v3, create_metadata_accounts_v2, update_metadata_accounts_v2,
    },
    state::{MAX_NAME_LENGTH, MAX_URI_LENGTH},
};
use solana_gateway::{borsh::try_from_slice_incomplete, state::GatewayToken, Gateway};
use solana_program::{
    clock::Clock,
    program::{invoke, invoke_signed},
    serialize_utils::{read_pubkey, read_u16},
    system_instruction, sysvar,
    sysvar::instructions::get_instruction_relative,
};
use std::{cell::RefMut, ops::Deref, str::FromStr};

/// Mint a new NFT pseudo-randomly from the config array.
#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct MintNFT<'info> {
    #[account(
    mut,
    has_one = wallet
    )]
    candy_machine: Box<Account<'info, CandyMachine>>,
    /// CHECK: account constraints checked in account trait
    #[account(seeds=[PREFIX.as_bytes(), candy_machine.key().as_ref()], bump=creator_bump)]
    candy_machine_creator: UncheckedAccount<'info>,
    payer: Signer<'info>,
    /// CHECK: wallet can be any account and is not written to or read
    #[account(mut)]
    wallet: UncheckedAccount<'info>,
    // With the following accounts we aren't using anchor macros because they are CPI'd
    // through to token-metadata which will do all the validations we need on them.
    /// CHECK: account checked in CPI
    #[account(mut)]
    metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    mint: UncheckedAccount<'info>,
    mint_authority: Signer<'info>,
    update_authority: Signer<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
    // Leaving the name the same for IDL backward compatability
    /// CHECK: account checked in CPI
    recent_blockhashes: UncheckedAccount<'info>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    instruction_sysvar_account: UncheckedAccount<'info>,
    // > Only needed if candy machine has a gatekeeper
    // gateway_token
    // > Only needed if candy machine has a gatekeeper and it has expire_on_use set to true:
    // gateway program
    // network_expire_feature
    // > Only needed if candy machine has whitelist_mint_settings
    // whitelist_token_account
    // > Only needed if candy machine has whitelist_mint_settings and mode is BurnEveryTime
    // whitelist_token_mint
    // whitelist_burn_authority
    // > Only needed if candy machine has token mint
    // token_account_info
    // transfer_authority_info
}

#[inline(never)]
pub fn handle_mint_nft<'info>(
    ctx: Context<'_, '_, '_, 'info, MintNFT<'info>>,
    creator_bump: u8,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;
    let candy_machine_creator = &ctx.accounts.candy_machine_creator;
    let clock = &ctx.accounts.clock;
    // Note this is the wallet of the Candy machine
    let wallet = &ctx.accounts.wallet;
    let payer = &ctx.accounts.payer;
    let token_program = &ctx.accounts.token_program;
    let instruction_sysvar_account = &ctx.accounts.instruction_sysvar_account;

    let instruction_sysvar_account_info = instruction_sysvar_account.to_account_info();
    let instruction_sysvar = instruction_sysvar_account_info.data.borrow();
    if is_feature_active(&candy_machine.data.uuid, COLLECTIONS_FEATURE_INDEX) {
        let next_instruction = get_instruction_relative(1, &instruction_sysvar_account_info)?;
        if next_instruction.program_id != candy_machine::id() {
            msg!(
                "Transaction had ix with program id {}",
                &next_instruction.program_id
            );
            return Err(CandyError::SuspiciousTransaction.into());
        }
        let discriminator = &next_instruction.data[0..8];

        // Set collection during mint discriminator
        if discriminator != [103, 17, 200, 25, 118, 95, 125, 61] {
            msg!("Transaction had ix with data {:?}", discriminator);
            return Err(CandyError::SuspiciousTransaction.into());
        }
    }

    let mut price = candy_machine.data.price;
    if let Some(es) = &candy_machine.data.end_settings {
        match es.end_setting_type {
            EndSettingType::Date => {
                if clock.unix_timestamp > es.number as i64
                    && ctx.accounts.payer.key() != candy_machine.authority
                {
                    return err!(CandyError::CandyMachineNotLive);
                }
            }
            EndSettingType::Amount => {
                if candy_machine.items_redeemed >= es.number {
                    return err!(CandyError::CandyMachineNotLive);
                }
            }
        }
    }

    let mut remaining_accounts_counter: usize = 0;
    if let Some(gatekeeper) = &candy_machine.data.gatekeeper {
        if ctx.remaining_accounts.len() <= remaining_accounts_counter {
            return err!(CandyError::GatewayTokenMissing);
        }
        let gateway_token_info = &ctx.remaining_accounts[remaining_accounts_counter];
        let gateway_token =
            try_from_slice_incomplete::<GatewayToken>(*gateway_token_info.data.borrow())?;
        // stores the expire_time before the verification, since the verification
        // will update the expire_time of the token and we won't be able to
        // calculate the creation time
        let expire_time = gateway_token
            .expire_time
            .ok_or(CandyError::GatewayTokenExpireTimeInvalid)? as i64;
        remaining_accounts_counter += 1;
        if gatekeeper.expire_on_use {
            if ctx.remaining_accounts.len() <= remaining_accounts_counter {
                return err!(CandyError::GatewayAppMissing);
            }
            let gateway_app = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            if ctx.remaining_accounts.len() <= remaining_accounts_counter {
                return err!(CandyError::NetworkExpireFeatureMissing);
            }
            let network_expire_feature = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            Gateway::verify_and_expire_token(
                gateway_app.clone(),
                gateway_token_info.clone(),
                payer.deref().clone(),
                &gatekeeper.gatekeeper_network,
                network_expire_feature.clone(),
            )?;
        } else {
            Gateway::verify_gateway_token_account_info(
                gateway_token_info,
                &payer.key(),
                &gatekeeper.gatekeeper_network,
                None,
            )
            .map_err(Into::<ProgramError>::into)?;
        }
        // verifies that the gatway token was not created before the candy
        // machine go_live_date (avoids pre-solving the captcha)
        match candy_machine.data.go_live_date {
            Some(val) => {
                if (expire_time - EXPIRE_OFFSET) < val {
                    if let Some(ws) = &candy_machine.data.whitelist_mint_settings {
                        // when dealing with whitelist, the expire_time can be
                        // before the go_live_date only if presale enabled
                        if !ws.presale {
                            msg!(
                                    "Invalid gateway token: calculated creation time {} and go_live_date {}",
                                    expire_time - EXPIRE_OFFSET,
                                    val);
                            return err!(CandyError::GatewayTokenExpireTimeInvalid);
                        }
                    } else {
                        msg!(
                                "Invalid gateway token: calculated creation time {} and go_live_date {}",
                                expire_time - EXPIRE_OFFSET,
                                val);
                        return err!(CandyError::GatewayTokenExpireTimeInvalid);
                    }
                }
            }
            None => {}
        }
    }

    if let Some(ws) = &candy_machine.data.whitelist_mint_settings {
        let whitelist_token_account = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
        // If the user has not actually made this account,
        // this explodes and we just check normal dates.
        // If they have, we check amount, if it's > 0 we let them use the logic
        // if 0, check normal dates.
        match assert_is_ata(whitelist_token_account, &payer.key(), &ws.mint) {
            Ok(wta) => {
                if wta.amount > 0 {
                    if ws.mode == WhitelistMintMode::BurnEveryTime {
                        let whitelist_token_mint =
                            &ctx.remaining_accounts[remaining_accounts_counter];
                        remaining_accounts_counter += 1;

                        let whitelist_burn_authority =
                            &ctx.remaining_accounts[remaining_accounts_counter];
                        remaining_accounts_counter += 1;

                        assert_keys_equal(whitelist_token_mint.key(), ws.mint)?;

                        spl_token_burn(TokenBurnParams {
                            mint: whitelist_token_mint.clone(),
                            source: whitelist_token_account.clone(),
                            amount: 1,
                            authority: whitelist_burn_authority.clone(),
                            authority_signer_seeds: None,
                            token_program: token_program.to_account_info(),
                        })?;
                    }

                    match candy_machine.data.go_live_date {
                        None => {
                            if ctx.accounts.payer.key() != candy_machine.authority && !ws.presale {
                                return err!(CandyError::CandyMachineNotLive);
                            }
                        }
                        Some(val) => {
                            if clock.unix_timestamp < val
                                && ctx.accounts.payer.key() != candy_machine.authority
                                && !ws.presale
                            {
                                return err!(CandyError::CandyMachineNotLive);
                            }
                        }
                    }

                    if let Some(dp) = ws.discount_price {
                        price = dp;
                    }
                } else {
                    if wta.amount == 0 && ws.discount_price.is_none() && !ws.presale {
                        // A non-presale whitelist with no discount price is a forced whitelist
                        // If a pre-sale has no discount, its no issue, because the "discount"
                        // is minting first - a presale whitelist always has an open post sale.
                        return err!(CandyError::NoWhitelistToken);
                    }
                    assert_valid_go_live(payer, clock, candy_machine)?;
                    if ws.mode == WhitelistMintMode::BurnEveryTime {
                        remaining_accounts_counter += 2;
                    }
                }
            }
            Err(_) => {
                if ws.discount_price.is_none() && !ws.presale {
                    // A non-presale whitelist with no discount price is a forced whitelist
                    // If a pre-sale has no discount, its no issue, because the "discount"
                    // is minting first - a presale whitelist always has an open post sale.
                    return err!(CandyError::NoWhitelistToken);
                }
                if ws.mode == WhitelistMintMode::BurnEveryTime {
                    remaining_accounts_counter += 2;
                }
                assert_valid_go_live(payer, clock, candy_machine)?
            }
        }
    } else {
        // no whitelist means normal datecheck
        assert_valid_go_live(payer, clock, candy_machine)?;
    }

    if candy_machine.items_redeemed >= candy_machine.data.items_available {
        return err!(CandyError::CandyMachineEmpty);
    }

    if let Some(mint) = candy_machine.token_mint {
        let token_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
        let transfer_authority_info = &ctx.remaining_accounts[remaining_accounts_counter];

        // If we ever add another account, this will need to be uncommented:
        // remaining_accounts_counter += 1;

        let token_account = assert_is_ata(token_account_info, &payer.key(), &mint)?;

        if token_account.amount < price {
            return err!(CandyError::NotEnoughTokens);
        }

        spl_token_transfer(TokenTransferParams {
            source: token_account_info.clone(),
            destination: wallet.to_account_info(),
            authority: transfer_authority_info.clone(),
            authority_signer_seeds: &[],
            token_program: token_program.to_account_info(),
            amount: price,
        })?;
    } else {
        if ctx.accounts.payer.lamports() < price {
            return err!(CandyError::NotEnoughSOL);
        }

        invoke(
            &system_instruction::transfer(&ctx.accounts.payer.key(), &wallet.key(), price),
            &[
                ctx.accounts.payer.to_account_info(),
                wallet.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }
    let recent_slothashes = <SlotHashes as sysvar::Sysvar>::get()?;
    let (_most_recent, hash) = recent_slothashes
        .first()
        .ok_or(CandyError::SlotHashesEmpty)?;

    let most_recent = array_ref![hash.as_ref(), 0, 8];
    let index = u64::from_le_bytes(*most_recent);
    let modded: usize = index
        .checked_rem(candy_machine.data.items_available)
        .ok_or(CandyError::NumericalOverflowError)? as usize;

    let config_line = get_config_line(candy_machine, modded, candy_machine.items_redeemed)?;

    candy_machine.items_redeemed = candy_machine
        .items_redeemed
        .checked_add(1)
        .ok_or(CandyError::NumericalOverflowError)?;

    let cm_key = candy_machine.key();
    let authority_seeds = [PREFIX.as_bytes(), cm_key.as_ref(), &[creator_bump]];

    let mut creators: Vec<mpl_token_metadata::state::Creator> =
        vec![mpl_token_metadata::state::Creator {
            address: candy_machine_creator.key(),
            verified: true,
            share: 0,
        }];

    for c in &candy_machine.data.creators {
        creators.push(mpl_token_metadata::state::Creator {
            address: c.address,
            verified: false,
            share: c.share,
        });
    }

    let metadata_infos = vec![
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.mint_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        candy_machine_creator.to_account_info(),
    ];

    let master_edition_infos = vec![
        ctx.accounts.master_edition.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.mint_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        candy_machine_creator.to_account_info(),
    ];
    invoke_signed(
        &create_metadata_accounts_v2(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.mint.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.payer.key(),
            candy_machine_creator.key(),
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
            ctx.accounts.master_edition.key(),
            ctx.accounts.mint.key(),
            candy_machine_creator.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.payer.key(),
            Some(candy_machine.data.max_supply),
        ),
        master_edition_infos.as_slice(),
        &[&authority_seeds],
    )?;

    let mut new_update_authority = Some(candy_machine.authority);

    if !candy_machine.data.retain_authority {
        new_update_authority = Some(ctx.accounts.update_authority.key());
    }
    invoke_signed(
        &update_metadata_accounts_v2(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata.key(),
            candy_machine_creator.key(),
            new_update_authority,
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
            ctx.accounts.metadata.to_account_info(),
            candy_machine_creator.to_account_info(),
        ],
        &[&authority_seeds],
    )?;

    let mut idx = 0;
    let num_instructions =
        read_u16(&mut idx, &instruction_sysvar).map_err(|_| ProgramError::InvalidAccountData)?;

    let associated_token =
        Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();

    for index in 0..num_instructions {
        let mut current = 2 + (index * 2) as usize;
        let start = read_u16(&mut current, &instruction_sysvar).unwrap();

        current = start as usize;
        let num_accounts = read_u16(&mut current, &instruction_sysvar).unwrap();
        current += (num_accounts as usize) * (1 + 32);
        let program_id = read_pubkey(&mut current, &instruction_sysvar).unwrap();

        if program_id != candy_machine::id()
            && program_id != spl_token::id()
            && program_id != anchor_lang::solana_program::system_program::ID
            && program_id != associated_token
        {
            msg!("Transaction had ix with program id {}", program_id);
            return err!(CandyError::SuspiciousTransaction);
        }
    }

    Ok(())
}

pub fn get_good_index(
    arr: &mut RefMut<&mut [u8]>,
    items_available: usize,
    index: usize,
    pos: bool,
) -> Result<(usize, bool)> {
    let mut index_to_use = index;
    let mut taken = 1;
    let mut found = false;
    let bit_mask_vec_start = CONFIG_ARRAY_START
        + 4
        + (items_available) * CONFIG_LINE_SIZE
        + 4
        + items_available
            .checked_div(8)
            .ok_or(CandyError::NumericalOverflowError)?
        + 4;

    while taken > 0 && index_to_use < items_available {
        let my_position_in_vec = bit_mask_vec_start
            + index_to_use
                .checked_div(8)
                .ok_or(CandyError::NumericalOverflowError)?;
        /*msg!(
            "My position is {} and value there is {}",
            my_position_in_vec,
            arr[my_position_in_vec]
        );*/
        if arr[my_position_in_vec] == 255 {
            //msg!("We are screwed here, move on");
            let eight_remainder = 8 - index_to_use
                .checked_rem(8)
                .ok_or(CandyError::NumericalOverflowError)?;
            let reversed = 8 - eight_remainder + 1;
            if (eight_remainder != 0 && pos) || (reversed != 0 && !pos) {
                //msg!("Moving by {}", eight_remainder);
                if pos {
                    index_to_use += eight_remainder;
                } else {
                    if index_to_use < 8 {
                        break;
                    }
                    index_to_use -= reversed;
                }
            } else {
                //msg!("Moving by 8");
                if pos {
                    index_to_use += 8;
                } else {
                    index_to_use -= 8;
                }
            }
        } else {
            let position_from_right = 7 - index_to_use
                .checked_rem(8)
                .ok_or(CandyError::NumericalOverflowError)?;
            let mask = u8::pow(2, position_from_right as u32);

            taken = mask & arr[my_position_in_vec];

            match taken {
                x if x > 0 => {
                    //msg!("Index to use {} is taken", index_to_use);
                    if pos {
                        index_to_use += 1;
                    } else {
                        if index_to_use == 0 {
                            break;
                        }
                        index_to_use -= 1;
                    }
                }
                0 => {
                    //msg!("Index to use {} is not taken, exiting", index_to_use);
                    found = true;
                    arr[my_position_in_vec] |= mask;
                }
                _ => (),
            }
        }
    }
    Ok((index_to_use, found))
}

pub fn get_config_line(
    a: &Account<'_, CandyMachine>,
    index: usize,
    mint_number: u64,
) -> Result<ConfigLine> {
    if let Some(hs) = &a.data.hidden_settings {
        return Ok(ConfigLine {
            name: hs.name.clone() + "#" + &(mint_number + 1).to_string(),
            uri: hs.uri.clone(),
        });
    }
    msg!("Index is set to {:?}", index);
    let a_info = a.to_account_info();

    let mut arr = a_info.data.borrow_mut();

    let (mut index_to_use, good) =
        get_good_index(&mut arr, a.data.items_available as usize, index, true)?;
    if !good {
        let (index_to_use_new, good_new) =
            get_good_index(&mut arr, a.data.items_available as usize, index, false)?;
        index_to_use = index_to_use_new;
        if !good_new {
            return err!(CandyError::CannotFindUsableConfigLine);
        }
    }

    msg!(
        "Index actually ends up due to used bools {:?}",
        index_to_use
    );
    if arr[CONFIG_ARRAY_START + 4 + index_to_use * (CONFIG_LINE_SIZE)] == 1 {
        return err!(CandyError::CannotFindUsableConfigLine);
    }

    let data_array = &mut arr[CONFIG_ARRAY_START + 4 + index_to_use * (CONFIG_LINE_SIZE)
        ..CONFIG_ARRAY_START + 4 + (index_to_use + 1) * (CONFIG_LINE_SIZE)];

    let mut name_vec = vec![];
    let mut uri_vec = vec![];

    #[allow(clippy::needless_range_loop)]
    for i in 4..4 + MAX_NAME_LENGTH {
        if data_array[i] == 0 {
            break;
        }
        name_vec.push(data_array[i])
    }

    #[allow(clippy::needless_range_loop)]
    for i in 8 + MAX_NAME_LENGTH..8 + MAX_NAME_LENGTH + MAX_URI_LENGTH {
        if data_array[i] == 0 {
            break;
        }
        uri_vec.push(data_array[i])
    }
    let config_line: ConfigLine = ConfigLine {
        name: match String::from_utf8(name_vec) {
            Ok(val) => val,
            Err(_) => return err!(CandyError::InvalidString),
        },
        uri: match String::from_utf8(uri_vec) {
            Ok(val) => val,
            Err(_) => return err!(CandyError::InvalidString),
        },
    };

    Ok(config_line)
}
