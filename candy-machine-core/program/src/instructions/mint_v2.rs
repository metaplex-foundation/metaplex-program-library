use anchor_lang::prelude::*;
use arrayref::array_ref;
use mpl_token_metadata::{
    instruction::{
        builders::{CreateBuilder, MintBuilder, UpdateBuilder, VerifyBuilder},
        create_master_edition_v3, create_metadata_accounts_v3, set_and_verify_collection,
        set_and_verify_sized_collection_item, update_metadata_accounts_v2, CreateArgs,
        InstructionBuilder, MintArgs, RuleSetToggle, UpdateArgs, VerificationArgs,
    },
    state::{
        AssetData, Collection, Metadata, PrintSupply, ProgrammableConfig, TokenMetadataAccount,
        TokenStandard,
    },
};
use solana_program::{program::invoke_signed, sysvar};

use crate::{
    constants::{AUTHORITY_SEED, EMPTY_STR, HIDDEN_SECTION, NULL_STRING},
    utils::*,
    AccountVersion, CandyError, CandyMachine, ConfigLine,
};

/// Accounts to mint an NFT.
pub(crate) struct MintAccounts<'info> {
    pub(crate) authority_pda: AccountInfo<'info>,
    pub(crate) payer: AccountInfo<'info>,
    pub(crate) nft_mint: AccountInfo<'info>,
    pub(crate) nft_mint_authority: AccountInfo<'info>,
    pub(crate) nft_metadata: AccountInfo<'info>,
    pub(crate) nft_master_edition: AccountInfo<'info>,
    pub(crate) token: Option<AccountInfo<'info>>,
    pub(crate) token_record: Option<AccountInfo<'info>>,
    pub(crate) collection_delegate_record: AccountInfo<'info>,
    pub(crate) collection_mint: AccountInfo<'info>,
    pub(crate) collection_metadata: AccountInfo<'info>,
    pub(crate) collection_master_edition: AccountInfo<'info>,
    pub(crate) collection_update_authority: AccountInfo<'info>,
    pub(crate) token_metadata_program: AccountInfo<'info>,
    pub(crate) spl_token_program: AccountInfo<'info>,
    pub(crate) spl_ata_program: Option<AccountInfo<'info>>,
    pub(crate) system_program: AccountInfo<'info>,
    pub(crate) sysvar_instructions: Option<AccountInfo<'info>>,
    pub(crate) recent_slothashes: AccountInfo<'info>,
}

pub fn mint_v2<'info>(ctx: Context<'_, '_, '_, 'info, MintV2<'info>>) -> Result<()> {
    let accounts = MintAccounts {
        spl_ata_program: ctx
            .accounts
            .spl_ata_program
            .as_ref()
            .map(|spl_ata_program| spl_ata_program.to_account_info()),
        authority_pda: ctx.accounts.authority_pda.to_account_info(),
        collection_delegate_record: ctx.accounts.collection_delegate_record.to_account_info(),
        collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
        collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
        collection_mint: ctx.accounts.collection_mint.to_account_info(),
        collection_update_authority: ctx.accounts.collection_update_authority.to_account_info(),
        nft_master_edition: ctx.accounts.nft_master_edition.to_account_info(),
        nft_metadata: ctx.accounts.nft_metadata.to_account_info(),
        nft_mint: ctx.accounts.nft_mint.to_account_info(),
        nft_mint_authority: ctx.accounts.nft_mint_authority.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        recent_slothashes: ctx.accounts.recent_slothashes.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        sysvar_instructions: ctx
            .accounts
            .sysvar_instructions
            .as_ref()
            .map(|sysvar_instructions| sysvar_instructions.to_account_info()),
        token: ctx
            .accounts
            .token
            .as_ref()
            .map(|token| token.to_account_info()),
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        spl_token_program: ctx.accounts.spl_token_program.to_account_info(),
        token_record: ctx
            .accounts
            .token_record
            .as_ref()
            .map(|token_record| token_record.to_account_info()),
    };

    process_mint(
        &mut ctx.accounts.candy_machine,
        accounts,
        ctx.bumps["authority_pda"],
    )
}

