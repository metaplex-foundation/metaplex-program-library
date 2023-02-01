use crate::{
    error::BubblegumError,
    state::{
        leaf_schema::LeafSchema,
        metaplex_adapter::{self, Creator, MetadataArgs, TokenProgramVersion},
        metaplex_anchor::{MasterEdition, MplTokenMetadata, TokenMetadata},
        TreeConfig, Voucher, ASSET_PREFIX, COLLECTION_CPI_PREFIX, TREE_AUTHORITY_SIZE,
        VOUCHER_PREFIX, VOUCHER_SIZE,
    },
    utils::{
        append_leaf, assert_metadata_is_mpl_compatible, assert_pubkey_equal, cmp_bytes,
        cmp_pubkeys, get_asset_id, replace_leaf,
    },
};
use anchor_lang::{
    prelude::*,
    solana_program::{
        account_info::AccountInfo,
        keccak,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        system_instruction,
    },
    system_program::System,
};
use mpl_token_metadata::{
    assertions::collection::{assert_collection_verify_is_valid, assert_has_collection_authority},
    state::CollectionDetails,
};
use spl_account_compression::{
    program::SplAccountCompression, wrap_application_data_v1, Node, Noop,
};
use spl_token::state::Mint as SplMint;
use std::collections::HashSet;

pub mod error;
pub mod state;
pub mod utils;

declare_id!("BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY");

#[derive(Accounts)]
pub struct CreateTree<'info> {
    #[account(
        init,
        seeds = [merkle_tree.key().as_ref()],
        payer = payer,
        space = TREE_AUTHORITY_SIZE,
        bump,
    )]
    pub tree_authority: Account<'info, TreeConfig>,
    #[account(zero)]
    /// CHECK: This account must be all zeros
    pub merkle_tree: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub tree_creator: Signer<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintV1<'info> {
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    pub tree_authority: Account<'info, TreeConfig>,
    /// CHECK: This account is neither written to nor read from.
    pub leaf_owner: AccountInfo<'info>,
    /// CHECK: This account is neither written to nor read from.
    pub leaf_delegate: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: unsafe
    pub merkle_tree: UncheckedAccount<'info>,
    pub payer: Signer<'info>,
    pub tree_delegate: Signer<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintToCollectionV1<'info> {
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    pub tree_authority: Account<'info, TreeConfig>,
    /// CHECK: This account is neither written to nor read from.
    pub leaf_owner: AccountInfo<'info>,
    /// CHECK: This account is neither written to nor read from.
    pub leaf_delegate: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: unsafe
    pub merkle_tree: UncheckedAccount<'info>,
    pub payer: Signer<'info>,
    pub tree_delegate: Signer<'info>,
    pub collection_authority: Signer<'info>,
    /// CHECK: Optional collection authority record PDA.
    /// If there is no collecton authority record PDA then
    /// this must be the Bubblegum program address.
    pub collection_authority_record_pda: UncheckedAccount<'info>,
    /// CHECK: This account is checked in the instruction
    pub collection_mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub collection_metadata: Box<Account<'info, TokenMetadata>>,
    /// CHECK: This account is checked in the instruction
    pub edition_account: UncheckedAccount<'info>,
    /// CHECK: This is just used as a signing PDA.
    #[account(
        seeds = [COLLECTION_CPI_PREFIX.as_ref()],
        bump,
    )]
    pub bubblegum_signer: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub token_metadata_program: Program<'info, MplTokenMetadata>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Burn<'info> {
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    pub tree_authority: Account<'info, TreeConfig>,
    /// CHECK: This account is checked in the instruction
    pub leaf_owner: UncheckedAccount<'info>,
    /// CHECK: This account is checked in the instruction
    pub leaf_delegate: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This account is modified in the downstream program
    pub merkle_tree: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatorVerification<'info> {
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    pub tree_authority: Account<'info, TreeConfig>,
    /// CHECK: This account is checked in the instruction
    pub leaf_owner: UncheckedAccount<'info>,
    /// CHECK: This account is chekced in the instruction
    pub leaf_delegate: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This account is modified in the downstream program
    pub merkle_tree: UncheckedAccount<'info>,
    pub payer: Signer<'info>,
    pub creator: Signer<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CollectionVerification<'info> {
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    pub tree_authority: Account<'info, TreeConfig>,
    /// CHECK: This account is checked in the instruction
    pub leaf_owner: UncheckedAccount<'info>,
    /// CHECK: This account is checked in the instruction
    pub leaf_delegate: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This account is modified in the downstream program
    pub merkle_tree: UncheckedAccount<'info>,
    pub payer: Signer<'info>,
    /// CHECK: This account is checked to be a signer in
    /// the case of `set_and_verify_collection` where
    /// we are actually changing the NFT metadata.
    pub tree_delegate: UncheckedAccount<'info>,
    pub collection_authority: Signer<'info>,
    /// CHECK: Optional collection authority record PDA.
    /// If there is no collecton authority record PDA then
    /// this must be the Bubblegum program address.
    pub collection_authority_record_pda: UncheckedAccount<'info>,
    /// CHECK: This account is checked in the instruction
    pub collection_mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub collection_metadata: Box<Account<'info, TokenMetadata>>,
    /// CHECK: This account is checked in the instruction
    pub edition_account: UncheckedAccount<'info>,
    /// CHECK: This is just used as a signing PDA.
    #[account(
        seeds = [COLLECTION_CPI_PREFIX.as_ref()],
        bump,
    )]
    pub bubblegum_signer: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub token_metadata_program: Program<'info, MplTokenMetadata>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    /// CHECK: This account is neither written to nor read from.
    pub tree_authority: Account<'info, TreeConfig>,
    /// CHECK: This account is checked in the instruction
    pub leaf_owner: UncheckedAccount<'info>,
    /// CHECK: This account is chekced in the instruction
    pub leaf_delegate: UncheckedAccount<'info>,
    /// CHECK: This account is neither written to nor read from.
    pub new_leaf_owner: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This account is modified in the downstream program
    pub merkle_tree: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Delegate<'info> {
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    /// CHECK: This account is neither written to nor read from.
    pub tree_authority: Account<'info, TreeConfig>,
    pub leaf_owner: Signer<'info>,
    /// CHECK: This account is neither written to nor read from.
    pub previous_leaf_delegate: UncheckedAccount<'info>,
    /// CHECK: This account is neither written to nor read from.
    pub new_leaf_delegate: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This account is modified in the downstream program
    pub merkle_tree: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    _root: [u8; 32],
    _data_hash: [u8; 32],
    _creator_hash: [u8; 32],
    nonce: u64,
    _index: u32,
)]
pub struct Redeem<'info> {
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    /// CHECK: This account is neither written to nor read from.
    pub tree_authority: Account<'info, TreeConfig>,
    #[account(mut)]
    pub leaf_owner: Signer<'info>,
    /// CHECK: This account is chekced in the instruction
    pub leaf_delegate: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in cpi
    pub merkle_tree: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [
        VOUCHER_PREFIX.as_ref(),
        merkle_tree.key().as_ref(),
        & nonce.to_le_bytes()
    ],
    payer = leaf_owner,
    space = VOUCHER_SIZE,
    bump
    )]
    pub voucher: Account<'info, Voucher>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelRedeem<'info> {
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    /// CHECK: This account is neither written to nor read from.
    pub tree_authority: Account<'info, TreeConfig>,
    #[account(mut)]
    pub leaf_owner: Signer<'info>,
    #[account(mut)]
    /// CHECK: unsafe
    pub merkle_tree: UncheckedAccount<'info>,
    #[account(
        mut,
        close = leaf_owner,
        seeds = [
        VOUCHER_PREFIX.as_ref(),
        merkle_tree.key().as_ref(),
        & voucher.leaf_schema.nonce().to_le_bytes()
    ],
    bump
    )]
    pub voucher: Account<'info, Voucher>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DecompressV1<'info> {
    #[account(
        mut,
        close = leaf_owner,
        seeds = [
            VOUCHER_PREFIX.as_ref(),
            voucher.merkle_tree.as_ref(),
            voucher.leaf_schema.nonce().to_le_bytes().as_ref()
        ],
        bump
    )]
    pub voucher: Box<Account<'info, Voucher>>,
    #[account(mut)]
    pub leaf_owner: Signer<'info>,
    /// CHECK: versioning is handled in the instruction
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: versioning is handled in the instruction
    #[account(
        mut,
        seeds = [
            ASSET_PREFIX.as_ref(),
            voucher.merkle_tree.as_ref(),
            voucher.leaf_schema.nonce().to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub mint: UncheckedAccount<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: Initialized in Token Metadata Program
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub sysvar_rent: Sysvar<'info, Rent>,
    /// CHECK:
    pub token_metadata_program: Program<'info, MplTokenMetadata>,
    /// CHECK: versioning is handled in the instruction
    pub token_program: UncheckedAccount<'info>,
    /// CHECK:
    pub associated_token_program: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
}

