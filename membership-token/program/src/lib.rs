pub mod error;
pub mod state;
pub mod utils;

use crate::{
    error::ErrorCode,
    state::{Market, MarketState, SellingResource, SellingResourceState, Store, TradeHistory},
    utils::{
        assert_derivation, assert_keys_equal, mpl_mint_new_edition_from_master_edition_via_token,
        mpl_update_primary_sale_happened_via_token, puffed_out_string, sys_create_account,
        sys_transfer, DESCRIPTION_MAX_LEN, FLAG_ACCOUNT_SIZE, HISTORY_PREFIX, HOLDER_PREFIX,
        NAME_MAX_LEN, PAYOUT_TICKET_PREFIX, VAULT_OWNER_PREFIX,
    },
};
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, program_pack::Pack, system_instruction},
    AnchorDeserialize, AnchorSerialize, System,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address, AssociatedToken},
    token::{self, accessor, Mint, Token, TokenAccount},
};

declare_id!("EHE2kYEETZbRfhQoNtknbnqrrpKEojbohSagkGdiJ6wm");

#[program]
pub mod membership_token {
    use super::*;

    pub fn init_selling_resource<'info>(
        ctx: Context<'_, '_, '_, 'info, InitSellingResource<'info>>,
        _master_edition_bump: u8,
        _vault_owner_bump: u8,
        max_supply: Option<u64>,
    ) -> ProgramResult {
        let store = &ctx.accounts.store;
        let admin = &ctx.accounts.admin;
        let selling_resource = &mut ctx.accounts.selling_resource;
        let selling_resource_owner = &ctx.accounts.selling_resource_owner;
        let resource_mint = &ctx.accounts.resource_mint;
        let master_edition_info = &ctx.accounts.master_edition.to_account_info();
        let metadata = &ctx.accounts.metadata;
        let vault = &ctx.accounts.vault;
        let owner = &ctx.accounts.owner;
        let resource_token = &ctx.accounts.resource_token;
        let token_program = &ctx.accounts.token_program;

        // Check `MasterEdition` derivation
        assert_derivation(
            &mpl_token_metadata::id(),
            master_edition_info,
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                resource_mint.key().as_ref(),
                mpl_token_metadata::state::EDITION.as_bytes(),
            ],
        )?;

        // Check, that provided metadata is correct
        assert_derivation(
            &mpl_token_metadata::id(),
            metadata,
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                resource_mint.key().as_ref(),
            ],
        )?;

        let metadata =
            mpl_token_metadata::state::Metadata::from_account_info(&metadata.to_account_info())?;

        // Check, that at least one creator exists in primary sale
        if !metadata.primary_sale_happened {
            if let Some(creators) = metadata.data.creators {
                if creators.len() == 0 {
                    return Err(ErrorCode::MetadataCreatorsIsEmpty.into());
                }
            } else {
                return Err(ErrorCode::MetadataCreatorsIsEmpty.into());
            }
        }

        let master_edition =
            mpl_token_metadata::state::MasterEditionV2::from_account_info(master_edition_info)?;

        let mut actual_max_supply = max_supply;

        // Ensure, that provided `max_supply` is under `MasterEditionV2::max_supply` bounds
        if let Some(me_max_supply) = master_edition.max_supply {
            let x = if let Some(max_supply) = max_supply {
                let available_supply = me_max_supply - master_edition.supply;
                if max_supply > available_supply {
                    return Err(ErrorCode::SupplyIsGtThanAvailable.into());
                } else {
                    max_supply
                }
            } else {
                return Err(ErrorCode::SupplyIsNotProvided.into());
            };

            actual_max_supply = Some(x);
        }

        // Transfer `MasterEdition` ownership
        let cpi_program = token_program.to_account_info();
        let cpi_accounts = token::Transfer {
            from: resource_token.to_account_info(),
            to: vault.to_account_info(),
            authority: admin.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, 1)?;

        selling_resource.store = store.key();
        selling_resource.owner = selling_resource_owner.key();
        selling_resource.resource = resource_mint.key();
        selling_resource.vault = vault.key();
        selling_resource.vault_owner = owner.key();
        selling_resource.supply = 0;
        selling_resource.max_supply = actual_max_supply;
        selling_resource.state = SellingResourceState::Created;

        Ok(())
    }

    pub fn create_store<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateStore<'info>>,
        name: String,
        description: String,
    ) -> ProgramResult {
        let admin = &ctx.accounts.admin;
        let store = &mut ctx.accounts.store;

        if name.len() > NAME_MAX_LEN {
            return Err(ErrorCode::NameIsTooLong.into());
        }

        if description.len() > DESCRIPTION_MAX_LEN {
            return Err(ErrorCode::DescriptionIsTooLong.into());
        }

        store.admin = admin.key();
        store.name = puffed_out_string(name, NAME_MAX_LEN);
        store.description = puffed_out_string(description, DESCRIPTION_MAX_LEN);

        Ok(())
    }

    pub fn buy<'info>(
        ctx: Context<'_, '_, '_, 'info, Buy<'info>>,
        _trade_history_bump: u8,
        vault_owner_bump: u8,
    ) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        let selling_resource = &mut ctx.accounts.selling_resource;
        let user_token_account = &mut ctx.accounts.user_token_account;
        let user_wallet = &mut ctx.accounts.user_wallet;
        let trade_history = &mut ctx.accounts.trade_history;
        let treasury_holder = &mut ctx.accounts.treasury_holder;
        let new_metadata = &mut ctx.accounts.new_metadata;
        let new_edition = &mut ctx.accounts.new_edition;
        let master_edition = &mut ctx.accounts.master_edition;
        let new_mint = &mut ctx.accounts.new_mint;
        let edition_marker_info = &mut ctx.accounts.edition_marker.to_account_info();
        let vault = &mut ctx.accounts.vault;
        let owner = &mut ctx.accounts.owner;
        let master_edition_metadata = &mut ctx.accounts.master_edition_metadata;
        let clock = &ctx.accounts.clock;
        let rent = &ctx.accounts.rent;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        let metadata_mint = selling_resource.resource.clone();
        let edition = selling_resource.supply;

        // Check, that `Market` is not in `Suspended` state
        if market.state == MarketState::Suspended {
            return Err(ErrorCode::MarketIsSuspended.into());
        }

        // Check, that `Market` is started
        if market.start_date > clock.unix_timestamp as u64 {
            return Err(ErrorCode::MarketIsNotStarted.into());
        }

        // Check, that `Market` is ended
        if let Some(end_date) = market.end_date {
            if clock.unix_timestamp as u64 > end_date {
                return Err(ErrorCode::MarketIsEnded.into());
            }
        } else if market.state == MarketState::Ended {
            return Err(ErrorCode::MarketIsEnded.into());
        }

        if trade_history.market != market.key() {
            trade_history.market = market.key();
        }

        if trade_history.wallet != user_wallet.key() {
            trade_history.wallet = user_wallet.key();
        }

        // Check, that user not reach buy limit
        if let Some(pieces_in_one_wallet) = market.pieces_in_one_wallet {
            if trade_history.already_bought == pieces_in_one_wallet {
                return Err(ErrorCode::UserReachBuyLimit.into());
            }
        }

        // Buy new edition
        let is_native = market.treasury_mint == System::id();

        if !is_native {
            let cpi_program = token_program.to_account_info();
            let cpi_accounts = token::Transfer {
                from: user_token_account.to_account_info(),
                to: treasury_holder.to_account_info(),
                authority: user_wallet.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, market.price)?;
        } else {
            if user_token_account.key() != user_wallet.key() {
                return Err(ErrorCode::UserWalletMustMatchUserTokenAccount.into());
            }

            invoke(
                // for native SOL transfer user_wallet key == user_token_account key
                &system_instruction::transfer(
                    &user_token_account.key(),
                    &treasury_holder.key(),
                    market.price,
                ),
                &[
                    user_token_account.to_account_info(),
                    treasury_holder.to_account_info(),
                ],
            )?;
        }

        mpl_mint_new_edition_from_master_edition_via_token(
            &new_metadata.to_account_info(),
            &new_edition.to_account_info(),
            &new_mint.to_account_info(),
            &user_wallet.to_account_info(),
            &user_wallet.to_account_info(),
            &owner.to_account_info(),
            &vault.to_account_info(),
            &master_edition_metadata.to_account_info(),
            &master_edition.to_account_info(),
            &metadata_mint,
            &edition_marker_info,
            &token_program.to_account_info(),
            &system_program.to_account_info(),
            &rent.to_account_info(),
            edition,
            &[
                VAULT_OWNER_PREFIX.as_bytes(),
                selling_resource.resource.as_ref(),
                selling_resource.store.as_ref(),
                &[vault_owner_bump],
            ],
        )?;

        trade_history.already_bought = trade_history
            .already_bought
            .checked_add(1)
            .ok_or(ErrorCode::MathOverflow)?;

        selling_resource.supply = selling_resource
            .supply
            .checked_add(1)
            .ok_or(ErrorCode::MathOverflow)?;

        // Check, that `SellingResource::max_supply` is not overflowed by `supply`
        if let Some(max_supply) = selling_resource.max_supply {
            if selling_resource.supply > max_supply {
                return Err(ErrorCode::SupplyIsGtThanMaxSupply.into());
            } else if selling_resource.supply == max_supply {
                selling_resource.state = SellingResourceState::Exhausted;
            }
        }

        Ok(())
    }

    pub fn close_market<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseMarket<'info>>,
    ) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        let clock = &ctx.accounts.clock;

        // Check, that `Market` is with unlimited duration
        if market.end_date.is_some() {
            return Err(ErrorCode::MarketDurationIsNotUnlimited.into());
        }

        // Check, that `Market` is started
        if market.start_date > clock.unix_timestamp as u64 {
            return Err(ErrorCode::MarketIsNotStarted.into());
        }

        market.state = MarketState::Ended;

        Ok(())
    }

    pub fn suspend_market<'info>(
        ctx: Context<'_, '_, '_, 'info, SuspendMarket<'info>>,
    ) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        let clock = &ctx.accounts.clock;

        // Check, that `Market` is in `Active` state
        if market.state == MarketState::Ended {
            return Err(ErrorCode::MarketIsEnded.into());
        }

        if let Some(end_date) = market.end_date {
            if clock.unix_timestamp as u64 > end_date {
                return Err(ErrorCode::MarketIsEnded.into());
            }
        }

        // Check, that `Market` is started
        if market.start_date > clock.unix_timestamp as u64 {
            return Err(ErrorCode::MarketIsNotStarted.into());
        }

        // Check, that `Market` is mutable
        if !market.mutable {
            return Err(ErrorCode::MarketIsImmutable.into());
        }

        // Check, that `Market` is not in `Suspended` state
        if market.state == MarketState::Suspended {
            return Err(ErrorCode::MarketIsSuspended.into());
        }

        market.state = MarketState::Suspended;

        Ok(())
    }

    pub fn change_market<'info>(
        ctx: Context<'_, '_, '_, 'info, ChangeMarket<'info>>,
        new_name: Option<String>,
        new_description: Option<String>,
        mutable: Option<bool>,
        new_price: Option<u64>,
        new_pieces_in_one_wallet: Option<u64>,
    ) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        let clock = &ctx.accounts.clock;

        // Check, that `Market` is in `Suspended` state
        if market.state != MarketState::Suspended {
            return Err(ErrorCode::MarketInInvalidState.into());
        }

        // Check, that `Market` is not in `Ended` state
        if let Some(end_date) = market.end_date {
            if clock.unix_timestamp as u64 > end_date {
                return Err(ErrorCode::MarketIsEnded.into());
            }
        }

        // Check, that `Market` is mutable
        if !market.mutable {
            return Err(ErrorCode::MarketIsImmutable.into());
        }

        if let Some(new_name) = new_name {
            if new_name.len() > NAME_MAX_LEN {
                return Err(ErrorCode::NameIsTooLong.into());
            }

            market.name = puffed_out_string(new_name, NAME_MAX_LEN);
        }

        if let Some(new_description) = new_description {
            if new_description.len() > DESCRIPTION_MAX_LEN {
                return Err(ErrorCode::DescriptionIsTooLong.into());
            }

            market.description = puffed_out_string(new_description, DESCRIPTION_MAX_LEN);
        }

        if let Some(mutable) = mutable {
            market.mutable = mutable;
        }

        if let Some(new_price) = new_price {
            if new_price == 0 {
                return Err(ErrorCode::PriceIsZero.into());
            }

            market.price = new_price;
        }

        // Check is required, because we can overwrite existing value
        if let Some(new_pieces_in_one_wallet) = new_pieces_in_one_wallet {
            market.pieces_in_one_wallet = Some(new_pieces_in_one_wallet);
        }

        Ok(())
    }

    pub fn resume_market<'info>(
        ctx: Context<'_, '_, '_, 'info, ResumeMarket<'info>>,
    ) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        let clock = &ctx.accounts.clock;

        // Check, that `Market` is not in `Ended` state
        if market.state == MarketState::Ended {
            return Err(ErrorCode::MarketIsEnded.into());
        }

        if let Some(end_date) = market.end_date {
            if clock.unix_timestamp as u64 > end_date {
                return Err(ErrorCode::MarketIsEnded.into());
            }
        }

        // Check, that `Market` is in `Suspended` state
        if market.state != MarketState::Suspended {
            return Err(ErrorCode::MarketInInvalidState.into());
        }

        market.state = MarketState::Active;

        Ok(())
    }

    pub fn withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>,
        treasury_owner_bump: u8,
        payout_ticket_bump: u8,
    ) -> ProgramResult {
        let market = &ctx.accounts.market;
        let token_program = &ctx.accounts.token_program;
        let associated_token_program = &ctx.accounts.associated_token_program;
        let system_program = &ctx.accounts.system_program;
        let treasury_holder = Box::new(&ctx.accounts.treasury_holder);
        let treasury_mint = Box::new(&ctx.accounts.treasury_mint);
        let treasury_owner = &ctx.accounts.owner;
        let destination = &ctx.accounts.destination;
        let selling_resource = &ctx.accounts.selling_resource;
        let funder = &ctx.accounts.funder;
        let payer = &ctx.accounts.payer;
        let payout_ticket = &ctx.accounts.payout_ticket;
        let rent = &ctx.accounts.rent;
        let clock = &ctx.accounts.clock;
        let metadata = &ctx.accounts.metadata.to_account_info();

        let selling_resource_key = selling_resource.key().clone();
        let treasury_mint_key = market.treasury_mint.clone();
        let funder_key = funder.key();

        // Check, that `Market` is `Ended`
        if let Some(end_date) = market.end_date {
            if clock.unix_timestamp as u64 <= end_date {
                return Err(ErrorCode::MarketInInvalidState.into());
            }
        } else {
            if market.state != MarketState::Ended {
                return Err(ErrorCode::MarketInInvalidState.into());
            }
        }

        // Check, that provided metadata is correct
        assert_derivation(
            &mpl_token_metadata::id(),
            metadata,
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                selling_resource.resource.as_ref(),
            ],
        )?;

        // Check, that funder is `Creator` or `Market` owner
        let metadata = mpl_token_metadata::state::Metadata::from_account_info(&metadata)?;

        // `Some` mean funder is `Creator`
        // `None` mean funder is `Market` owner
        let funder_creator = if let Some(creators) = metadata.data.creators {
            let funder_creator = creators.iter().find(|&c| c.address == funder_key).cloned();
            if funder_creator.is_none() && funder_key != market.owner {
                return Err(ErrorCode::FunderIsInvalid.into());
            }

            funder_creator
        } else if funder_key != market.owner {
            return Err(ErrorCode::FunderIsInvalid.into());
        } else {
            None
        };

        // Check, that user can withdraw funds(first time)
        if payout_ticket.lamports() > 0 && !payout_ticket.data_is_empty() {
            return Err(ErrorCode::PayoutTicketExists.into());
        }

        let is_native = market.treasury_mint == System::id();

        // Calculate amount
        let total_amount = if is_native {
            treasury_holder.lamports()
        } else {
            let token_account = spl_token::state::Account::unpack(&treasury_holder.data.borrow())?;
            if token_account.owner != treasury_owner.key() {
                return Err(ErrorCode::DerivedKeyInvalid.into());
            }

            token_account.amount
        };
        let amount = if metadata.primary_sale_happened {
            if let Some(funder_creator) = funder_creator {
                let share_bp = (funder_creator.share as u64)
                    .checked_mul(100)
                    .ok_or(ErrorCode::MathOverflow)?;
                total_amount
                    .checked_mul(share_bp)
                    .ok_or(ErrorCode::MathOverflow)?
                    .checked_div(10000)
                    .ok_or(ErrorCode::MathOverflow)?
            } else {
                0
            }
        } else {
            if funder_creator.is_some() && funder_key == market.owner {
                let funder_creator = funder_creator.as_ref().unwrap();

                let x = (total_amount
                    .checked_mul(metadata.data.seller_fee_basis_points as u64)
                    .ok_or(ErrorCode::MathOverflow)?
                    .checked_div(10000)
                    .ok_or(ErrorCode::MathOverflow)?)
                .checked_mul(funder_creator.share as u64)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_div(100)
                .ok_or(ErrorCode::MathOverflow)?;

                let y = total_amount
                    .checked_sub(
                        total_amount
                            .checked_mul(metadata.data.seller_fee_basis_points as u64)
                            .ok_or(ErrorCode::MathOverflow)?
                            .checked_div(10000)
                            .ok_or(ErrorCode::MathOverflow)?,
                    )
                    .ok_or(ErrorCode::MathOverflow)?;

                x.checked_add(y).ok_or(ErrorCode::MathOverflow)?
            } else if let Some(funder_creator) = &funder_creator {
                (total_amount
                    .checked_mul(metadata.data.seller_fee_basis_points as u64)
                    .ok_or(ErrorCode::MathOverflow)?
                    .checked_div(10000)
                    .ok_or(ErrorCode::MathOverflow)?)
                .checked_mul(funder_creator.share as u64)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_div(100)
                .ok_or(ErrorCode::MathOverflow)?
            } else {
                total_amount
                    .checked_sub(
                        total_amount
                            .checked_mul(metadata.data.seller_fee_basis_points as u64)
                            .ok_or(ErrorCode::MathOverflow)?
                            .checked_div(10000)
                            .ok_or(ErrorCode::MathOverflow)?,
                    )
                    .ok_or(ErrorCode::MathOverflow)?
            }
        };

        // Transfer royalties
        let signer_seeds: &[&[&[u8]]] = &[&[
            HOLDER_PREFIX.as_bytes(),
            treasury_mint_key.as_ref(),
            selling_resource_key.as_ref(),
            &[treasury_owner_bump],
        ]];

        if is_native {
            if funder_key != destination.key() {
                return Err(ErrorCode::InvalidFunderDestination.into());
            }

            sys_transfer(
                &treasury_holder.to_account_info(),
                &destination.to_account_info(),
                amount,
                signer_seeds[0],
            )?;
        } else {
            if *treasury_mint.owner != spl_token::id() {
                return Err(ProgramError::InvalidArgument);
            }

            if *treasury_holder.owner != spl_token::id() {
                return Err(ProgramError::InvalidArgument);
            }

            let associated_token_account =
                get_associated_token_address(&funder_key, &market.treasury_mint);

            // Check, that provided destination is associated token account
            if associated_token_account != destination.key() {
                return Err(ErrorCode::InvalidFunderDestination.into());
            }

            // Check, that provided destination is exists
            if destination.lamports() == 0 && destination.data_is_empty() {
                let cpi_program = associated_token_program.to_account_info();
                let cpi_accounts = associated_token::Create {
                    payer: payer.to_account_info(),
                    associated_token: destination.to_account_info(),
                    authority: funder.to_account_info(),
                    mint: treasury_mint.to_account_info(),
                    rent: rent.to_account_info(),
                    token_program: token_program.to_account_info(),
                    system_program: system_program.to_account_info(),
                };
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                associated_token::create(cpi_ctx)?;
            }

            let cpi_program = token_program.to_account_info();
            let cpi_accounts = token::Transfer {
                from: treasury_holder.to_account_info(),
                to: destination.to_account_info(),
                authority: treasury_owner.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            token::transfer(cpi_ctx, amount)?;
        }

        // Create ticket account to prevent twice withdrawal
        sys_create_account(
            &payer.to_account_info(),
            &payout_ticket.to_account_info(),
            rent.minimum_balance(FLAG_ACCOUNT_SIZE),
            FLAG_ACCOUNT_SIZE,
            &id(),
            &[
                PAYOUT_TICKET_PREFIX.as_bytes(),
                market.key().as_ref(),
                funder_key.as_ref(),
                &[payout_ticket_bump],
            ],
        )?;

        Ok(())
    }

    pub fn create_market<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateMarket<'info>>,
        _treasyry_owner_bump: u8,
        name: String,
        description: String,
        mutable: bool,
        price: u64,
        pieces_in_one_wallet: Option<u64>,
        start_date: u64,
        end_date: Option<u64>,
    ) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        let store = &ctx.accounts.store;
        let selling_resource_owner = &ctx.accounts.selling_resource_owner;
        let selling_resource = &ctx.accounts.selling_resource;
        let mint = ctx.accounts.mint.to_account_info();
        let treasury_holder = ctx.accounts.treasury_holder.to_account_info();
        let owner = &ctx.accounts.owner;

        if name.len() > NAME_MAX_LEN {
            return Err(ErrorCode::NameIsTooLong.into());
        }

        if description.len() > DESCRIPTION_MAX_LEN {
            return Err(ErrorCode::DescriptionIsTooLong.into());
        }

        // Pieces in one wallet cannot be greater than Max Supply value
        if pieces_in_one_wallet.is_some()
            && selling_resource.max_supply.is_some()
            && pieces_in_one_wallet.unwrap() > selling_resource.max_supply.unwrap()
        {
            return Err(ErrorCode::PiecesInOneWalletIsTooMuch.into());
        }

        // start_date cannot be in the past
        if start_date < Clock::get().unwrap().unix_timestamp as u64 {
            return Err(ErrorCode::StartDateIsInPast.into());
        }

        // end_date should not be greater than start_date
        if end_date.is_some() && start_date > end_date.unwrap() {
            return Err(ErrorCode::EndDateIsEarlierThanBeginDate.into());
        }

        let is_native = mint.key() == System::id();

        if !is_native {
            if mint.owner != &anchor_spl::token::ID
                || treasury_holder.owner != &anchor_spl::token::ID
            {
                return Err(ProgramError::IllegalOwner);
            }

            if accessor::mint(&treasury_holder)? != *mint.key {
                return Err(ProgramError::InvalidAccountData);
            }

            if accessor::authority(&treasury_holder)? != owner.key() {
                return Err(ProgramError::InvalidAccountData);
            }
        } else {
            // for native SOL we use PDA as a treasury holder
            // because of security reasons(only program can spend this SOL)
            if treasury_holder.key != owner.key {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        // Check selling resource ownership
        assert_keys_equal(selling_resource.owner, selling_resource_owner.key())?;

        market.store = store.key();
        market.selling_resource = selling_resource.key();
        market.treasury_mint = mint.key();
        market.treasury_holder = treasury_holder.key();
        market.treasury_owner = owner.key();
        market.owner = selling_resource_owner.key();
        market.name = puffed_out_string(name, NAME_MAX_LEN);
        market.description = puffed_out_string(description, DESCRIPTION_MAX_LEN);
        market.mutable = mutable;
        market.price = price;
        market.pieces_in_one_wallet = pieces_in_one_wallet;
        market.start_date = start_date;
        market.end_date = end_date;
        market.state = MarketState::Created;

        Ok(())
    }

    pub fn claim_resource<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimResource<'info>>,
        vault_owner_bump: u8,
    ) -> ProgramResult {
        let market = &ctx.accounts.market;
        let selling_resource = &ctx.accounts.selling_resource;
        let vault = &ctx.accounts.vault;
        let metadata = &ctx.accounts.metadata;
        let vault_owner = &ctx.accounts.owner;
        let destination = &ctx.accounts.destination;
        let clock = &ctx.accounts.clock;
        let treasury_holder = &ctx.accounts.treasury_holder;
        let token_program = &ctx.accounts.token_program;

        // Check, that `Market` is `Ended`
        if let Some(end_date) = market.end_date {
            if clock.unix_timestamp as u64 <= end_date {
                return Err(ErrorCode::MarketInInvalidState.into());
            }
        } else {
            if market.state != MarketState::Ended {
                return Err(ErrorCode::MarketInInvalidState.into());
            }
        }

        // Check, that treasury balance is zero
        if treasury_holder.amount != 0 {
            return Err(ErrorCode::TreasuryIsNotEmpty.into());
        }

        // Check, that provided metadata is correct
        assert_derivation(
            &mpl_token_metadata::id(),
            metadata,
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                selling_resource.resource.as_ref(),
            ],
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            VAULT_OWNER_PREFIX.as_bytes(),
            selling_resource.resource.as_ref(),
            selling_resource.store.as_ref(),
            &[vault_owner_bump],
        ]];

        // Update primary sale flag
        let metadata_state = mpl_token_metadata::state::Metadata::from_account_info(&metadata)?;
        if !metadata_state.primary_sale_happened {
            mpl_update_primary_sale_happened_via_token(
                &metadata.to_account_info(),
                &vault_owner.to_account_info(),
                &vault.to_account_info(),
                signer_seeds[0],
            )?;
        }

        // Transfer token(ownership)
        let cpi_program = token_program.to_account_info();
        let cpi_accounts = token::Transfer {
            from: vault.to_account_info(),
            to: destination.to_account_info(),
            authority: vault_owner.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, 1)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(master_edition_bump:u8, vault_owner_bump: u8, max_supply: Option<u64>)]