/// Mint a new NFT.
///
/// The index minted depends on the configuration of the candy machine: it could be
/// a psuedo-randomly selected one or sequential. In both cases, after minted a
/// specific index, the candy machine does not allow to mint the same index again.
pub(crate) fn process_mint<'info>(
    candy_machine: &mut Box<Account<'info, CandyMachine>>,
    accounts: MintAccounts,
    bump: u8,
) -> Result<()> {
    if !accounts.nft_metadata.data_is_empty() {
        return err!(CandyError::MetadataAccountMustBeEmpty);
    }

    // are there items to be minted?
    if candy_machine.items_redeemed >= candy_machine.data.items_available {
        return err!(CandyError::CandyMachineEmpty);
    }

    // check that we got the correct collection mint
    if !cmp_pubkeys(
        &accounts.collection_mint.key(),
        &candy_machine.collection_mint,
    ) {
        return err!(CandyError::CollectionKeyMismatch);
    }

    // collection metadata must be owner by token metadata
    if !cmp_pubkeys(
        accounts.collection_metadata.owner,
        &mpl_token_metadata::id(),
    ) {
        return err!(CandyError::IncorrectOwner);
    }

    let collection_metadata_info = &accounts.collection_metadata;
    let collection_metadata: Metadata =
        Metadata::from_account_info(&collection_metadata_info.to_account_info())?;
    // check that the update authority matches the collection update authority
    if !cmp_pubkeys(
        &collection_metadata.update_authority,
        &accounts.collection_update_authority.key(),
    ) {
        return err!(CandyError::IncorrectCollectionAuthority);
    }

    // (2) selecting an item to mint

    let recent_slothashes = &accounts.recent_slothashes;
    let data = recent_slothashes.data.borrow();
    let most_recent = array_ref![data, 12, 8];

    let clock = Clock::get()?;
    // seed for the random number is a combination of the slot_hash - timestamp
    let seed = u64::from_le_bytes(*most_recent).saturating_sub(clock.unix_timestamp as u64);

    let remainder: usize = seed
        .checked_rem(candy_machine.data.items_available - candy_machine.items_redeemed)
        .ok_or(CandyError::NumericalOverflowError)? as usize;

    let config_line = get_config_line(candy_machine, remainder, candy_machine.items_redeemed)?;

    candy_machine.items_redeemed = candy_machine
        .items_redeemed
        .checked_add(1)
        .ok_or(CandyError::NumericalOverflowError)?;
    // release the data borrow
    drop(data);

    // (3) minting

    let mut creators: Vec<mpl_token_metadata::state::Creator> =
        vec![mpl_token_metadata::state::Creator {
            address: accounts.authority_pda.key(),
            verified: true,
            share: 0,
        }];

    for c in &candy_machine.data.creators {
        creators.push(mpl_token_metadata::state::Creator {
            address: c.address,
            verified: false,
            share: c.percentage_share,
        });
    }

    match candy_machine.version {
        AccountVersion::V1 => create(
            candy_machine,
            accounts,
            bump,
            config_line,
            creators,
            collection_metadata,
        ),
        AccountVersion::V2 => create_and_mint(
            candy_machine,
            accounts,
            bump,
            config_line,
            creators,
            collection_metadata,
        ),
    }
}