#[derive(Accounts)]
pub struct Compress<'info> {
    #[account(
        seeds = [merkle_tree.key().as_ref()],
        bump,
    )]
    /// CHECK: This account is neither written to nor read from.
    pub tree_authority: UncheckedAccount<'info>,
    /// CHECK: This account is checked in the instruction
    pub leaf_owner: Signer<'info>,
    /// CHECK: This account is chekced in the instruction
    pub leaf_delegate: UncheckedAccount<'info>,
    /// CHECK: This account is not read
    pub merkle_tree: UncheckedAccount<'info>,

    /// CHECK: versioning is handled in the instruction
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    /// CHECK: versioning is handled in the instruction
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub metadata: Box<Account<'info, TokenMetadata>>,
    #[account(mut)]
    pub master_edition: Box<Account<'info, MasterEdition>>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    /// CHECK:
    pub token_program: UncheckedAccount<'info>,
    /// CHECK:
    pub token_metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetTreeDelegate<'info> {
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        bump,
        has_one = tree_creator
    )]
    pub tree_authority: Account<'info, TreeConfig>,
    pub tree_creator: Signer<'info>,
    /// CHECK: this account is neither read from or written to
    pub new_tree_delegate: UncheckedAccount<'info>,
    /// CHECK: this account is neither read from or written to
    pub merkle_tree: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn hash_creators(creators: &[Creator]) -> Result<[u8; 32]> {
    // Convert creator Vec to bytes Vec.
    let creator_data = creators
        .iter()
        .map(|c| [c.address.as_ref(), &[c.verified as u8], &[c.share]].concat())
        .collect::<Vec<_>>();
    // Calculate new creator hash.
    Ok(keccak::hashv(
        creator_data
            .iter()
            .map(|c| c.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_ref(),
    )
    .to_bytes())
}

pub fn hash_metadata(metadata: &MetadataArgs) -> Result<[u8; 32]> {
    let metadata_args_hash = keccak::hashv(&[metadata.try_to_vec()?.as_slice()]);
    // Calculate new data hash.
    Ok(keccak::hashv(&[
        &metadata_args_hash.to_bytes(),
        &metadata.seller_fee_basis_points.to_le_bytes(),
    ])
    .to_bytes())
}

pub enum InstructionName {
    Unknown,
    MintV1,
    Redeem,
    CancelRedeem,
    Transfer,
    Delegate,
    DecompressV1,
    Compress,
    Burn,
    CreateTree,
    VerifyCreator,
    UnverifyCreator,
    VerifyCollection,
    UnverifyCollection,
    SetAndVerifyCollection,
    MintToCollectionV1,
}

pub fn get_instruction_type(full_bytes: &[u8]) -> InstructionName {
    let disc: [u8; 8] = {
        let mut disc = [0; 8];
        disc.copy_from_slice(&full_bytes[..8]);
        disc
    };
    match disc {
        [145, 98, 192, 118, 184, 147, 118, 104] => InstructionName::MintV1,
        [153, 18, 178, 47, 197, 158, 86, 15] => InstructionName::MintToCollectionV1,
        [111, 76, 232, 50, 39, 175, 48, 242] => InstructionName::CancelRedeem,
        [184, 12, 86, 149, 70, 196, 97, 225] => InstructionName::Redeem,
        [163, 52, 200, 231, 140, 3, 69, 186] => InstructionName::Transfer,
        [90, 147, 75, 178, 85, 88, 4, 137] => InstructionName::Delegate,
        [54, 85, 76, 70, 228, 250, 164, 81] => InstructionName::DecompressV1,
        [116, 110, 29, 56, 107, 219, 42, 93] => InstructionName::Burn,
        [82, 193, 176, 117, 176, 21, 115, 253] => InstructionName::Compress,
        [165, 83, 136, 142, 89, 202, 47, 220] => InstructionName::CreateTree,
        [52, 17, 96, 132, 71, 4, 85, 194] => InstructionName::VerifyCreator,
        [107, 178, 57, 39, 105, 115, 112, 152] => InstructionName::UnverifyCreator,
        [56, 113, 101, 253, 79, 55, 122, 169] => InstructionName::VerifyCollection,
        [250, 251, 42, 106, 41, 137, 186, 168] => InstructionName::UnverifyCollection,
        [235, 242, 121, 216, 158, 234, 180, 234] => InstructionName::SetAndVerifyCollection,

        _ => InstructionName::Unknown,
    }
}

fn process_mint_v1<'info>(
    message: MetadataArgs,
    owner: Pubkey,
    delegate: Pubkey,
    metadata_auth: HashSet<Pubkey>,
    authority_bump: u8,
    authority: &mut Account<'info, TreeConfig>,
    merkle_tree: &AccountInfo<'info>,
    wrapper: &Program<'info, Noop>,
    compression_program: &AccountInfo<'info>,
    allow_verified_collection: bool,
) -> Result<()> {
    assert_metadata_is_mpl_compatible(&message)?;
    if !allow_verified_collection {
        if let Some(collection) = &message.collection {
            if collection.verified {
                return Err(BubblegumError::CollectionCannotBeVerifiedInThisInstruction.into());
            }
        }
    }

    // @dev: seller_fee_basis points is encoded twice so that it can be passed to marketplace
    // instructions, without passing the entire, un-hashed MetadataArgs struct
    let metadata_args_hash = keccak::hashv(&[message.try_to_vec()?.as_slice()]);
    let data_hash = keccak::hashv(&[
        &metadata_args_hash.to_bytes(),
        &message.seller_fee_basis_points.to_le_bytes(),
    ]);

    // Use the metadata auth to check whether we can allow `verified` to be set to true in the
    // creator Vec.
    let creator_data = message
        .creators
        .iter()
        .map(|c| {
            if c.verified && !metadata_auth.contains(&c.address) {
                Err(BubblegumError::CreatorDidNotVerify.into())
            } else {
                Ok([c.address.as_ref(), &[c.verified as u8], &[c.share]].concat())
            }
        })
        .collect::<Result<Vec<_>>>()?;

    // Calculate creator hash.
    let creator_hash = keccak::hashv(
        creator_data
            .iter()
            .map(|c| c.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_ref(),
    );

    let asset_id = get_asset_id(&merkle_tree.key(), authority.num_minted);
    let leaf = LeafSchema::new_v0(
        asset_id,
        owner,
        delegate,
        authority.num_minted,
        data_hash.to_bytes(),
        creator_hash.to_bytes(),
    );

    wrap_application_data_v1(leaf.to_event().try_to_vec()?, wrapper)?;

    append_leaf(
        &merkle_tree.key(),
        authority_bump,
        &compression_program.to_account_info(),
        &authority.to_account_info(),
        &merkle_tree.to_account_info(),
        &wrapper.to_account_info(),
        leaf.to_node(),
    )
}

fn process_creator_verification<'info>(
    ctx: Context<'_, '_, '_, 'info, CreatorVerification<'info>>,
    root: [u8; 32],
    data_hash: [u8; 32],
    creator_hash: [u8; 32],
    nonce: u64,
    index: u32,
    mut message: MetadataArgs,
    verify: bool,
) -> Result<()> {
    let owner = ctx.accounts.leaf_owner.to_account_info();
    let delegate = ctx.accounts.leaf_delegate.to_account_info();
    let merkle_tree = ctx.accounts.merkle_tree.to_account_info();

    let creator = ctx.accounts.creator.key();

    // Creator Vec must contain creators.
    if message.creators.is_empty() {
        return Err(BubblegumError::NoCreatorsPresent.into());
    }

    // Creator must be in user-provided creator Vec.
    if !message.creators.iter().any(|c| c.address == creator) {
        return Err(BubblegumError::CreatorNotFound.into());
    }

    // User-provided creator Vec must result in same user-provided creator hash.
    let incoming_creator_hash = hash_creators(&message.creators)?;
    if creator_hash != incoming_creator_hash {
        return Err(BubblegumError::CreatorHashMismatch.into());
    }

    // User-provided metadata must result in same user-provided data hash.
    let incoming_data_hash = hash_metadata(&message)?;
    if data_hash != incoming_data_hash {
        return Err(BubblegumError::DataHashMismatch.into());
    }

    // Calculate new creator Vec with `verified` set to true for signing creator.
    let updated_creator_vec = message
        .creators
        .iter()
        .map(|c| {
            let verified = if c.address == creator.key() {
                verify
            } else {
                c.verified
            };
            Creator {
                address: c.address,
                verified,
                share: c.share,
            }
        })
        .collect::<Vec<Creator>>();

    // Calculate new creator hash.
    let updated_creator_hash = hash_creators(&updated_creator_vec)?;

    // Update creator Vec in metadata args.
    message.creators = updated_creator_vec;

    // Calculate new data hash.
    let updated_data_hash = hash_metadata(&message)?;

    // Build previous leaf struct, new leaf struct, and replace the leaf in the tree.
    let asset_id = get_asset_id(&merkle_tree.key(), nonce);
    let previous_leaf = LeafSchema::new_v0(
        asset_id,
        owner.key(),
        delegate.key(),
        nonce,
        data_hash,
        creator_hash,
    );
    let new_leaf = LeafSchema::new_v0(
        asset_id,
        owner.key(),
        delegate.key(),
        nonce,
        updated_data_hash,
        updated_creator_hash,
    );

    wrap_application_data_v1(new_leaf.to_event().try_to_vec()?, &ctx.accounts.log_wrapper)?;

    replace_leaf(
        &merkle_tree.key(),
        *ctx.bumps.get("tree_authority").unwrap(),
        &ctx.accounts.compression_program.to_account_info(),
        &ctx.accounts.tree_authority.to_account_info(),
        &ctx.accounts.merkle_tree.to_account_info(),
        &ctx.accounts.log_wrapper.to_account_info(),
        ctx.remaining_accounts,
        root,
        previous_leaf.to_node(),
        new_leaf.to_node(),
        index,
    )
}