pub struct InitSellingResource<'info> {
    #[account(has_one=admin)]
    store: Box<Account<'info, Store>>,
    #[account(mut)]
    admin: Signer<'info>,
    #[account(init, payer=admin, space=SellingResource::LEN)]
    selling_resource: Box<Account<'info, SellingResource>>,
    selling_resource_owner: UncheckedAccount<'info>,
    resource_mint: Box<Account<'info, Mint>>,
    #[account(owner=mpl_token_metadata::id())]
    master_edition: UncheckedAccount<'info>,
    #[account(owner=mpl_token_metadata::id())]
    metadata: UncheckedAccount<'info>,
    #[account(mut, has_one=owner)]
    vault: Box<Account<'info, TokenAccount>>,
    #[account(seeds=[VAULT_OWNER_PREFIX.as_bytes(), resource_mint.key().as_ref(), store.key().as_ref()], bump=vault_owner_bump)]
    owner: UncheckedAccount<'info>,
    #[account(mut)]
    resource_token: UncheckedAccount<'info>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(name: String, description: String)]
pub struct CreateStore<'info> {
    #[account(mut)]
    admin: Signer<'info>,
    #[account(init, space=Store::LEN, payer=admin)]
    store: Box<Account<'info, Store>>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(trade_history_bump:u8, vault_owner_bump: u8)]