/// Selects and returns the information of a config line.
///
/// The selection could be either sequential or random.
pub fn get_config_line(
    candy_machine: &Account<'_, CandyMachine>,
    index: usize,
    mint_number: u64,
) -> Result<ConfigLine> {
    if let Some(hs) = &candy_machine.data.hidden_settings {
        return Ok(ConfigLine {
            name: replace_patterns(hs.name.clone(), mint_number as usize),
            uri: replace_patterns(hs.uri.clone(), mint_number as usize),
        });
    }
    let settings = if let Some(settings) = &candy_machine.data.config_line_settings {
        settings
    } else {
        return err!(CandyError::MissingConfigLinesSettings);
    };

    let account_info = candy_machine.to_account_info();
    let mut account_data = account_info.data.borrow_mut();

    // validates that all config lines were added to the candy machine
    let config_count = get_config_count(&account_data)? as u64;
    if config_count != candy_machine.data.items_available {
        return err!(CandyError::NotFullyLoaded);
    }

    // (1) determine the mint index (index is a random index on the available indices array)

    let value_to_use = if settings.is_sequential {
        mint_number as usize
    } else {
        let items_available = candy_machine.data.items_available;
        let indices_start = HIDDEN_SECTION
            + 4
            + (items_available as usize) * candy_machine.data.get_config_line_size()
            + (items_available
                .checked_div(8)
                .ok_or(CandyError::NumericalOverflowError)?
                + 1) as usize;
        // calculates the mint index and retrieves the value at that position
        let mint_index = indices_start + index * 4;
        let value_to_use = u32::from_le_bytes(*array_ref![account_data, mint_index, 4]) as usize;
        // calculates the last available index and retrieves the value at that position
        let last_index = indices_start + ((items_available - mint_number - 1) * 4) as usize;
        let last_value = u32::from_le_bytes(*array_ref![account_data, last_index, 4]);
        // swap-remove: this guarantees that we remove the used mint index from the available array
        // in a constant time O(1) no matter how big the indices array is
        account_data[mint_index..mint_index + 4].copy_from_slice(&u32::to_le_bytes(last_value));

        value_to_use
    };

    // (2) retrieve the config line at the mint_index position

    let mut position =
        HIDDEN_SECTION + 4 + value_to_use * candy_machine.data.get_config_line_size();
    let name_length = settings.name_length as usize;
    let uri_length = settings.uri_length as usize;

    let name = if name_length > 0 {
        let name_slice: &mut [u8] = &mut account_data[position..position + name_length];
        let name = String::from_utf8(name_slice.to_vec())
            .map_err(|_| CandyError::CouldNotRetrieveConfigLineData)?;
        name.trim_end_matches(NULL_STRING).to_string()
    } else {
        EMPTY_STR.to_string()
    };

    position += name_length;
    let uri = if uri_length > 0 {
        let uri_slice: &mut [u8] = &mut account_data[position..position + uri_length];
        let uri = String::from_utf8(uri_slice.to_vec())
            .map_err(|_| CandyError::CouldNotRetrieveConfigLineData)?;
        uri.trim_end_matches(NULL_STRING).to_string()
    } else {
        EMPTY_STR.to_string()
    };

    let complete_name = replace_patterns(settings.prefix_name.clone(), value_to_use) + &name;
    let complete_uri = replace_patterns(settings.prefix_uri.clone(), value_to_use) + &uri;

    Ok(ConfigLine {
        name: complete_name,
        uri: complete_uri,
    })
}