fn process_collection_verification_mpl_only<'info>(
    collection_metadata: &Box<Account<'info, TokenMetadata>>,
    collection_mint: &AccountInfo<'info>,
    collection_authority: &AccountInfo<'info>,
    collection_authority_record_pda: &AccountInfo<'info>,
    edition_account: &AccountInfo<'info>,
    bubblegum_signer: &AccountInfo<'info>,
    bubblegum_bump: u8,
    token_metadata_program: &AccountInfo<'info>,
    message: &mut MetadataArgs,
    verify: bool,
    new_collection: Option<Pubkey>,
) -> Result<()> {
    // See if a collection authority record PDA was provided.
    let collection_authority_record = if collection_authority_record_pda.key() == crate::id() {
        None
    } else {
        Some(collection_authority_record_pda)
    };

    // Verify correct account ownerships.
    require!(
        *collection_metadata.to_account_info().owner == token_metadata_program.key(),
        BubblegumError::IncorrectOwner
    );
    require!(
        *collection_mint.owner == spl_token::id(),
        BubblegumError::IncorrectOwner
    );
    require!(
        *edition_account.owner == token_metadata_program.key(),
        BubblegumError::IncorrectOwner
    );

    // If new collection was provided, set it in the NFT metadata.
    if new_collection.is_some() {
        message.collection = new_collection.map(|key| metaplex_adapter::Collection {
            verified: false, // Set to true below.
            key,
        });
    }

    // If the NFT has collection data, we set it to the correct value after doing some validation.
    if let Some(collection) = &mut message.collection {
        // Don't verify already verified items, or unverify unverified items, otherwise for sized
        // collections we end up with invalid size data.
        if verify && collection.verified {
            return Err(BubblegumError::AlreadyVerified.into());
        } else if !verify && !collection.verified {
            return Err(BubblegumError::AlreadyUnverified.into());
        }

        // Collection verify assert from token-metadata program.
        assert_collection_verify_is_valid(
            &Some(collection.adapt()),
            collection_metadata,
            collection_mint,
            edition_account,
        )?;

        // Collection authority assert from token-metadata.
        assert_has_collection_authority(
            collection_authority,
            collection_metadata,
            collection_mint.key,
            collection_authority_record,
        )?;

        // Update collection in metadata args.  Note since this is a mutable reference,
        // it is still updating `message.collection` after being destructured.
        collection.verified = verify;
    } else {
        return Err(BubblegumError::CollectionNotFound.into());
    }

    // If this is a sized collection, then increment or decrement collection size.
    if let Some(details) = &collection_metadata.collection_details {
        // Increment or decrement existing size.
        let new_size = match details {
            CollectionDetails::V1 { size } => {
                if verify {
                    size.checked_add(1)
                        .ok_or(BubblegumError::NumericalOverflowError)?
                } else {
                    size.checked_sub(1)
                        .ok_or(BubblegumError::NumericalOverflowError)?
                }
            }
        };

        // CPI into to token-metadata program to change the collection size.
        let mut bubblegum_set_collection_size_infos = vec![
            collection_metadata.to_account_info(),
            collection_authority.clone(),
            collection_mint.clone(),
            bubblegum_signer.clone(),
        ];

        if let Some(record) = collection_authority_record {
            bubblegum_set_collection_size_infos.push(record.clone());
        }

        invoke_signed(
            &mpl_token_metadata::instruction::bubblegum_set_collection_size(
                token_metadata_program.key(),
                collection_metadata.to_account_info().key(),
                collection_authority.key(),
                collection_mint.key(),
                bubblegum_signer.key(),
                collection_authority_record.map(|r| r.key()),
                new_size,
            ),
            bubblegum_set_collection_size_infos.as_slice(),
            &[&[COLLECTION_CPI_PREFIX.as_bytes(), &[bubblegum_bump]]],
        )?;
    } else {
        return Err(BubblegumError::CollectionMustBeSized.into());
    }

    Ok(())
}

