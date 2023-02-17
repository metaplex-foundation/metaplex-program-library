use anchor_lang::prelude::*;
use mpl_token_metadata::{
    instruction::{approve_collection_authority, revoke_collection_authority},
    state::{Metadata, TokenMetadataAccount},
};
use solana_program::program::{invoke, invoke_signed};

use crate::{cmp_pubkeys, constants::AUTHORITY_SEED, CandyError, CandyMachine};

pub fn set_collection(ctx: Context<SetCollection>) -> Result<()> {
    let accounts = ctx.accounts;
    let candy_machine = &mut accounts.candy_machine;

    if candy_machine.items_redeemed > 0 {
        return err!(CandyError::NoChangingCollectionDuringMint);
    } else if !cmp_pubkeys(accounts.collection_mint.key, &candy_machine.collection_mint) {
        return err!(CandyError::MintMismatch);
    }

    // Revoking old collection authority

    let revoke_collection_infos = vec![
        accounts.collection_authority_record.to_account_info(),
        accounts.authority_pda.to_account_info(),
        accounts.collection_metadata.to_account_info(),
        accounts.collection_mint.to_account_info(),
    ];

    let cm_key = candy_machine.key();

    let authority_seeds = [
        AUTHORITY_SEED.as_bytes(),
        cm_key.as_ref(),
        &[*ctx.bumps.get("authority_pda").unwrap()],
    ];

    invoke_signed(
        &revoke_collection_authority(
            accounts.token_metadata_program.key(),
            accounts.collection_authority_record.key(),
            accounts.authority_pda.key(),
            accounts.authority_pda.key(),
            accounts.collection_metadata.key(),
            accounts.collection_mint.key(),
        ),
        revoke_collection_infos.as_slice(),
        &[&authority_seeds],
    )?;

    candy_machine.collection_mint = accounts.new_collection_mint.key();

    let approve_collection_authority_helper_accounts = ApproveCollectionAuthorityHelperAccounts {
        payer: accounts.payer.to_account_info(),
        authority_pda: accounts.authority_pda.to_account_info(),
        collection_update_authority: accounts.new_collection_update_authority.to_account_info(),
        collection_mint: accounts.new_collection_mint.to_account_info(),
        collection_metadata: accounts.new_collection_metadata.to_account_info(),
        collection_authority_record: accounts.new_collection_authority_record.to_account_info(),
        token_metadata_program: accounts.token_metadata_program.to_account_info(),
        system_program: accounts.system_program.to_account_info(),
    };

    approve_collection_authority_helper(approve_collection_authority_helper_accounts)?;

    Ok(())
}

pub fn approve_collection_authority_helper(
    accounts: ApproveCollectionAuthorityHelperAccounts,
) -> Result<()> {
    let ApproveCollectionAuthorityHelperAccounts {
        payer,
        authority_pda,
        collection_update_authority,
        collection_mint,
        collection_metadata,
        collection_authority_record,
        token_metadata_program,
        system_program,
    } = accounts;

    let collection_data: Metadata = Metadata::from_account_info(&collection_metadata)?;

    if !cmp_pubkeys(
        &collection_data.update_authority,
        &collection_update_authority.key(),
    ) {
        return err!(CandyError::IncorrectCollectionAuthority);
    }

    if !cmp_pubkeys(&collection_data.mint, &collection_mint.key()) {
        return err!(CandyError::MintMismatch);
    }

    let approve_collection_authority_ix = approve_collection_authority(
        token_metadata_program.key(),
        collection_authority_record.key(),
        authority_pda.key(),
        collection_update_authority.key(),
        payer.key(),
        collection_metadata.key(),
        collection_mint.key(),
    );

    if collection_authority_record.data_is_empty() {
        let approve_collection_infos = vec![
            collection_authority_record,
            authority_pda,
            collection_update_authority,
            payer,
            collection_metadata,
            collection_mint,
            system_program,
        ];

        invoke(
            &approve_collection_authority_ix,
            approve_collection_infos.as_slice(),
        )?;
    }

    Ok(())
}

pub struct ApproveCollectionAuthorityHelperAccounts<'info> {
    /// CHECK: account checked in CPI
    pub payer: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub authority_pda: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_update_authority: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_mint: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_metadata: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub collection_authority_record: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub token_metadata_program: AccountInfo<'info>,
    /// CHECK: account checked in CPI
    pub system_program: AccountInfo<'info>,
}

/// Set the collection PDA for the candy machine
#[derive(Accounts)]
pub struct SetCollection<'info> {
    #[account(mut, has_one = authority)]
    candy_machine: Account<'info, CandyMachine>,
    // candy machine authority
    authority: Signer<'info>,
    /// CHECK: account checked in seeds constraint
    #[account(
        mut, seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.to_account_info().key.as_ref()],
        bump
    )]
    authority_pda: UncheckedAccount<'info>,
    // payer of the transaction
    payer: Signer<'info>,
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,
    // update authority of the new collection NFT
    #[account(mut)]
    new_collection_update_authority: Signer<'info>,
    /// CHECK: account checked in CPI
    new_collection_metadata: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    new_collection_mint: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    new_collection_master_edition: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(mut)]
    new_collection_authority_record: UncheckedAccount<'info>,
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}