/// Creates the metadata accounts and mint a new token.
fn create_and_mint<'info>(
    candy_machine: &mut Box<Account<'info, CandyMachine>>,
    accounts: MintAccounts,
    bump: u8,
    config_line: ConfigLine,
    creators: Vec<mpl_token_metadata::state::Creator>,
    collection_metadata: Metadata,
) -> Result<()> {
    let mut asset_data = AssetData::new(
        if candy_machine.token_standard == TokenStandard::ProgrammableNonFungible as u8 {
            TokenStandard::ProgrammableNonFungible
        } else {
            TokenStandard::NonFungible
        },
        config_line.name,
        candy_machine.data.symbol.to_string(),
        config_line.uri,
    );
    asset_data.primary_sale_happened = true;
    asset_data.seller_fee_basis_points = candy_machine.data.seller_fee_basis_points;
    asset_data.is_mutable = candy_machine.data.is_mutable;
    asset_data.creators = Some(creators);
    asset_data.collection = Some(Collection {
        verified: false,
        key: candy_machine.collection_mint,
    });

    // create metadata accounts

    let sysvar_instructions_info = accounts
        .sysvar_instructions
        .as_ref()
        .ok_or(CandyError::MissingInstructionsSysvar)?;

    let create_ix = CreateBuilder::new()
        .metadata(accounts.nft_metadata.key())
        .mint(accounts.nft_mint.key())
        .authority(accounts.nft_mint_authority.key())
        .payer(accounts.payer.key())
        .update_authority(accounts.authority_pda.key())
        .master_edition(accounts.nft_master_edition.key())
        .initialize_mint(accounts.nft_mint.is_signer)
        .update_authority_as_signer(true)
        .build(CreateArgs::V1 {
            asset_data,
            decimals: Some(0),
            print_supply: if candy_machine.data.max_supply == 0 {
                Some(PrintSupply::Zero)
            } else {
                Some(PrintSupply::Limited(candy_machine.data.max_supply))
            },
        })
        .map_err(|_| CandyError::InstructionBuilderFailed)?
        .instruction();

    let create_infos = vec![
        accounts.nft_metadata.to_account_info(),
        accounts.nft_mint.to_account_info(),
        accounts.nft_mint_authority.to_account_info(),
        accounts.payer.to_account_info(),
        accounts.authority_pda.to_account_info(),
        accounts.nft_master_edition.to_account_info(),
        accounts.system_program.to_account_info(),
        sysvar_instructions_info.to_account_info(),
        accounts.spl_token_program.to_account_info(),
    ];

    let candy_machine_key = candy_machine.key();
    let authority_seeds = [
        AUTHORITY_SEED.as_bytes(),
        candy_machine_key.as_ref(),
        &[bump],
    ];

    invoke_signed(&create_ix, &create_infos, &[&authority_seeds])?;

    // mints one token

    let token_info = accounts
        .token
        .as_ref()
        .ok_or(CandyError::MissingTokenAccount)?;
    let token_record_info =
        if candy_machine.token_standard == TokenStandard::ProgrammableNonFungible as u8 {
            Some(
                accounts
                    .token_record
                    .as_ref()
                    .ok_or(CandyError::MissingTokenRecord)?,
            )
        } else {
            accounts.token_record.as_ref()
        };
    let spl_ata_program_info = accounts
        .spl_ata_program
        .as_ref()
        .ok_or(CandyError::MissingSplAtaProgram)?;

    let mut mint_builder = MintBuilder::new();
    mint_builder
        .token(token_info.key())
        .token_owner(accounts.nft_mint_authority.key())
        .metadata(accounts.nft_metadata.key())
        .master_edition(accounts.nft_master_edition.key())
        .mint(accounts.nft_mint.key())
        .payer(accounts.payer.key())
        .authority(accounts.authority_pda.key());

    let mut mint_infos = vec![
        token_info.to_account_info(),
        accounts.nft_mint_authority.to_account_info(),
        accounts.nft_metadata.to_account_info(),
        accounts.nft_master_edition.to_account_info(),
        accounts.nft_mint.to_account_info(),
        accounts.payer.to_account_info(),
        accounts.authority_pda.to_account_info(),
        accounts.system_program.to_account_info(),
        sysvar_instructions_info.to_account_info(),
        accounts.spl_token_program.to_account_info(),
        spl_ata_program_info.to_account_info(),
    ];

    if let Some(token_record_info) = token_record_info {
        mint_builder.token_record(token_record_info.key());
        mint_infos.push(token_record_info.to_account_info());
    }

    let mint_ix = mint_builder
        .build(MintArgs::V1 {
            amount: 1,
            authorization_data: None,
        })
        .map_err(|_| CandyError::InstructionBuilderFailed)?
        .instruction();

    invoke_signed(&mint_ix, &mint_infos, &[&authority_seeds])?;

    // changes the update authority and authorization rules

    let mut update_args = UpdateArgs::default();
    let UpdateArgs::V1 {
        new_update_authority,
        rule_set,
        ..
    } = &mut update_args;
    // set the update authority to the authority of the candy machine
    *new_update_authority = Some(candy_machine.authority);
    // set the rule set to be the same as the parent collection
    *rule_set = if let Some(ProgrammableConfig::V1 {
        rule_set: Some(rule_set),
    }) = collection_metadata.programmable_config
    {
        RuleSetToggle::Set(rule_set)
    } else {
        RuleSetToggle::None
    };

    let update_ix = UpdateBuilder::new()
        .authority(accounts.authority_pda.key())
        .token(token_info.key())
        .metadata(accounts.nft_metadata.key())
        .edition(accounts.nft_master_edition.key())
        .mint(accounts.nft_mint.key())
        .payer(accounts.payer.key())
        .build(update_args)
        .map_err(|_| CandyError::InstructionBuilderFailed)?
        .instruction();

    let update_infos = vec![
        accounts.authority_pda.to_account_info(),
        token_info.to_account_info(),
        accounts.nft_metadata.to_account_info(),
        accounts.nft_master_edition.to_account_info(),
        accounts.nft_mint.to_account_info(),
        accounts.payer.to_account_info(),
        accounts.system_program.to_account_info(),
        sysvar_instructions_info.to_account_info(),
    ];

    invoke_signed(&update_ix, &update_infos, &[&authority_seeds])?;

    // verify the minted nft into the collection

    let verify_ix = VerifyBuilder::new()
        .authority(accounts.authority_pda.key())
        .delegate_record(accounts.collection_delegate_record.key())
        .metadata(accounts.nft_metadata.key())
        .collection_mint(accounts.collection_mint.key())
        .collection_metadata(accounts.collection_metadata.key())
        .collection_master_edition(accounts.collection_master_edition.key())
        .build(VerificationArgs::CollectionV1)
        .map_err(|_| CandyError::InstructionBuilderFailed)?
        .instruction();

    let verify_infos = vec![
        accounts.authority_pda.to_account_info(),
        accounts.collection_delegate_record.to_account_info(),
        accounts.nft_metadata.to_account_info(),
        accounts.collection_mint.to_account_info(),
        accounts.collection_metadata.to_account_info(),
        accounts.collection_master_edition.to_account_info(),
        accounts.system_program.to_account_info(),
        sysvar_instructions_info.to_account_info(),
    ];

    invoke_signed(&verify_ix, &verify_infos, &[&authority_seeds]).map_err(|error| error.into())
}