fn process_collection_verification<'info>(
    ctx: Context<'_, '_, '_, 'info, CollectionVerification<'info>>,
    root: [u8; 32],
    data_hash: [u8; 32],
    creator_hash: [u8; 32],
    nonce: u64,
    index: u32,
    mut message: MetadataArgs,
    verify: bool,
    new_collection: Option<Pubkey>,
) -> Result<()> {
    let owner = ctx.accounts.leaf_owner.to_account_info();
    let delegate = ctx.accounts.leaf_delegate.to_account_info();
    let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
    let collection_metadata = &ctx.accounts.collection_metadata;
    let collection_mint = ctx.accounts.collection_mint.to_account_info();
    let edition_account = ctx.accounts.edition_account.to_account_info();
    let collection_authority = ctx.accounts.collection_authority.to_account_info();
    let collection_authority_record_pda = ctx
        .accounts
        .collection_authority_record_pda
        .to_account_info();
    let bubblegum_signer = ctx.accounts.bubblegum_signer.to_account_info();
    let token_metadata_program = ctx.accounts.token_metadata_program.to_account_info();

    process_collection_verification_mpl_only(
        collection_metadata,
        &collection_mint,
        &collection_authority,
        &collection_authority_record_pda,
        &edition_account,
        &bubblegum_signer,
        ctx.bumps["bubblegum_signer"],
        &token_metadata_program,
        &mut message,
        verify,
        new_collection,
    )?;

    // User-provided metadata must result in same user-provided data hash.
    let incoming_data_hash = hash_metadata(&message)?;
    if data_hash != incoming_data_hash {
        return Err(BubblegumError::DataHashMismatch.into());
    }

    // Calculate new data hash.
    let updated_data_hash = hash_metadata(&message)?;

    // Build previous leaf struct, new leaf struct, and replace the leaf in the tree.
    let asset_id = get_asset_id(&merkle_tree.key(), nonce);
    let previous_leaf = LeafSchema::new_v0(
        asset_id,
        owner.key(),
        delegate.key(),
        nonce,
        data_hash,
        creator_hash,
    );
    let new_leaf = LeafSchema::new_v0(
        asset_id,
        owner.key(),
        delegate.key(),
        nonce,
        updated_data_hash,
        creator_hash,
    );

    wrap_application_data_v1(new_leaf.to_event().try_to_vec()?, &ctx.accounts.log_wrapper)?;

    replace_leaf(
        &merkle_tree.key(),
        *ctx.bumps.get("tree_authority").unwrap(),
        &ctx.accounts.compression_program.to_account_info(),
        &ctx.accounts.tree_authority.to_account_info(),
        &ctx.accounts.merkle_tree.to_account_info(),
        &ctx.accounts.log_wrapper.to_account_info(),
        ctx.remaining_accounts,
        root,
        previous_leaf.to_node(),
        new_leaf.to_node(),
        index,
    )
}

