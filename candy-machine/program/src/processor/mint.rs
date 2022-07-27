use std::{cell::RefMut, ops::Deref};

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token,
    token::{self, Token},
};
use arrayref::array_ref;
use mpl_token_metadata::{
    instruction::{
        create_master_edition_v3, create_metadata_accounts_v2, update_metadata_accounts_v2,
    },
    state::{MAX_NAME_LENGTH, MAX_URI_LENGTH},
};
use solana_gateway::{
    state::{GatewayTokenAccess, InPlaceGatewayToken},
    Gateway,
};
use solana_program::{
    clock::Clock,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    serialize_utils::{read_pubkey, read_u16},
    system_instruction, sysvar,
    sysvar::{instructions::get_instruction_relative, SysvarId},
};

use crate::{
    constants::{
        A_TOKEN, BLOCK_HASHES, BOT_FEE, COLLECTIONS_FEATURE_INDEX, COMPUTE_ID, CONFIG_ARRAY_START,
        CONFIG_LINE_SIZE, CUPCAKE_ID, EXPIRE_OFFSET, GUMDROP_ID, LOCKUP_SETTINGS_FEATURE_INDEX,
        PREFIX,
    },
    state::LockupSettings,
    utils::*,
    CandyError, CandyMachine, CandyMachineData, ConfigLine, EndSettingType, LockupType,
    WhitelistMintMode, WhitelistMintSettings,
};

use cardinal_token_manager::{
    self,
    state::{InvalidationType, TokenManagerKind},
};

use cardinal_time_invalidator::{self};

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
    /// Account not actually used.
    clock: Sysvar<'info, Clock>,
    // Leaving the name the same for IDL backward compatability
    /// CHECK: checked in program.
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
    // > Only needed if candy machine has lockup feature flag enabled
    // lockup_settings
    // token_manager
    // token_manager_token_account
    // mint_counter
    // recipient_token_account
    // time_invalidator
    // time_invalidator_program
    // token_manager_program
}