/// Creates the metadata accounts
fn create<'info>(
    candy_machine: &mut Box<Account<'info, CandyMachine>>,
    accounts: MintAccounts,
    bump: u8,
    config_line: ConfigLine,
    creators: Vec<mpl_token_metadata::state::Creator>,
    collection_metadata: Metadata,
) -> Result<()> {
    let metadata_infos = vec![
        accounts.nft_metadata.to_account_info(),
        accounts.nft_mint.to_account_info(),
        accounts.nft_mint_authority.to_account_info(),
        accounts.payer.to_account_info(),
        accounts.authority_pda.to_account_info(),
        accounts.nft_master_edition.to_account_info(),
        accounts.system_program.to_account_info(),
        accounts.authority_pda.to_account_info(),
    ];

    let master_edition_infos = vec![
        accounts.nft_master_edition.to_account_info(),
        accounts.nft_mint.to_account_info(),
        accounts.nft_mint_authority.to_account_info(),
        accounts.payer.to_account_info(),
        accounts.nft_metadata.to_account_info(),
        accounts.token_metadata_program.to_account_info(),
        accounts.spl_token_program.to_account_info(),
        accounts.system_program.to_account_info(),
        accounts.authority_pda.to_account_info(),
    ];

    let cm_key = candy_machine.key();
    let authority_seeds = [AUTHORITY_SEED.as_bytes(), cm_key.as_ref(), &[bump]];

    invoke_signed(
        &create_metadata_accounts_v3(
            accounts.token_metadata_program.key(),
            accounts.nft_metadata.key(),
            accounts.nft_mint.key(),
            accounts.nft_mint_authority.key(),
            accounts.payer.key(),
            accounts.authority_pda.key(),
            config_line.name,
            candy_machine.data.symbol.clone(),
            config_line.uri,
            Some(creators),
            candy_machine.data.seller_fee_basis_points,
            true,
            candy_machine.data.is_mutable,
            None,
            None,
            None,
        ),
        metadata_infos.as_slice(),
        &[&authority_seeds],
    )?;

    invoke_signed(
        &create_master_edition_v3(
            accounts.token_metadata_program.key(),
            accounts.nft_master_edition.key(),
            accounts.nft_mint.key(),
            accounts.authority_pda.key(),
            accounts.nft_mint_authority.key(),
            accounts.nft_metadata.key(),
            accounts.payer.key(),
            Some(candy_machine.data.max_supply),
        ),
        master_edition_infos.as_slice(),
        &[&authority_seeds],
    )?;

    invoke_signed(
        &update_metadata_accounts_v2(
            accounts.token_metadata_program.key(),
            accounts.nft_metadata.key(),
            accounts.authority_pda.key(),
            Some(collection_metadata.update_authority),
            None,
            Some(true),
            if !candy_machine.data.is_mutable {
                Some(false)
            } else {
                None
            },
        ),
        &[
            accounts.token_metadata_program.to_account_info(),
            accounts.nft_metadata.to_account_info(),
            accounts.authority_pda.to_account_info(),
        ],
        &[&authority_seeds],
    )?;

    let collection_mint = &accounts.collection_mint;
    let collection_master_edition = &accounts.collection_master_edition;
    let set_collection_ix = if collection_metadata.collection_details.is_some() {
        set_and_verify_sized_collection_item(
            accounts.token_metadata_program.key(),
            accounts.nft_metadata.key(),
            accounts.authority_pda.key(),
            accounts.payer.key(),
            accounts.collection_update_authority.key(),
            collection_mint.key(),
            accounts.collection_metadata.key(),
            collection_master_edition.key(),
            Some(accounts.collection_delegate_record.key()),
        )
    } else {
        set_and_verify_collection(
            accounts.token_metadata_program.key(),
            accounts.nft_metadata.key(),
            accounts.authority_pda.key(),
            accounts.payer.key(),
            accounts.collection_update_authority.key(),
            collection_mint.key(),
            accounts.collection_metadata.key(),
            collection_master_edition.key(),
            Some(accounts.collection_delegate_record.key()),
        )
    };

    let set_collection_infos = vec![
        accounts.nft_metadata.to_account_info(),
        accounts.authority_pda.to_account_info(),
        accounts.collection_update_authority.to_account_info(),
        accounts.payer.to_account_info(),
        collection_mint.to_account_info(),
        accounts.collection_metadata.to_account_info(),
        collection_master_edition.to_account_info(),
        accounts.collection_delegate_record.to_account_info(),
    ];

    invoke_signed(
        &set_collection_ix,
        set_collection_infos.as_slice(),
        &[&authority_seeds],
    )
    .map_err(|error| error.into())
}