#[program]
pub mod bubblegum {
    use super::*;

    pub fn create_tree(
        ctx: Context<CreateTree>,
        max_depth: u32,
        max_buffer_size: u32,
        public: Option<bool>,
    ) -> Result<()> {
        let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
        let seed = merkle_tree.key();
        let seeds = &[seed.as_ref(), &[*ctx.bumps.get("tree_authority").unwrap()]];
        let authority = &mut ctx.accounts.tree_authority;
        authority.set_inner(TreeConfig {
            tree_creator: ctx.accounts.tree_creator.key(),
            tree_delegate: ctx.accounts.tree_creator.key(),
            total_mint_capacity: 1 << max_depth,
            num_minted: 0,
            is_public: public.unwrap_or(false),
        });
        let authority_pda_signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.compression_program.to_account_info(),
            spl_account_compression::cpi::accounts::Initialize {
                authority: ctx.accounts.tree_authority.to_account_info(),
                merkle_tree,
                noop: ctx.accounts.log_wrapper.to_account_info(),
            },
            authority_pda_signer,
        );
        spl_account_compression::cpi::init_empty_merkle_tree(cpi_ctx, max_depth, max_buffer_size)
    }

    pub fn set_tree_delegate(ctx: Context<SetTreeDelegate>) -> Result<()> {
        ctx.accounts.tree_authority.tree_delegate = ctx.accounts.new_tree_delegate.key();
        Ok(())
    }

    pub fn mint_v1(ctx: Context<MintV1>, message: MetadataArgs) -> Result<()> {
        // TODO -> Separate V1 / V1 into seperate instructions
        let payer = ctx.accounts.payer.key();
        let incoming_tree_delegate = ctx.accounts.tree_delegate.key();
        let owner = ctx.accounts.leaf_owner.key();
        let delegate = ctx.accounts.leaf_delegate.key();
        let authority = &mut ctx.accounts.tree_authority;
        let tree_creator = authority.tree_creator;
        let tree_delegate = authority.tree_delegate;
        let merkle_tree = &ctx.accounts.merkle_tree;
        if !authority.is_public {
            require!(
                incoming_tree_delegate == tree_creator || incoming_tree_delegate == tree_delegate,
                BubblegumError::TreeAuthorityIncorrect,
            );
        }

        if !authority.contains_mint_capacity(1) {
            return Err(BubblegumError::InsufficientMintCapacity.into());
        }

        // Create a HashSet to store signers to use with creator validation.  Any signer can be
        // counted as a validated creator.
        let mut metadata_auth = HashSet::<Pubkey>::new();
        metadata_auth.insert(payer);
        metadata_auth.insert(tree_delegate);

        // If there are any remaining accounts that are also signers, they can also be used for
        // creator validation.
        metadata_auth.extend(
            ctx.remaining_accounts
                .iter()
                .filter(|a| a.is_signer)
                .map(|a| a.key()),
        );

        process_mint_v1(
            message,
            owner,
            delegate,
            metadata_auth,
            *ctx.bumps.get("tree_authority").unwrap(),
            authority,
            merkle_tree,
            &ctx.accounts.log_wrapper,
            &ctx.accounts.compression_program,
            false,
        )?;

        authority.increment_mint_count();

        Ok(())
    }

    pub fn mint_to_collection_v1(
        ctx: Context<MintToCollectionV1>,
        metadata_args: MetadataArgs,
    ) -> Result<()> {
        let mut message = metadata_args;
        // TODO -> Separate V1 / V1 into seperate instructions
        let payer = ctx.accounts.payer.key();
        let incoming_tree_delegate = ctx.accounts.tree_delegate.key();
        let owner = ctx.accounts.leaf_owner.key();
        let delegate = ctx.accounts.leaf_delegate.key();
        let authority = &mut ctx.accounts.tree_authority;
        let tree_creator = authority.tree_creator;
        let tree_delegate = authority.tree_delegate;
        let merkle_tree = &ctx.accounts.merkle_tree;

        let collection_metadata = &ctx.accounts.collection_metadata;
        let collection_mint = ctx.accounts.collection_mint.to_account_info();
        let edition_account = ctx.accounts.edition_account.to_account_info();
        let collection_authority = ctx.accounts.collection_authority.to_account_info();
        let collection_authority_record_pda = ctx
            .accounts
            .collection_authority_record_pda
            .to_account_info();
        let bubblegum_signer = ctx.accounts.bubblegum_signer.to_account_info();
        let token_metadata_program = ctx.accounts.token_metadata_program.to_account_info();

        if !authority.is_public {
            require!(
                incoming_tree_delegate == tree_creator || incoming_tree_delegate == tree_delegate,
                BubblegumError::TreeAuthorityIncorrect,
            );
        }

        if !authority.contains_mint_capacity(1) {
            return Err(BubblegumError::InsufficientMintCapacity.into());
        }

        // Create a HashSet to store signers to use with creator validation.  Any signer can be
        // counted as a validated creator.
        let mut metadata_auth = HashSet::<Pubkey>::new();
        metadata_auth.insert(payer);
        metadata_auth.insert(tree_delegate);

        // If there are any remaining accounts that are also signers, they can also be used for
        // creator validation.
        metadata_auth.extend(
            ctx.remaining_accounts
                .iter()
                .filter(|a| a.is_signer)
                .map(|a| a.key()),
        );

        process_collection_verification_mpl_only(
            collection_metadata,
            &collection_mint,
            &collection_authority,
            &collection_authority_record_pda,
            &edition_account,
            &bubblegum_signer,
            ctx.bumps["bubblegum_signer"],
            &token_metadata_program,
            &mut message,
            true,
            None,
        )?;

        process_mint_v1(
            message,
            owner,
            delegate,
            metadata_auth,
            *ctx.bumps.get("tree_authority").unwrap(),
            authority,
            merkle_tree,
            &ctx.accounts.log_wrapper,
            &ctx.accounts.compression_program,
            true,
        )?;

        authority.increment_mint_count();

        Ok(())
    }

    pub fn verify_creator<'info>(
        ctx: Context<'_, '_, '_, 'info, CreatorVerification<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
        message: MetadataArgs,
    ) -> Result<()> {
        process_creator_verification(
            ctx,
            root,
            data_hash,
            creator_hash,
            nonce,
            index,
            message,
            true,
        )
    }

    pub fn unverify_creator<'info>(
        ctx: Context<'_, '_, '_, 'info, CreatorVerification<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
        message: MetadataArgs,
    ) -> Result<()> {
        process_creator_verification(
            ctx,
            root,
            data_hash,
            creator_hash,
            nonce,
            index,
            message,
            false,
        )
    }

    pub fn verify_collection<'info>(
        ctx: Context<'_, '_, '_, 'info, CollectionVerification<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
        message: MetadataArgs,
    ) -> Result<()> {
        process_collection_verification(
            ctx,
            root,
            data_hash,
            creator_hash,
            nonce,
            index,
            message,
            true,
            None,
        )
    }

    pub fn unverify_collection<'info>(
        ctx: Context<'_, '_, '_, 'info, CollectionVerification<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
        message: MetadataArgs,
    ) -> Result<()> {
        process_collection_verification(
            ctx,
            root,
            data_hash,
            creator_hash,
            nonce,
            index,
            message,
            false,
            None,
        )
    }

    pub fn set_and_verify_collection<'info>(
        ctx: Context<'_, '_, '_, 'info, CollectionVerification<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
        message: MetadataArgs,
        collection: Pubkey,
    ) -> Result<()> {
        let incoming_tree_delegate = &ctx.accounts.tree_delegate;
        let tree_creator = ctx.accounts.tree_authority.tree_creator;
        let tree_delegate = ctx.accounts.tree_authority.tree_delegate;
        let collection_metadata = &ctx.accounts.collection_metadata;

        // Require that either the tree authority signed this transaction, or the tree authority is
        // the collection update authority which means the leaf update is approved via proxy, when
        // we later call `assert_has_collection_authority()`.
        //
        // This is similar to logic in token-metadata for `set_and_verify_collection()` except
        // this logic also allows the tree authority (which we are treating as the leaf metadata
        // authority) to be different than the collection authority (actual or delegated).  The
        // token-metadata program required them to be the same.
        let tree_authority_signed = incoming_tree_delegate.is_signer
            && (incoming_tree_delegate.key() == tree_creator
                || incoming_tree_delegate.key() == tree_delegate);

        let tree_authority_is_collection_update_authority = collection_metadata.update_authority
            == tree_creator
            || collection_metadata.update_authority == tree_delegate;

        require!(
            tree_authority_signed || tree_authority_is_collection_update_authority,
            BubblegumError::UpdateAuthorityIncorrect
        );

        process_collection_verification(
            ctx,
            root,
            data_hash,
            creator_hash,
            nonce,
            index,
            message,
            true,
            Some(collection),
        )
    }

    pub fn transfer<'info>(
        ctx: Context<'_, '_, '_, 'info, Transfer<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        // TODO add back version to select hash schema
        let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
        let owner = ctx.accounts.leaf_owner.to_account_info();
        let delegate = ctx.accounts.leaf_delegate.to_account_info();

        // Transfers must be initiated by either the leaf owner or leaf delegate.
        require!(
            owner.is_signer || delegate.is_signer,
            BubblegumError::LeafAuthorityMustSign
        );
        let new_owner = ctx.accounts.new_leaf_owner.key();
        let asset_id = get_asset_id(&merkle_tree.key(), nonce);
        let previous_leaf = LeafSchema::new_v0(
            asset_id,
            owner.key(),
            delegate.key(),
            nonce,
            data_hash,
            creator_hash,
        );
        // New leafs are instantiated with no delegate
        let new_leaf = LeafSchema::new_v0(
            asset_id,
            new_owner,
            new_owner,
            nonce,
            data_hash,
            creator_hash,
        );

        wrap_application_data_v1(new_leaf.to_event().try_to_vec()?, &ctx.accounts.log_wrapper)?;

        replace_leaf(
            &merkle_tree.key(),
            *ctx.bumps.get("tree_authority").unwrap(),
            &ctx.accounts.compression_program.to_account_info(),
            &ctx.accounts.tree_authority.to_account_info(),
            &ctx.accounts.merkle_tree.to_account_info(),
            &ctx.accounts.log_wrapper.to_account_info(),
            ctx.remaining_accounts,
            root,
            previous_leaf.to_node(),
            new_leaf.to_node(),
            index,
        )
    }

    pub fn delegate<'info>(
        ctx: Context<'_, '_, '_, 'info, Delegate<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
        let owner = ctx.accounts.leaf_owner.key();
        let previous_delegate = ctx.accounts.previous_leaf_delegate.key();
        let new_delegate = ctx.accounts.new_leaf_delegate.key();
        let asset_id = get_asset_id(&merkle_tree.key(), nonce);
        let previous_leaf = LeafSchema::new_v0(
            asset_id,
            owner,
            previous_delegate,
            nonce,
            data_hash,
            creator_hash,
        );
        let new_leaf = LeafSchema::new_v0(
            asset_id,
            owner,
            new_delegate,
            nonce,
            data_hash,
            creator_hash,
        );

        wrap_application_data_v1(new_leaf.to_event().try_to_vec()?, &ctx.accounts.log_wrapper)?;

        replace_leaf(
            &merkle_tree.key(),
            *ctx.bumps.get("tree_authority").unwrap(),
            &ctx.accounts.compression_program.to_account_info(),
            &ctx.accounts.tree_authority.to_account_info(),
            &ctx.accounts.merkle_tree.to_account_info(),
            &ctx.accounts.log_wrapper.to_account_info(),
            ctx.remaining_accounts,
            root,
            previous_leaf.to_node(),
            new_leaf.to_node(),
            index,
        )
    }

    pub fn burn<'info>(
        ctx: Context<'_, '_, '_, 'info, Burn<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        let owner = ctx.accounts.leaf_owner.to_account_info();
        let delegate = ctx.accounts.leaf_delegate.to_account_info();

        // Burn must be initiated by either the leaf owner or leaf delegate.
        require!(
            owner.is_signer || delegate.is_signer,
            BubblegumError::LeafAuthorityMustSign
        );
        let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
        let asset_id = get_asset_id(&merkle_tree.key(), nonce);

        let previous_leaf = LeafSchema::new_v0(
            asset_id,
            owner.key(),
            delegate.key(),
            nonce,
            data_hash,
            creator_hash,
        );

        let new_leaf = Node::default();

        replace_leaf(
            &merkle_tree.key(),
            *ctx.bumps.get("tree_authority").unwrap(),
            &ctx.accounts.compression_program.to_account_info(),
            &ctx.accounts.tree_authority.to_account_info(),
            &ctx.accounts.merkle_tree.to_account_info(),
            &ctx.accounts.log_wrapper.to_account_info(),
            ctx.remaining_accounts,
            root,
            previous_leaf.to_node(),
            new_leaf,
            index,
        )
    }

    pub fn redeem<'info>(
        ctx: Context<'_, '_, '_, 'info, Redeem<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        let owner = ctx.accounts.leaf_owner.key();
        let delegate = ctx.accounts.leaf_delegate.key();
        let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
        let asset_id = get_asset_id(&merkle_tree.key(), nonce);
        let previous_leaf =
            LeafSchema::new_v0(asset_id, owner, delegate, nonce, data_hash, creator_hash);

        let new_leaf = Node::default();

        replace_leaf(
            &merkle_tree.key(),
            *ctx.bumps.get("tree_authority").unwrap(),
            &ctx.accounts.compression_program.to_account_info(),
            &ctx.accounts.tree_authority.to_account_info(),
            &ctx.accounts.merkle_tree.to_account_info(),
            &ctx.accounts.log_wrapper.to_account_info(),
            ctx.remaining_accounts,
            root,
            previous_leaf.to_node(),
            new_leaf,
            index,
        )?;
        ctx.accounts
            .voucher
            .set_inner(Voucher::new(previous_leaf, index, merkle_tree.key()));

        Ok(())
    }

    pub fn cancel_redeem<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelRedeem<'info>>,
        root: [u8; 32],
    ) -> Result<()> {
        let voucher = &ctx.accounts.voucher;
        match ctx.accounts.voucher.leaf_schema {
            LeafSchema::V1 { owner, .. } => assert_pubkey_equal(
                &ctx.accounts.leaf_owner.key(),
                &owner,
                Some(BubblegumError::AssetOwnerMismatch.into()),
            ),
        }?;
        let merkle_tree = ctx.accounts.merkle_tree.to_account_info();

        wrap_application_data_v1(
            voucher.leaf_schema.to_event().try_to_vec()?,
            &ctx.accounts.log_wrapper,
        )?;

        replace_leaf(
            &merkle_tree.key(),
            *ctx.bumps.get("tree_authority").unwrap(),
            &ctx.accounts.compression_program.to_account_info(),
            &ctx.accounts.tree_authority.to_account_info(),
            &ctx.accounts.merkle_tree.to_account_info(),
            &ctx.accounts.log_wrapper.to_account_info(),
            ctx.remaining_accounts,
            root,
            [0; 32],
            voucher.leaf_schema.to_node(),
            voucher.index,
        )
    }

    pub fn decompress_v1(ctx: Context<DecompressV1>, metadata: MetadataArgs) -> Result<()> {
        // Allocate and create mint
        let incoming_data_hash = hash_metadata(&metadata)?;
        match ctx.accounts.voucher.leaf_schema {
            LeafSchema::V1 {
                owner, data_hash, ..
            } => {
                if !cmp_bytes(&data_hash, &incoming_data_hash, 32) {
                    return Err(BubblegumError::HashingMismatch.into());
                }
                if !cmp_pubkeys(&owner, ctx.accounts.leaf_owner.key) {
                    return Err(BubblegumError::AssetOwnerMismatch.into());
                }
            }
        }

        let voucher = &ctx.accounts.voucher;
        match metadata.token_program_version {
            TokenProgramVersion::Original => {
                if ctx.accounts.mint.data_is_empty() {
                    invoke_signed(
                        &system_instruction::create_account(
                            &ctx.accounts.leaf_owner.key(),
                            &ctx.accounts.mint.key(),
                            Rent::get()?.minimum_balance(SplMint::LEN),
                            SplMint::LEN as u64,
                            &spl_token::id(),
                        ),
                        &[
                            ctx.accounts.leaf_owner.to_account_info(),
                            ctx.accounts.mint.to_account_info(),
                            ctx.accounts.system_program.to_account_info(),
                        ],
                        &[&[
                            ASSET_PREFIX.as_bytes(),
                            voucher.merkle_tree.key().as_ref(),
                            voucher.leaf_schema.nonce().to_le_bytes().as_ref(),
                            &[*ctx.bumps.get("mint").unwrap()],
                        ]],
                    )?;
                    invoke(
                        &spl_token::instruction::initialize_mint2(
                            &spl_token::id(),
                            &ctx.accounts.mint.key(),
                            &ctx.accounts.mint_authority.key(),
                            Some(&ctx.accounts.mint_authority.key()),
                            0,
                        )?,
                        &[
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.mint.to_account_info(),
                        ],
                    )?;
                }
                if ctx.accounts.token_account.data_is_empty() {
                    invoke(
                        &spl_associated_token_account::instruction::create_associated_token_account(
                            &ctx.accounts.leaf_owner.key(),
                            &ctx.accounts.leaf_owner.key(),
                            &ctx.accounts.mint.key(),
                            &spl_token::ID,
                        ),
                        &[
                            ctx.accounts.leaf_owner.to_account_info(),
                            ctx.accounts.mint.to_account_info(),
                            ctx.accounts.token_account.to_account_info(),
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.associated_token_program.to_account_info(),
                            ctx.accounts.system_program.to_account_info(),
                            ctx.accounts.sysvar_rent.to_account_info(),
                        ],
                    )?;
                }
                invoke_signed(
                    &spl_token::instruction::mint_to(
                        &spl_token::id(),
                        &ctx.accounts.mint.key(),
                        &ctx.accounts.token_account.key(),
                        &ctx.accounts.mint_authority.key(),
                        &[],
                        1,
                    )?,
                    &[
                        ctx.accounts.mint.to_account_info(),
                        ctx.accounts.token_account.to_account_info(),
                        ctx.accounts.mint_authority.to_account_info(),
                        ctx.accounts.token_program.to_account_info(),
                    ],
                    &[&[
                        ctx.accounts.mint.key().as_ref(),
                        &[ctx.bumps["mint_authority"]],
                    ]],
                )?;
            }
            TokenProgramVersion::Token2022 => return Err(ProgramError::InvalidArgument.into()),
        }

        invoke_signed(
            &system_instruction::assign(&ctx.accounts.mint_authority.key(), &crate::id()),
            &[ctx.accounts.mint_authority.to_account_info()],
            &[&[
                ctx.accounts.mint.key().as_ref(),
                &[*ctx.bumps.get("mint_authority").unwrap()],
            ]],
        )?;

        let metadata_infos = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.leaf_owner.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_rent.to_account_info(),
        ];

        let master_edition_infos = vec![
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.leaf_owner.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_rent.to_account_info(),
        ];

        msg!("Creating metadata!");
        invoke_signed(
            &mpl_token_metadata::instruction::create_metadata_accounts_v3(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.leaf_owner.key(),
                ctx.accounts.mint_authority.key(),
                metadata.name.clone(),
                metadata.symbol.clone(),
                metadata.uri.clone(),
                if !metadata.creators.is_empty() {
                    Some(metadata.creators.iter().map(|c| c.adapt()).collect())
                } else {
                    None
                },
                metadata.seller_fee_basis_points,
                true,
                metadata.is_mutable,
                metadata.collection.map(|c| c.adapt()),
                metadata.uses.map(|u| u.adapt()),
                None,
            ),
            metadata_infos.as_slice(),
            &[&[
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps["mint_authority"]],
            ]],
        )?;

        msg!("Creating master edition!");
        invoke_signed(
            &mpl_token_metadata::instruction::create_master_edition_v3(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.master_edition.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.leaf_owner.key(),
                Some(0),
            ),
            master_edition_infos.as_slice(),
            &[&[
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps["mint_authority"]],
            ]],
        )?;

        ctx.accounts
            .mint_authority
            .to_account_info()
            .assign(&System::id());
        Ok(())
    }

    pub fn compress(_ctx: Context<Compress>) -> Result<()> {
        // TODO
        Ok(())
    }
}
