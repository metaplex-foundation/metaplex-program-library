use crate::{
    error::ErrorCode,
    state::{MarketState, SellingResourceState},
    utils::*,
    Buy,
};
use anchor_lang::prelude::*;
use anchor_lang::{
    solana_program::{program::invoke, system_instruction},
    System,
};
use anchor_spl::token;
use mpl_token_metadata::utils::get_supply_off_master_edition;

impl<'info> Buy<'info> {
    pub fn process(&mut self, _trade_history_bump: u8, vault_owner_bump: u8) -> Result<()> {
        let market = &mut self.market;
        let selling_resource = &mut self.selling_resource;
        let user_token_account = Box::new(&self.user_token_account);
        let user_wallet = &mut self.user_wallet;
        let trade_history = &mut self.trade_history;
        let treasury_holder = Box::new(&self.treasury_holder);
        let new_metadata = Box::new(&self.new_metadata);
        let new_edition = Box::new(&self.new_edition);
        let master_edition = Box::new(&self.master_edition);
        let new_mint = &mut self.new_mint;
        let edition_marker_info = &mut self.edition_marker.to_account_info();
        let vault = &mut self.vault;
        let owner = Box::new(&self.owner);
        let new_token_account = &self.new_token_account;
        let master_edition_metadata = Box::new(&self.master_edition_metadata);
        let clock = &self.clock;
        let rent = &self.rent;
        let token_program = &self.token_program;
        let system_program = &self.system_program;

        let metadata_mint = selling_resource.resource.clone();
        // do supply +1 to increase master edition supply
        let edition = get_supply_off_master_edition(&master_edition.to_account_info())?
            .checked_add(1)
            .ok_or(ErrorCode::MathOverflow)?;

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

        if market.state != MarketState::Active {
            market.state = MarketState::Active;
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

        market.funds_collected = market
            .funds_collected
            .checked_add(market.price)
            .ok_or(ErrorCode::MathOverflow)?;

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

        mpl_update_primary_sale_happened_via_token(
            &new_metadata.to_account_info(),
            &user_wallet.to_account_info(),
            &new_token_account.to_account_info(),
            &[],
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
                market.state = MarketState::Ended;
            }
        }

        Ok(())
    }
}