pub struct Buy<'info> {
    #[account(has_one=treasury_holder)]
    market: Account<'info, Market>,
    #[account(mut)]
    selling_resource: Box<Account<'info, SellingResource>>,
    #[account(mut)]
    user_token_account: UncheckedAccount<'info>,
    user_wallet: Signer<'info>,
    #[account(init_if_needed, seeds=[HISTORY_PREFIX.as_bytes(), user_wallet.key().as_ref(), market.key().as_ref()], bump=trade_history_bump, payer=user_wallet)]
    trade_history: Account<'info, TradeHistory>,
    #[account(mut)]
    treasury_holder: UncheckedAccount<'info>,
    // Will be created by `mpl_token_metadata`
    #[account(mut)]
    new_metadata: UncheckedAccount<'info>,
    // Will be created by `mpl_token_metadata`
    #[account(mut)]
    new_edition: UncheckedAccount<'info>,
    #[account(mut, owner=mpl_token_metadata::id())]
    master_edition: UncheckedAccount<'info>,
    #[account(mut)]
    new_mint: Box<Account<'info, Mint>>,
    // Will be created by `mpl_token_metadata`
    #[account(mut)]
    edition_marker: UncheckedAccount<'info>,
    #[account(mut, has_one=owner)]
    vault: Box<Account<'info, TokenAccount>>,
    #[account(seeds=[VAULT_OWNER_PREFIX.as_bytes(), selling_resource.resource.as_ref(), selling_resource.store.as_ref()], bump=vault_owner_bump)]
    owner: UncheckedAccount<'info>,
    #[account(owner=mpl_token_metadata::id())]
    master_edition_metadata: UncheckedAccount<'info>,
    clock: Sysvar<'info, Clock>,
    rent: Sysvar<'info, Rent>,
    token_metadata_program: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CloseMarket<'info> {
    #[account(mut, has_one=owner)]
    market: Account<'info, Market>,
    owner: Signer<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction()]
