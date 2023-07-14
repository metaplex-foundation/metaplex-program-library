use crate::{
    pda::MARKER,
    state::{EditionMarkerV2, MasterEdition, MasterEditionV2, EDITION_MARKER_BIT_SIZE},
};

use super::*;

pub(crate) fn burn_nonfungible_edition(
    ctx: &Context<Burn>,
    edition_close_authority: bool,
    token_standard: &TokenStandard,
) -> ProgramResult {
    let edition_info = ctx.accounts.edition_info.unwrap();

    let master_edition_mint_info = ctx
        .accounts
        .master_edition_mint_info
        .ok_or(MetadataError::MissingMasterEditionMintAccount)?;

    let master_edition_info = ctx
        .accounts
        .master_edition_info
        .ok_or(MetadataError::MissingMasterEditionAccount)?;

    let master_edition_token_info = ctx
        .accounts
        .master_edition_token_info
        .ok_or(MetadataError::MissingMasterEditionTokenAccount)?;

    let edition_marker_info = ctx
        .accounts
        .edition_marker_info
        .ok_or(MetadataError::MissingEditionMarkerAccount)?;

    // Ensure the master edition is actually a master edition.
    let master_edition_mint_decimals = get_mint_decimals(master_edition_mint_info)?;
    let master_edition_mint_supply = get_mint_supply(master_edition_mint_info)?;

    if !is_master_edition(
        master_edition_info,
        master_edition_mint_decimals,
        master_edition_mint_supply,
    ) {
        return Err(MetadataError::NotAMasterEdition.into());
    }

    // Ensure the print edition is actually a print edition.
    let print_edition_mint_decimals = get_mint_decimals(ctx.accounts.mint_info)?;
    let print_edition_mint_supply = get_mint_supply(ctx.accounts.mint_info)?;

    if !is_print_edition(
        edition_info,
        print_edition_mint_decimals,
        print_edition_mint_supply,
    ) {
        return Err(MetadataError::NotAPrintEdition.into());
    }

    // Master Edition token account checks.
    let master_edition_token_account: TokenAccount = assert_initialized(master_edition_token_info)?;

    if master_edition_token_account.mint != *master_edition_mint_info.key {
        return Err(MetadataError::MintMismatch.into());
    }

    if master_edition_token_account.amount < 1 {
        return Err(MetadataError::InsufficientTokenBalance.into());
    }

    // Master and Print editions are valid PDAs for their given mints.
    let master_edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        master_edition_mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    assert_derivation(&crate::ID, master_edition_info, &master_edition_info_path)
        .map_err(|_| MetadataError::InvalidMasterEdition)?;

    let print_edition_info_path = Vec::from([
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        ctx.accounts.mint_info.key.as_ref(),
        EDITION.as_bytes(),
    ]);
    let bump = assert_derivation(&crate::ID, edition_info, &print_edition_info_path)
        .map_err(|_| MetadataError::InvalidPrintEdition)?;

    let edition_seeds = &[
        PREFIX.as_bytes(),
        crate::ID.as_ref(),
        ctx.accounts.mint_info.key.as_ref(),
        EDITION.as_bytes(),
        &[bump],
    ];

    let print_edition = Edition::from_account_info(edition_info)?;

    // Print Edition actually belongs to the master edition.
    if print_edition.parent != *master_edition_info.key {
        return Err(MetadataError::PrintEditionDoesNotMatchMasterEdition.into());
    }

    if token_standard == &TokenStandard::ProgrammableNonFungibleEdition {
        // Ensure we were passed the correct edition marker PDA.
        let edition_marker_info_path = Vec::from([
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            master_edition_mint_info.key.as_ref(),
            EDITION.as_bytes(),
            MARKER.as_bytes(),
        ]);
        assert_derivation(&crate::ID, edition_marker_info, &edition_marker_info_path)
            .map_err(|_| MetadataError::InvalidEditionMarker)?;
    } else {
        // Which edition marker is this edition in
        let edition_marker_number = print_edition
            .edition
            .checked_div(EDITION_MARKER_BIT_SIZE)
            .ok_or(MetadataError::NumericalOverflowError)?;
        let edition_marker_number_str = edition_marker_number.to_string();

        // Ensure we were passed the correct edition marker PDA.
        let edition_marker_info_path = Vec::from([
            PREFIX.as_bytes(),
            crate::ID.as_ref(),
            master_edition_mint_info.key.as_ref(),
            EDITION.as_bytes(),
            edition_marker_number_str.as_bytes(),
        ]);
        assert_derivation(&crate::ID, edition_marker_info, &edition_marker_info_path)
            .map_err(|_| MetadataError::InvalidEditionMarker)?;
    }

    // Burn the SPL token
    let params = TokenBurnParams {
        mint: ctx.accounts.mint_info.clone(),
        source: ctx.accounts.token_info.clone(),
        authority: ctx.accounts.authority_info.clone(),
        token_program: ctx.accounts.spl_token_program_info.clone(),
        amount: 1,
        authority_signer_seeds: None,
    };
    spl_token_burn(params)?;

    let params = TokenCloseParams {
        token_program: ctx.accounts.spl_token_program_info.clone(),
        account: ctx.accounts.token_info.clone(),
        destination: ctx.accounts.authority_info.clone(),
        owner: if edition_close_authority {
            edition_info.clone()
        } else {
            ctx.accounts.authority_info.clone()
        },
        authority_signer_seeds: if edition_close_authority {
            Some(edition_seeds.as_slice())
        } else {
            None
        },
    };
    spl_token_close(params)?;

    //       **EDITION HOUSEKEEPING**
    // Set the particular bit for this edition to 0 to allow reprinting,
    // IF the print edition owner is also the master edition owner.
    // Otherwise leave the bit set to 1 to disallow reprinting.
    if token_standard == &TokenStandard::ProgrammableNonFungibleEdition {
        let mut edition_marker: EditionMarkerV2 =
            EditionMarkerV2::from_account_info(edition_marker_info)?;

        let owner_is_the_same =
            *ctx.accounts.authority_info.key == master_edition_token_account.owner;

        if owner_is_the_same {
            let (index, mask) = EditionMarkerV2::get_index_and_mask(print_edition.edition)?;
            edition_marker.ledger[index] ^= mask;
        }

        // If the entire edition marker is empty, then we can close the account.
        // Otherwise, serialize the new edition marker and update the account data.
        if edition_marker.ledger.iter().all(|i| *i == 0) {
            close_program_account(
                edition_marker_info,
                ctx.accounts.authority_info,
                Key::EditionMarkerV2,
            )?;
        } else {
            edition_marker.save(
                edition_marker_info,
                ctx.accounts.authority_info,
                ctx.accounts.system_program_info,
            )?;
        }
    } else {
        let mut edition_marker: EditionMarker =
            EditionMarker::from_account_info(edition_marker_info)?;

        let owner_is_the_same =
            *ctx.accounts.authority_info.key == master_edition_token_account.owner;

        if owner_is_the_same {
            let (index, mask) = EditionMarker::get_index_and_mask(print_edition.edition)?;
            edition_marker.ledger[index] ^= mask;
        }

        // If the entire edition marker is empty, then we can close the account.
        // Otherwise, serialize the new edition marker and update the account data.
        if edition_marker.ledger.iter().all(|i| *i == 0) {
            close_program_account(
                edition_marker_info,
                ctx.accounts.authority_info,
                Key::EditionMarker,
            )?;
        } else {
            edition_marker.save(edition_marker_info)?;
        }
    }

    // Decrement the suppply on the master edition now that we've successfully burned a print.
    let mut master_edition: MasterEditionV2 =
        MasterEditionV2::from_account_info(master_edition_info)?;
    master_edition.supply = master_edition
        .supply
        .checked_sub(1)
        .ok_or(MetadataError::NumericalOverflowError)?;

    master_edition.save(master_edition_info)?;

    close_program_account(
        ctx.accounts.metadata_info,
        ctx.accounts.authority_info,
        Key::MetadataV1,
    )?;
    close_program_account(edition_info, ctx.accounts.authority_info, Key::EditionV1)?;

    Ok(())
}
