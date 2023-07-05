use crate::{error::BubblegumError, state::metaplex_adapter::MetadataArgs, ASSET_PREFIX};
use anchor_lang::{
    prelude::*,
    solana_program::{program_memory::sol_memcmp, pubkey::PUBKEY_BYTES},
};
use mpl_token_metadata::{
    instruction::MetadataDelegateRole,
    pda::{find_collection_authority_account, find_metadata_delegate_record_account},
    state::{CollectionAuthorityRecord, Metadata, MetadataDelegateRecord, TokenMetadataAccount},
};
use spl_account_compression::Node;

/// Assert that the provided MetadataArgs are compatible with MPL `Data`
pub fn assert_metadata_is_mpl_compatible(metadata: &MetadataArgs) -> Result<()> {
    if metadata.name.len() > mpl_token_metadata::state::MAX_NAME_LENGTH {
        return Err(BubblegumError::MetadataNameTooLong.into());
    }

    if metadata.symbol.len() > mpl_token_metadata::state::MAX_SYMBOL_LENGTH {
        return Err(BubblegumError::MetadataSymbolTooLong.into());
    }

    if metadata.uri.len() > mpl_token_metadata::state::MAX_URI_LENGTH {
        return Err(BubblegumError::MetadataUriTooLong.into());
    }

    if metadata.seller_fee_basis_points > 10000 {
        return Err(BubblegumError::MetadataBasisPointsTooHigh.into());
    }
    if !metadata.creators.is_empty() {
        if metadata.creators.len() > mpl_token_metadata::state::MAX_CREATOR_LIMIT {
            return Err(BubblegumError::CreatorsTooLong.into());
        }

        let mut total: u8 = 0;
        for i in 0..metadata.creators.len() {
            let creator = &metadata.creators[i];
            for iter in metadata.creators.iter().skip(i + 1) {
                if iter.address == creator.address {
                    return Err(BubblegumError::DuplicateCreatorAddress.into());
                }
            }
            total = total
                .checked_add(creator.share)
                .ok_or(BubblegumError::CreatorShareTotalMustBe100)?;
        }
        if total != 100 {
            return Err(BubblegumError::CreatorShareTotalMustBe100.into());
        }
    }
    Ok(())
}

pub fn replace_leaf<'info>(
    seed: &Pubkey,
    bump: u8,
    compression_program: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    merkle_tree: &AccountInfo<'info>,
    log_wrapper: &AccountInfo<'info>,
    remaining_accounts: &[AccountInfo<'info>],
    root_node: Node,
    previous_leaf: Node,
    new_leaf: Node,
    index: u32,
) -> Result<()> {
    let seeds = &[seed.as_ref(), &[bump]];
    let authority_pda_signer = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(
        compression_program.clone(),
        spl_account_compression::cpi::accounts::Modify {
            authority: authority.clone(),
            merkle_tree: merkle_tree.clone(),
            noop: log_wrapper.clone(),
        },
        authority_pda_signer,
    )
    .with_remaining_accounts(remaining_accounts.to_vec());
    spl_account_compression::cpi::replace_leaf(cpi_ctx, root_node, previous_leaf, new_leaf, index)
}

pub fn append_leaf<'info>(
    seed: &Pubkey,
    bump: u8,
    compression_program: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    merkle_tree: &AccountInfo<'info>,
    log_wrapper: &AccountInfo<'info>,
    leaf_node: Node,
) -> Result<()> {
    let seeds = &[seed.as_ref(), &[bump]];
    let authority_pda_signer = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(
        compression_program.clone(),
        spl_account_compression::cpi::accounts::Modify {
            authority: authority.clone(),
            merkle_tree: merkle_tree.clone(),
            noop: log_wrapper.clone(),
        },
        authority_pda_signer,
    );
    spl_account_compression::cpi::append(cpi_ctx, leaf_node)
}

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

pub fn cmp_bytes(a: &[u8], b: &[u8], size: usize) -> bool {
    sol_memcmp(a, b, size) == 0
}

pub fn assert_pubkey_equal(
    a: &Pubkey,
    b: &Pubkey,
    error: Option<anchor_lang::error::Error>,
) -> Result<()> {
    if !cmp_pubkeys(a, b) {
        if let Some(err) = error {
            Err(err)
        } else {
            Err(BubblegumError::PublicKeyMismatch.into())
        }
    } else {
        Ok(())
    }
}

pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
    error: Option<error::Error>,
) -> Result<u8> {
    let (key, bump) = Pubkey::find_program_address(path, program_id);
    if !cmp_pubkeys(&key, account.key) {
        if let Some(err) = error {
            msg!("Derivation {:?}", err);
            Err(err)
        } else {
            msg!("DerivedKeyInvalid");
            Err(ProgramError::InvalidInstructionData.into())
        }
    } else {
        Ok(bump)
    }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<()> {
    if !cmp_pubkeys(account.owner, owner) {
        //todo add better errors
        Err(ProgramError::IllegalOwner.into())
    } else {
        Ok(())
    }
}

pub fn get_asset_id(tree_id: &Pubkey, nonce: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[
            ASSET_PREFIX.as_ref(),
            tree_id.as_ref(),
            &nonce.to_le_bytes(),
        ],
        &crate::id(),
    )
    .0
}

// Checks both delegate types: old collection_authority_record and newer
// metadata_delegate
pub fn assert_has_collection_authority(
    collection_data: &Metadata,
    mint: &Pubkey,
    collection_authority: &Pubkey,
    delegate_record: Option<&AccountInfo>,
) -> Result<()> {
    // Mint is the correct one for the metadata account.
    if collection_data.mint != *mint {
        return Err(BubblegumError::MetadataMintMismatch.into());
    }

    if let Some(record_info) = delegate_record {
        let (ca_pda, ca_bump) = find_collection_authority_account(mint, collection_authority);
        let (md_pda, md_bump) = find_metadata_delegate_record_account(
            mint,
            MetadataDelegateRole::Collection,
            &collection_data.update_authority,
            collection_authority,
        );

        let data = record_info.try_borrow_data()?;
        if data.len() == 0 {
            return Err(BubblegumError::InvalidCollectionAuthority.into());
        }

        if record_info.key == &ca_pda {
            let record = CollectionAuthorityRecord::safe_deserialize(&data)?;
            if record.bump != ca_bump {
                return Err(BubblegumError::InvalidCollectionAuthority.into());
            }

            match record.update_authority {
                Some(update_authority) => {
                    if update_authority != collection_data.update_authority {
                        return Err(BubblegumError::InvalidCollectionAuthority.into());
                    }
                }
                None => return Err(BubblegumError::InvalidCollectionAuthority.into()),
            }
        } else if record_info.key == &md_pda {
            let record = MetadataDelegateRecord::safe_deserialize(&data)?;
            if record.bump != md_bump {
                return Err(BubblegumError::InvalidCollectionAuthority.into());
            }

            if record.update_authority != collection_data.update_authority {
                return Err(BubblegumError::InvalidCollectionAuthority.into());
            }
        } else {
            return Err(BubblegumError::InvalidDelegateRecord.into());
        }
    } else if collection_data.update_authority != *collection_authority {
        return Err(BubblegumError::InvalidCollectionAuthority.into());
    }
    Ok(())
}