pub struct SuspendMarket<'info> {
    #[account(mut, has_one=owner)]
    market: Account<'info, Market>,
    owner: Signer<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction()]
pub struct ResumeMarket<'info> {
    #[account(mut, has_one=owner)]
    market: Account<'info, Market>,
    owner: Signer<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(new_name: Option<String>, new_description: Option<String>, mutable: Option<bool>, new_price: Option<u64>, new_pieces_in_one_wallet: Option<u64>)]
pub struct ChangeMarket<'info> {
    #[account(mut, has_one=owner)]
    market: Account<'info, Market>,
    owner: Signer<'info>,
    clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(treasury_owner_bump: u8, payout_ticket_bump: u8)]
pub struct Withdraw<'info> {
    #[account(has_one=treasury_holder, has_one=selling_resource, has_one=treasury_mint)]
    market: Box<Account<'info, Market>>,
    selling_resource: Box<Account<'info, SellingResource>>,
    #[account(owner=mpl_token_metadata::id())]
    metadata: UncheckedAccount<'info>,
    #[account(mut)]
    treasury_holder: UncheckedAccount<'info>,
    treasury_mint: UncheckedAccount<'info>,
    #[account(seeds=[HOLDER_PREFIX.as_bytes(), market.treasury_mint.as_ref(), market.selling_resource.as_ref()], bump=treasury_owner_bump)]
    owner: UncheckedAccount<'info>,
    #[account(mut)]
    destination: UncheckedAccount<'info>,
    funder: UncheckedAccount<'info>,
    payer: Signer<'info>,
    #[account(mut, seeds=[PAYOUT_TICKET_PREFIX.as_bytes(), market.key().as_ref(), funder.key().as_ref()], bump=payout_ticket_bump)]
    payout_ticket: UncheckedAccount<'info>,
    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(vault_owner_bump: u8)]
