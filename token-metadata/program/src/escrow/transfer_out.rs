use crate::{
    instruction::MetadataInstruction,
    state::{
        EscrowAuthority, Metadata, TokenMetadataAccount, TokenOwnedEscrow, ESCROW_PREFIX, PREFIX,
    },
    utils::{assert_derivation, assert_owned_by, assert_signer, close_account_raw},
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_vault::solana_program::msg;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
};

#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferOutOfEscrowArgs {
    pub amount: u64,
}

pub fn transfer_out_of_escrow(
    program_id: Pubkey,
    escrow: Pubkey,
    payer: Pubkey,
    attribute_mint: Pubkey,
    attribute_src: Pubkey,
    attribute_dst: Pubkey,
    attribute_metadata: Pubkey,
    escrow_mint: Pubkey,
    escrow_account: Pubkey,
    authority: Option<Pubkey>,
    amount: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(escrow, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(attribute_mint, false),
        AccountMeta::new(attribute_src, false),
        AccountMeta::new(attribute_dst, false),
        AccountMeta::new_readonly(attribute_metadata, false),
        AccountMeta::new_readonly(escrow_mint, false),
        AccountMeta::new_readonly(escrow_account, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
    ];

    if let Some(authority) = authority {
        accounts.push(AccountMeta::new_readonly(authority, true));
    }

    let data = MetadataInstruction::TransferOutOfEscrow(TransferOutOfEscrowArgs { amount })
        .try_to_vec()
        .unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn process_transfer_out_of_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: TransferOutOfEscrowArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let escrow_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let attribute_mint_info = next_account_info(account_info_iter)?;
    let attribute_src_info = next_account_info(account_info_iter)?;
    let attribute_dst_info = next_account_info(account_info_iter)?;
    let attribute_metadata_info = next_account_info(account_info_iter)?;
    let escrow_mint_info = next_account_info(account_info_iter)?;
    let escrow_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let ata_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let is_using_authority = account_info_iter.len() == 1;

    let maybe_authority_info: Option<&AccountInfo> = if is_using_authority {
        Some(next_account_info(account_info_iter)?)
    } else {
        None
    };

    let authority = maybe_authority_info.unwrap_or(payer_info);

    let toe = TokenOwnedEscrow::from_account_info(escrow_info).unwrap();

    // Owned by token-metadata program.
    assert_owned_by(attribute_metadata_info, program_id)?;
    let _attribute_metadata: Metadata = Metadata::from_account_info(attribute_metadata_info)?;

    let tm_pid = crate::id();
    let mut escrow_seeds = vec![
        PREFIX.as_bytes(),
        tm_pid.as_ref(),
        escrow_mint_info.key.as_ref(),
    ];

    for seed in toe.authority.to_seeds() {
        escrow_seeds.push(seed);
    }

    escrow_seeds.push(ESCROW_PREFIX.as_bytes());

    let bump_seed = &[assert_derivation(&crate::id(), escrow_info, &escrow_seeds)?];

    // Derive the seeds for PDA signing.
    let escrow_authority_seeds = [escrow_seeds, vec![bump_seed]].concat();

    assert_signer(payer_info)?;

    // Allocate the escrow accounts new ATA.
    #[allow(deprecated)]
    let create_escrow_ata_ix = spl_associated_token_account::create_associated_token_account(
        payer_info.key,
        payer_info.key,
        attribute_mint_info.key,
    );

    msg!("Creating ATA");
    invoke(
        &create_escrow_ata_ix,
        &[
            payer_info.clone(),
            attribute_dst_info.clone(),
            attribute_mint_info.clone(),
            system_account_info.clone(),
            token_program_info.clone(),
            ata_program_info.clone(),
            rent_info.clone(),
        ],
    )?;
    msg!("Created ATA");

    // Deserialize the token accounts and perform checks.
    let attribute_src = spl_token::state::Account::unpack(&attribute_src_info.data.borrow())?;
    assert!(attribute_src.mint == *attribute_mint_info.key);
    assert!(attribute_src.delegate.is_none());
    assert!(attribute_src.amount >= args.amount);

    // Check that the authority matches based on the authority type.
    let escrow_account = spl_token::state::Account::unpack(&escrow_account_info.data.borrow())?;
    match toe.authority {
        EscrowAuthority::TokenOwner => {
            assert!(escrow_account.owner == *authority.key);
        }
        EscrowAuthority::Creator(creator) => {
            msg!("Creator: {:#?}", creator);
            msg!("Authority: {:#?}", authority.key);
            assert!(creator == *authority.key);
        }
    }

    let attribute_dst = spl_token::state::Account::unpack(&attribute_dst_info.data.borrow())?;
    assert!(attribute_dst.mint == *attribute_mint_info.key);
    assert!(attribute_dst.delegate.is_none());

    // Transfer the token from the current owner into the escrow.
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        attribute_src_info.key,
        attribute_dst_info.key,
        escrow_info.key,
        &[escrow_info.key],
        args.amount,
    )
    .unwrap();

    msg!("Transferring tokens");
    invoke_signed(
        &transfer_ix,
        &[
            attribute_src_info.clone(),
            attribute_dst_info.clone(),
            escrow_info.clone(),
            token_program_info.clone(),
        ],
        &[&escrow_authority_seeds],
    )?;
    msg!("Transferred tokens");

    // msg!("{:#?}", attribute_src_info);
    // msg!(
    //     "{:#?}",
    //     spl_token::state::Account::unpack(&attribute_src_info.data.borrow())
    // );
    // close_account_raw(payer_info, attribute_src_info)?;
    let close_ix = spl_token::instruction::close_account(
        &spl_token::id(),
        attribute_src_info.key,
        payer_info.key,
        escrow_info.key,
        &[escrow_info.key],
    )
    .unwrap();

    msg!("Closing ATA");
    invoke_signed(
        &close_ix,
        &[
            attribute_src_info.clone(),
            payer_info.clone(),
            escrow_info.clone(),
            token_program_info.clone(),
        ],
        &[&escrow_authority_seeds],
    )?;

    Ok(())
}