/// Mints a new NFT.
#[derive(Accounts)]
pub struct MintV2<'info> {
    /// Candy machine account.
    #[account(mut, has_one = mint_authority)]
    candy_machine: Box<Account<'info, CandyMachine>>,

    /// Candy machine authority account. This is the account that holds a delegate
    /// to verify an item into the collection.
    ///
    /// CHECK: account constraints checked in account trait
    #[account(mut, seeds = [AUTHORITY_SEED.as_bytes(), candy_machine.key().as_ref()], bump)]
    authority_pda: UncheckedAccount<'info>,

    /// Candy machine mint authority (mint only allowed for the mint_authority).
    mint_authority: Signer<'info>,

    /// Payer for the transaction and account allocation (rent).
    #[account(mut)]
    payer: Signer<'info>,

    /// Mint account of the NFT. The account will be initialized if necessary.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_mint: UncheckedAccount<'info>,

    /// Mint authority of the NFT. In most cases this will be the owner of the NFT.
    nft_mint_authority: Signer<'info>,

    /// Metadata account of the NFT. This account must be uninitialized.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_metadata: UncheckedAccount<'info>,

    /// Master edition account of the NFT. The account will be initialized if necessary.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    nft_master_edition: UncheckedAccount<'info>,

    /// Destination token account (required for pNFT).
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    token: Option<UncheckedAccount<'info>>,

    /// Token record (required for pNFT).
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    token_record: Option<UncheckedAccount<'info>>,

    /// Collection authority or metadata delegate record.
    ///
    /// CHECK: account checked in CPI
    collection_delegate_record: UncheckedAccount<'info>,

    /// Mint account of the collection NFT.
    ///
    /// CHECK: account checked in CPI
    collection_mint: UncheckedAccount<'info>,

    /// Metadata account of the collection NFT.
    ///
    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_metadata: UncheckedAccount<'info>,

    /// Master edition account of the collection NFT.
    ///
    /// CHECK: account checked in CPI
    collection_master_edition: UncheckedAccount<'info>,

    /// Update authority of the collection NFT.
    ///
    /// CHECK: account checked in CPI
    collection_update_authority: UncheckedAccount<'info>,

    /// Token Metadata program.
    ///
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,

    /// SPL Token program.
    spl_token_program: Program<'info, Token>,

    /// SPL Associated Token program.
    spl_ata_program: Option<Program<'info, AssociatedToken>>,

    /// System program.
    system_program: Program<'info, System>,

    /// Instructions sysvar account.
    ///
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    sysvar_instructions: Option<UncheckedAccount<'info>>,

    /// SlotHashes sysvar cluster data.
    ///
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::slot_hashes::id())]
    recent_slothashes: UncheckedAccount<'info>,

    /// Token Authorization Rules program.
    ///
    /// CHECK: account checked in CPI
    #[account(address = mpl_token_auth_rules::id())]
    authorization_rules_program: Option<UncheckedAccount<'info>>,

    /// Token Authorization rules account for the collection metadata (if any).
    ///
    /// CHECK: account constraints checked in account trait
    #[account(owner = mpl_token_auth_rules::id())]
    authorization_rules: Option<UncheckedAccount<'info>>,
}