pub fn handle_mint_nft<'info>(
    ctx: Context<'_, '_, '_, 'info, MintNFT<'info>>,
    creator_bump: u8,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;
    let candy_machine_creator = &ctx.accounts.candy_machine_creator;
    // Note this is the wallet of the Candy machine
    let wallet = &ctx.accounts.wallet;
    let payer = &ctx.accounts.payer;
    let token_program = &ctx.accounts.token_program;
    let clock = Clock::get()?;
    //Account name the same for IDL compatability
    let recent_slothashes = &ctx.accounts.recent_blockhashes;
    let instruction_sysvar_account = &ctx.accounts.instruction_sysvar_account;
    let instruction_sysvar_account_info = instruction_sysvar_account.to_account_info();
    let instruction_sysvar = instruction_sysvar_account_info.data.borrow();
    let current_ix = get_instruction_relative(0, &instruction_sysvar_account_info).unwrap();
    if !ctx.accounts.metadata.data_is_empty() {
        return err!(CandyError::MetadataAccountMustBeEmpty);
    }
    if cmp_pubkeys(&recent_slothashes.key(), &BLOCK_HASHES) {
        msg!("recent_blockhashes is deprecated and will break soon");
    }
    if !cmp_pubkeys(&recent_slothashes.key(), &SlotHashes::id())
        && !cmp_pubkeys(&recent_slothashes.key(), &BLOCK_HASHES)
    {
        return err!(CandyError::IncorrectSlotHashesPubkey);
    }
    // Restrict Who can call Candy Machine via CPI
    if !cmp_pubkeys(&current_ix.program_id, &crate::id())
        && !cmp_pubkeys(&current_ix.program_id, &GUMDROP_ID)
        && !cmp_pubkeys(&current_ix.program_id, &CUPCAKE_ID)
    {
        punish_bots(
            CandyError::SuspiciousTransaction,
            payer.to_account_info(),
            ctx.accounts.candy_machine.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            BOT_FEE,
        )?;
        return Ok(());
    }
    let next_ix = get_instruction_relative(1, &instruction_sysvar_account_info);
    match next_ix {
        Ok(ix) => {
            let discriminator = &ix.data[0..8];
            let after_collection_ix = get_instruction_relative(2, &instruction_sysvar_account_info);
            if !cmp_pubkeys(&ix.program_id, &crate::id())
                || discriminator != [103, 17, 200, 25, 118, 95, 125, 61]
                || after_collection_ix.is_ok()
            {
                // We fail here. Its much cheaper to fail here than to allow a malicious user to add an ix at the end and then fail.
                msg!("Failing and Halting Here due to an extra unauthorized instruction");
                return err!(CandyError::SuspiciousTransaction);
            }
        }
        Err(_) => {
            if is_feature_active(&candy_machine.data.uuid, COLLECTIONS_FEATURE_INDEX) {
                punish_bots(
                    CandyError::MissingSetCollectionDuringMint,
                    payer.to_account_info(),
                    ctx.accounts.candy_machine.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                    BOT_FEE,
                )?;
                return Ok(());
            }
        }
    }
    let mut idx = 0;
    let num_instructions =
        read_u16(&mut idx, &instruction_sysvar).map_err(|_| ProgramError::InvalidAccountData)?;

    for index in 0..num_instructions {
        let mut current = 2 + (index * 2) as usize;
        let start = read_u16(&mut current, &instruction_sysvar).unwrap();

        current = start as usize;
        let num_accounts = read_u16(&mut current, &instruction_sysvar).unwrap();
        current += (num_accounts as usize) * (1 + 32);
        let program_id = read_pubkey(&mut current, &instruction_sysvar).unwrap();

        if !cmp_pubkeys(&program_id, &crate::id())
            && !cmp_pubkeys(&program_id, &spl_token::id())
            && !cmp_pubkeys(
                &program_id,
                &anchor_lang::solana_program::system_program::ID,
            )
            && !cmp_pubkeys(&program_id, &A_TOKEN)
            && !cmp_pubkeys(&program_id, &COMPUTE_ID)
        {
            msg!("Transaction had ix with program id {}", program_id);
            punish_bots(
                CandyError::SuspiciousTransaction,
                payer.to_account_info(),
                ctx.accounts.candy_machine.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                BOT_FEE,
            )?;
            return Ok(());
        }
    }

    let mut price = candy_machine.data.price;
    if let Some(es) = &candy_machine.data.end_settings {
        match es.end_setting_type {
            EndSettingType::Date => {
                if clock.unix_timestamp > es.number as i64
                    && !cmp_pubkeys(&ctx.accounts.payer.key(), &candy_machine.authority)
                {
                    punish_bots(
                        CandyError::CandyMachineNotLive,
                        payer.to_account_info(),
                        ctx.accounts.candy_machine.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                        BOT_FEE,
                    )?;
                    return Ok(());
                }
            }
            EndSettingType::Amount => {
                if candy_machine.items_redeemed >= es.number {
                    if !cmp_pubkeys(&ctx.accounts.payer.key(), &candy_machine.authority) {
                        punish_bots(
                            CandyError::CandyMachineEmpty,
                            payer.to_account_info(),
                            ctx.accounts.candy_machine.to_account_info(),
                            ctx.accounts.system_program.to_account_info(),
                            BOT_FEE,
                        )?;
                        return Ok(());
                    }
                    return err!(CandyError::CandyMachineEmpty);
                }
            }
        }
    }
    let mut remaining_accounts_counter: usize = 0;
    if let Some(gatekeeper) = &candy_machine.data.gatekeeper {
        if ctx.remaining_accounts.len() <= remaining_accounts_counter {
            punish_bots(
                CandyError::GatewayTokenMissing,
                payer.to_account_info(),
                ctx.accounts.candy_machine.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                BOT_FEE,
            )?;
            return Ok(());
        }
        let gateway_token_info = &ctx.remaining_accounts[remaining_accounts_counter];

        remaining_accounts_counter += 1;

        // Eval function used in the gateway CPI
        let eval_function =
            |token: &InPlaceGatewayToken<&[u8]>| match (&candy_machine.data, token.expire_time()) {
                (
                    CandyMachineData {
                        go_live_date: Some(go_live_date),
                        whitelist_mint_settings: Some(WhitelistMintSettings { presale, .. }),
                        ..
                    },
                    Some(expire_time),
                ) if !*presale && expire_time < go_live_date + EXPIRE_OFFSET => {
                    msg!(
                        "Invalid gateway token: calculated creation time {} and go_live_date {}",
                        expire_time - EXPIRE_OFFSET,
                        go_live_date
                    );
                    Err(error!(CandyError::GatewayTokenExpireTimeInvalid).into())
                }
                _ => Ok(()),
            };

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

            if Gateway::verify_and_expire_token_with_eval(
                gateway_app.clone(),
                gateway_token_info.clone(),
                payer.deref().clone(),
                &gatekeeper.gatekeeper_network,
                network_expire_feature.clone(),
                eval_function,
            )
            .is_err()
            {
                punish_bots(
                    CandyError::GatewayProgramError,
                    payer.to_account_info(),
                    ctx.accounts.candy_machine.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                    BOT_FEE,
                )?;
                return Ok(());
            }
        } else if Gateway::verify_gateway_token_with_eval(
            gateway_token_info,
            &payer.key(),
            &gatekeeper.gatekeeper_network,
            None,
            eval_function,
        )
        .is_err()
        {
            punish_bots(
                CandyError::GatewayProgramError,
                payer.to_account_info(),
                ctx.accounts.candy_machine.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                BOT_FEE,
            )?;
            return Ok(());
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
                    match candy_machine.data.go_live_date {
                        None => {
                            if !cmp_pubkeys(&ctx.accounts.payer.key(), &candy_machine.authority)
                                && !ws.presale
                            {
                                punish_bots(
                                    CandyError::CandyMachineNotLive,
                                    payer.to_account_info(),
                                    ctx.accounts.candy_machine.to_account_info(),
                                    ctx.accounts.system_program.to_account_info(),
                                    BOT_FEE,
                                )?;
                                return Ok(());
                            }
                        }
                        Some(val) => {
                            if clock.unix_timestamp < val
                                && !cmp_pubkeys(&ctx.accounts.payer.key(), &candy_machine.authority)
                                && !ws.presale
                            {
                                punish_bots(
                                    CandyError::CandyMachineNotLive,
                                    payer.to_account_info(),
                                    ctx.accounts.candy_machine.to_account_info(),
                                    ctx.accounts.system_program.to_account_info(),
                                    BOT_FEE,
                                )?;
                                return Ok(());
                            }
                        }
                    }

                    if ws.mode == WhitelistMintMode::BurnEveryTime {
                        let whitelist_token_mint =
                            &ctx.remaining_accounts[remaining_accounts_counter];
                        remaining_accounts_counter += 1;

                        let whitelist_burn_authority =
                            &ctx.remaining_accounts[remaining_accounts_counter];
                        remaining_accounts_counter += 1;

                        let key_check = assert_keys_equal(&whitelist_token_mint.key(), &ws.mint);

                        if key_check.is_err() {
                            punish_bots(
                                CandyError::IncorrectOwner,
                                payer.to_account_info(),
                                ctx.accounts.candy_machine.to_account_info(),
                                ctx.accounts.system_program.to_account_info(),
                                BOT_FEE,
                            )?;
                            return Ok(());
                        }

                        spl_token_burn(TokenBurnParams {
                            mint: whitelist_token_mint.clone(),
                            source: whitelist_token_account.clone(),
                            amount: 1,
                            authority: whitelist_burn_authority.clone(),
                            authority_signer_seeds: None,
                            token_program: token_program.to_account_info(),
                        })?;
                    }

                    if let Some(dp) = ws.discount_price {
                        price = dp;
                    }
                } else {
                    if wta.amount == 0 && ws.discount_price.is_none() && !ws.presale {
                        // A non-presale whitelist with no discount price is a forced whitelist
                        // If a pre-sale has no discount, its no issue, because the "discount"
                        // is minting first - a presale whitelist always has an open post sale.
                        punish_bots(
                            CandyError::NoWhitelistToken,
                            payer.to_account_info(),
                            ctx.accounts.candy_machine.to_account_info(),
                            ctx.accounts.system_program.to_account_info(),
                            BOT_FEE,
                        )?;
                        return Ok(());
                    }
                    let go_live = assert_valid_go_live(payer, clock, candy_machine);
                    if go_live.is_err() {
                        punish_bots(
                            CandyError::CandyMachineNotLive,
                            payer.to_account_info(),
                            ctx.accounts.candy_machine.to_account_info(),
                            ctx.accounts.system_program.to_account_info(),
                            BOT_FEE,
                        )?;
                        return Ok(());
                    }
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
                    punish_bots(
                        CandyError::NoWhitelistToken,
                        payer.to_account_info(),
                        ctx.accounts.candy_machine.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                        BOT_FEE,
                    )?;
                    return Ok(());
                }
                if ws.mode == WhitelistMintMode::BurnEveryTime {
                    remaining_accounts_counter += 2;
                }
                let go_live = assert_valid_go_live(payer, clock, candy_machine);
                if go_live.is_err() {
                    punish_bots(
                        CandyError::CandyMachineNotLive,
                        payer.to_account_info(),
                        ctx.accounts.candy_machine.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                        BOT_FEE,
                    )?;
                    return Ok(());
                }
            }
        }
    } else {
        // no whitelist means normal datecheck
        let go_live = assert_valid_go_live(payer, clock, candy_machine);
        if go_live.is_err() {
            punish_bots(
                CandyError::CandyMachineNotLive,
                payer.to_account_info(),
                ctx.accounts.candy_machine.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                BOT_FEE,
            )?;
            return Ok(());
        }
    }

    if candy_machine.items_redeemed >= candy_machine.data.items_available {
        punish_bots(
            CandyError::CandyMachineEmpty,
            payer.to_account_info(),
            ctx.accounts.candy_machine.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            BOT_FEE,
        )?;
        return Ok(());
    }

    if let Some(mint) = candy_machine.token_mint {
        let token_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
        let transfer_authority_info = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
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

    let recent_slothashes_account = recent_slothashes.to_account_info();
    let data = recent_slothashes_account.data.borrow();
    let most_recent = array_ref![data, 12, 8];

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

    if ctx.accounts.mint.data_is_empty() {
        // create account for mint
        invoke(
            &system_instruction::create_account(
                ctx.accounts.payer.key,
                ctx.accounts.mint.key,
                ctx.accounts
                    .rent
                    .minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.mint.to_account_info(),
            ],
        )?;

        // Initialize mint
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0,
            &ctx.accounts.payer.key(),
            Some(&ctx.accounts.payer.key()),
        )?;

        let recipient_token_account = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;

        // create associated token account for recipient
        associated_token::create(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            associated_token::Create {
                payer: ctx.accounts.payer.to_account_info(),
                associated_token: recipient_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ))?;

        // mint stake_entry.amount tokens to token manager mint token account
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: recipient_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            1,
        )?;
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

    let candy_machine_data = &candy_machine.data;
    let uuid = candy_machine_data.uuid.clone();
    if is_feature_active(&uuid, LOCKUP_SETTINGS_FEATURE_INDEX) {
        if ctx.remaining_accounts.len() <= remaining_accounts_counter {
            return err!(CandyError::LockupSettingsAccountMissing);
        }
        let lockup_settings_info = &ctx.remaining_accounts[remaining_accounts_counter];
        remaining_accounts_counter += 1;
        let lockup_settings = Account::<LockupSettings>::try_from(lockup_settings_info)?;
        if lockup_settings.candy_machine != candy_machine.key() {
            return err!(CandyError::LockupSettingsAccountInvalid);
        }
        handle_lockup_settings(&ctx, &mut remaining_accounts_counter, lockup_settings)?;
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
        if arr[my_position_in_vec] == 255 {
            let eight_remainder = 8 - index_to_use
                .checked_rem(8)
                .ok_or(CandyError::NumericalOverflowError)?;
            let reversed = 8 - eight_remainder + 1;
            if (eight_remainder != 0 && pos) || (reversed != 0 && !pos) {
                if pos {
                    index_to_use += eight_remainder;
                } else {
                    if index_to_use < 8 {
                        break;
                    }
                    index_to_use -= reversed;
                }
            } else if pos {
                index_to_use += 8;
            } else {
                index_to_use -= 8;
            }
        } else {
            let position_from_right = 7 - index_to_use
                .checked_rem(8)
                .ok_or(CandyError::NumericalOverflowError)?;
            let mask = u8::pow(2, position_from_right as u32);

            taken = mask & arr[my_position_in_vec];

            match taken {
                x if x > 0 => {
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

    if arr[CONFIG_ARRAY_START + 4 + index_to_use * (CONFIG_LINE_SIZE)] == 1 {
        return err!(CandyError::CannotFindUsableConfigLine);
    }

    let data_array = &mut arr[CONFIG_ARRAY_START + 4 + index_to_use * (CONFIG_LINE_SIZE)
        ..CONFIG_ARRAY_START + 4 + (index_to_use + 1) * (CONFIG_LINE_SIZE)];

    let mut name_vec = Vec::with_capacity(MAX_NAME_LENGTH);
    let mut uri_vec = Vec::with_capacity(MAX_URI_LENGTH);

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

#[inline(never)]
fn handle_lockup_settings<'info>(
    ctx: &Context<'_, '_, '_, 'info, MintNFT<'info>>,
    remaining_accounts_counter: &mut usize,
    lockup_settings: Account<LockupSettings>,
) -> Result<()> {
    if ctx.remaining_accounts.len() <= *remaining_accounts_counter {
        punish_bots(
            CandyError::LockupSettingsMissingAccounts,
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.candy_machine.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            BOT_FEE,
        )?;
        return Ok(());
    }

    if ctx.remaining_accounts.len() <= *remaining_accounts_counter {
        return err!(CandyError::LockupSettingsMissingTokenManager);
    }
    let token_manager = &ctx.remaining_accounts[*remaining_accounts_counter];
    *remaining_accounts_counter += 1;

    if ctx.remaining_accounts.len() <= *remaining_accounts_counter {
        return err!(CandyError::LockupSettingsMissingTokenManagerTokenAccount);
    }
    let token_manager_token_account = &ctx.remaining_accounts[*remaining_accounts_counter];
    *remaining_accounts_counter += 1;

    if ctx.remaining_accounts.len() <= *remaining_accounts_counter {
        return err!(CandyError::LockupSettingsMissingMintCounter);
    }
    let mint_counter = &ctx.remaining_accounts[*remaining_accounts_counter];
    *remaining_accounts_counter += 1;

    if ctx.remaining_accounts.len() <= *remaining_accounts_counter {
        return err!(CandyError::LockupSettingsMissingRecipientTokenAccount);
    }
    let recipient_token_account = &ctx.remaining_accounts[*remaining_accounts_counter];
    *remaining_accounts_counter += 1;

    if ctx.remaining_accounts.len() <= *remaining_accounts_counter {
        return err!(CandyError::LockupSettingsMissingTokenManagerProgram);
    }
    let token_manager_program = &ctx.remaining_accounts[*remaining_accounts_counter];
    if token_manager_program.key() != cardinal_token_manager::id() {
        return err!(CandyError::LockupSettingsInvalidTokenManagerProgram);
    }
    *remaining_accounts_counter += 1;

    // token manager init
    let init_ix = cardinal_token_manager::instructions::InitIx {
        amount: 1, // todo change for fungible
        kind: TokenManagerKind::Edition as u8,
        invalidation_type: InvalidationType::Release as u8,
        num_invalidators: 1,
    };
    let cpi_accounts = cardinal_token_manager::cpi::accounts::InitCtx {
        token_manager: token_manager.to_account_info(),
        mint_counter: mint_counter.to_account_info(),
        issuer: ctx.accounts.payer.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        issuer_token_account: recipient_token_account.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(token_manager_program.to_account_info(), cpi_accounts);
    cardinal_token_manager::cpi::init(cpi_ctx, init_ix)?;

    handle_time_invalidator(
        ctx,
        remaining_accounts_counter,
        lockup_settings,
        token_manager,
        token_manager_program,
    )?;

    // create associated token account for recipient
    let cpi_accounts = associated_token::Create {
        payer: ctx.accounts.payer.to_account_info(),
        associated_token: token_manager_token_account.to_account_info(),
        authority: token_manager.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    associated_token::create(cpi_context)?;

    // token manager issue
    let cpi_accounts = cardinal_token_manager::cpi::accounts::IssueCtx {
        token_manager: token_manager.to_account_info(),
        token_manager_token_account: token_manager_token_account.to_account_info(),
        issuer: ctx.accounts.payer.to_account_info(),
        issuer_token_account: recipient_token_account.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(token_manager_program.to_account_info(), cpi_accounts);
    cardinal_token_manager::cpi::issue(cpi_ctx)?;

    // token manager claim
    let cpi_accounts = cardinal_token_manager::cpi::accounts::ClaimCtx {
        token_manager: token_manager.to_account_info(),
        token_manager_token_account: token_manager_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        recipient: ctx.accounts.payer.to_account_info(),
        recipient_token_account: recipient_token_account.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    // let remaining_accounts = ctx.remaining_accounts.to_vec();
    let cpi_ctx = CpiContext::new(token_manager_program.to_account_info(), cpi_accounts)
        .with_remaining_accounts(
            [
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.token_metadata_program.to_account_info(),
            ]
            .to_vec(),
        );
    cardinal_token_manager::cpi::claim(cpi_ctx)?;
    Ok(())
}

#[inline(never)]
fn handle_time_invalidator<'info>(
    ctx: &Context<'_, '_, '_, 'info, MintNFT<'info>>,
    remaining_accounts_counter: &mut usize,
    lockup_settings: Account<LockupSettings>,
    token_manager: &AccountInfo<'info>,
    token_manager_program: &AccountInfo<'info>,
) -> Result<()> {
    if ctx.remaining_accounts.len() <= *remaining_accounts_counter {
        return err!(CandyError::LockupSettingsMissingTimeInvalidator);
    }
    let time_invalidator = &ctx.remaining_accounts[*remaining_accounts_counter];
    *remaining_accounts_counter += 1;

    if ctx.remaining_accounts.len() <= *remaining_accounts_counter {
        return err!(CandyError::LockupSettingsMissingTimeInvalidatorProgram);
    }
    let time_invalidator_program = &ctx.remaining_accounts[*remaining_accounts_counter];
    if time_invalidator_program.key() != cardinal_time_invalidator::id() {
        return err!(CandyError::LockupSettingsInvalidTimeInvalidatorProgram);
    }
    *remaining_accounts_counter += 1;

    // init time invalidator
    let time_invalidator_init_ix = cardinal_time_invalidator::instructions::InitIx {
        collector: ctx.accounts.payer.key(),
        // no fees
        payment_manager: Pubkey::default(),
        duration_seconds: if lockup_settings.lockup_type == LockupType::DurationSeconds as u8 {
            Some(lockup_settings.number)
        } else {
            None
        },
        extension_payment_amount: None,
        extension_duration_seconds: None,
        extension_payment_mint: None,
        max_expiration: if lockup_settings.lockup_type == LockupType::ExpirationUnixTimstamp as u8 {
            Some(lockup_settings.number)
        } else {
            None
        },
        disable_partial_extension: None,
    };
    let cpi_accounts = cardinal_time_invalidator::cpi::accounts::InitCtx {
        token_manager: token_manager.to_account_info(),
        time_invalidator: time_invalidator.to_account_info(),
        issuer: ctx.accounts.payer.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(time_invalidator_program.to_account_info(), cpi_accounts);
    cardinal_time_invalidator::cpi::init(cpi_ctx, time_invalidator_init_ix)?;

    // add invalidator
    let cpi_accounts = cardinal_token_manager::cpi::accounts::AddInvalidatorCtx {
        token_manager: token_manager.to_account_info(),
        issuer: ctx.accounts.payer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(token_manager_program.to_account_info(), cpi_accounts);
    cardinal_token_manager::cpi::add_invalidator(cpi_ctx, time_invalidator.key())?;
    Ok(())
}