pub struct ClaimResource<'info> {
    #[account(has_one=selling_resource, has_one=treasury_holder)]
    market: Account<'info, Market>,
    treasury_holder: Box<Account<'info, TokenAccount>>,
    #[account(has_one=vault, constraint = selling_resource.owner == selling_resource_owner.key())]
    selling_resource: Account<'info, SellingResource>,
    selling_resource_owner: Signer<'info>,
    #[account(mut, has_one=owner)]
    vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, owner=mpl_token_metadata::id())]
    metadata: UncheckedAccount<'info>,
    #[account(seeds=[VAULT_OWNER_PREFIX.as_bytes(), selling_resource.resource.key().as_ref(), selling_resource.store.as_ref()], bump=vault_owner_bump)]
    owner: UncheckedAccount<'info>,
    #[account(mut)]
    destination: Box<Account<'info, TokenAccount>>,
    clock: Sysvar<'info, Clock>,
    token_program: Program<'info, Token>,
    token_metadata_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(treasury_owner_bump: u8, name: String, description: String, mutable: bool, price: u64, pieces_in_one_wallet: Option<u64>, start_date: u64, end_date: Option<u64>)]
pub struct CreateMarket<'info> {
    #[account(init, space=Market::LEN, payer=selling_resource_owner)]
    market: Box<Account<'info, Market>>,
    store: Box<Account<'info, Store>>,
    #[account(mut)]
    selling_resource_owner: Signer<'info>,
    #[account(mut, has_one=store)]
    selling_resource: Box<Account<'info, SellingResource>>,
    mint: UncheckedAccount<'info>,
    #[account(mut)]
    treasury_holder: UncheckedAccount<'info>,
    #[account(seeds=[HOLDER_PREFIX.as_bytes(), mint.key().as_ref(), selling_resource.key().as_ref()], bump=treasury_owner_bump)]
    owner: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}
