use std::cell::RefMut;

use anchor_lang::prelude::*;
use arrayref::array_ref;
use mpl_token_metadata::state::{MAX_NAME_LENGTH, MAX_URI_LENGTH};
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;

use crate::{
    constants::{CONFIG_ARRAY_START, CONFIG_LINE_SIZE, SWAP_REMOVE_FEATURE_INDEX},
    is_feature_active, CandyError, CandyMachine, ConfigLine,
};

/// Add multiple config lines to the candy machine.
#[derive(Accounts)]
pub struct AddConfigLines<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    authority: Signer<'info>,
}

pub fn handle_add_config_lines(
    ctx: Context<AddConfigLines>,
    index: u32,
    config_lines: Vec<ConfigLine>,
) -> Result<()> {
    let candy_machine = &mut ctx.accounts.candy_machine;
    let account = candy_machine.to_account_info();

    // No risk overflow because you literally cant store this many in an account
    // going beyond u32 only happens with the hidden store candies, which dont use this.
    if index > (candy_machine.data.items_available as u32) - 1 {
        return err!(CandyError::IndexGreaterThanLength);
    }
    if candy_machine.data.hidden_settings.is_some() {
        return err!(CandyError::HiddenSettingsConfigsDoNotHaveConfigLines);
    } else if is_feature_active(&candy_machine.data.uuid, SWAP_REMOVE_FEATURE_INDEX) {
        // checking if the account has the expected size
        let expected_size = CONFIG_ARRAY_START
            + 4
            + (candy_machine.data.items_available as usize) * CONFIG_LINE_SIZE
            + 4
            + ((candy_machine
                .data
                .items_available
                .checked_div(8)
                .ok_or(CandyError::NumericalOverflowError)?
                + 1) as usize)
            + 4
            + (candy_machine.data.items_available as usize) * 4;

        let current_size = account.data_len();

        if current_size < expected_size {
            // we can only increase the account by a maximum of MAX_PERMITTED_DATA_INCREASE
            // per call, so we need to divide the increments between multiple calls to
            // add_config_lines (this only happens when the CLI does not allocate all the
            // space needed for the swap_remove mint index array

            let increased_size = current_size
                + std::cmp::min(expected_size - current_size, MAX_PERMITTED_DATA_INCREASE);

            if account.realloc(increased_size, false).is_err() {
                return err!(CandyError::CandyMachineReallocFailed);
            }

            msg!("Accoount realloc by {} bytes", increased_size - current_size);

            // sanity check: we need to make sure that we have space to save the indices
            // of the config lines being added

            let position = (index as usize)
                .checked_add(config_lines.len())
                .ok_or(CandyError::NumericalOverflowError)?;

            let needed_size = CONFIG_ARRAY_START
                + 4
                + (candy_machine.data.items_available as usize) * CONFIG_LINE_SIZE
                + 4
                + ((candy_machine
                    .data
                    .items_available
                    .checked_div(8)
                    .ok_or(CandyError::NumericalOverflowError)?
                    + 1) as usize)
                + 4
                + position * 4;

            if increased_size < needed_size {
                // the only situation that this happens is when trying to add config
                // lines in reverse order (from the last index to the first)
                return err!(CandyError::IndexGreaterThanAccountLength);
            }
        }
    }

    let current_count = get_config_count(&account.data.borrow_mut())?;
    let mut data = account.data.borrow_mut();
    let mut fixed_config_lines = Vec::with_capacity(config_lines.len());

    for line in &config_lines {
        let array_of_zeroes = vec![0u8; MAX_NAME_LENGTH - line.name.len()];
        let name = line.name.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();

        let array_of_zeroes = vec![0u8; MAX_URI_LENGTH - line.uri.len()];
        let uri = line.uri.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        fixed_config_lines.push(ConfigLine { name, uri })
    }

    let as_vec = fixed_config_lines.try_to_vec()?;
    // remove unneeded u32 because we're just gonna edit the u32 at the front
    let serialized: &[u8] = &as_vec.as_slice()[4..];

    let position = CONFIG_ARRAY_START + 4 + (index as usize) * CONFIG_LINE_SIZE;

    let array_slice: &mut [u8] =
        &mut data[position..position + fixed_config_lines.len() * CONFIG_LINE_SIZE];

    array_slice.copy_from_slice(serialized);

    let bit_mask_vec_start = CONFIG_ARRAY_START
        + 4
        + (candy_machine.data.items_available as usize) * CONFIG_LINE_SIZE
        + 4;
    // (unordered) indices for the mint
    let indices_vec_start =
        if is_feature_active(&candy_machine.data.uuid, SWAP_REMOVE_FEATURE_INDEX) {
            bit_mask_vec_start
                + (candy_machine
                    .data
                    .items_available
                    .checked_div(8)
                    .ok_or(CandyError::NumericalOverflowError)?
                    + 1) as usize
                + 4
        } else {
            // we are not going to use this value
            0
        };

    let mut new_count = current_count;
    for i in 0..fixed_config_lines.len() {
        let position = (index as usize)
            .checked_add(i)
            .ok_or(CandyError::NumericalOverflowError)?;
        let my_position_in_vec = bit_mask_vec_start
            + position
                .checked_div(8)
                .ok_or(CandyError::NumericalOverflowError)?;
        let position_from_right = 7 - position
            .checked_rem(8)
            .ok_or(CandyError::NumericalOverflowError)?;
        let mask = u8::pow(2, position_from_right as u32);

        let old_value_in_vec = data[my_position_in_vec];
        data[my_position_in_vec] |= mask;
        msg!(
            "My position in vec is {} my mask is going to be {}, the old value is {}",
            position,
            mask,
            old_value_in_vec
        );
        msg!(
            "My new value is {} and my position from right is {}",
            data[my_position_in_vec],
            position_from_right
        );
        if old_value_in_vec != data[my_position_in_vec] {
            msg!("Increasing count");
            new_count = new_count
                .checked_add(1)
                .ok_or(CandyError::NumericalOverflowError)?;

            if is_feature_active(&candy_machine.data.uuid, SWAP_REMOVE_FEATURE_INDEX) {
                let mint_index = indices_vec_start + (position as usize) * 4;
                data[mint_index..mint_index + 4]
                    .copy_from_slice(&u32::to_le_bytes(position as u32));
            }
        }
    }

    // plug in new count.
    data[CONFIG_ARRAY_START..CONFIG_ARRAY_START + 4]
        .copy_from_slice(&(new_count as u32).to_le_bytes());

    Ok(())
}

pub fn get_config_count(data: &RefMut<&mut [u8]>) -> Result<usize> {
    return Ok(u32::from_le_bytes(*array_ref![data, CONFIG_ARRAY_START, 4]) as usize);
}
