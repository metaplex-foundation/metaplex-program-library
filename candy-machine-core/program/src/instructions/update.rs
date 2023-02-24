use anchor_lang::prelude::*;
use mpl_token_metadata::state::MAX_SYMBOL_LENGTH;

use crate::{utils::fixed_length_string, CandyError, CandyMachine, CandyMachineData};

pub fn update(ctx: Context<Update>, data: CandyMachineData) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;

    if (data.items_available != candy_machine.data.items_available)
        && data.hidden_settings.is_none()
    {
        return err!(CandyError::CannotChangeNumberOfLines);
    }

    if candy_machine.data.items_available > 0
        && candy_machine.data.hidden_settings.is_none()
        && data.hidden_settings.is_some()
    {
        return err!(CandyError::CannotSwitchToHiddenSettings);
    }

    let symbol = fixed_length_string(data.symbol.clone(), MAX_SYMBOL_LENGTH)?;
    // validates the config data settings
    data.validate()?;

    if let Some(config_lines) = &candy_machine.data.config_line_settings {
        if let Some(new_config_lines) = &data.config_line_settings {
            // it is only possible to update the config lines settings if the
            // new values are equal or smaller than the current ones
            if config_lines.name_length < new_config_lines.name_length
                || config_lines.uri_length < new_config_lines.uri_length
            {
                return err!(CandyError::CannotIncreaseLength);
            }

            if config_lines.is_sequential != new_config_lines.is_sequential
                && candy_machine.items_redeemed > 0
            {
                return err!(CandyError::CannotChangeSequentialIndexGeneration);
            }
        }
    } else if data.config_line_settings.is_some() {
        return err!(CandyError::CannotSwitchFromHiddenSettings);
    }

    candy_machine.data = data;
    candy_machine.data.symbol = symbol;

    Ok(())
}

/// Update the candy machine state.
#[derive(Accounts)]
pub struct Update<'info> {
    /// Candy Machine account.
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,

    /// Authority of the candy machine.
    authority: Signer<'info>,
}
