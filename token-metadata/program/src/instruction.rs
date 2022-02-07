use crate::{
    deprecated_instruction::{MintPrintingTokensViaTokenArgs, SetReservationListArgs},
    state::{Collection, Creator, Data, DataV2, Uses, EDITION, EDITION_MARKER_BIT_SIZE, PREFIX},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for update call
pub struct UpdateMetadataAccountArgs {
    pub data: Option<Data>,
    pub update_authority: Option<Pubkey>,
    pub primary_sale_happened: Option<bool>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for update call
pub struct UpdateMetadataAccountArgsV2 {
    pub data: Option<DataV2>,
    pub update_authority: Option<Pubkey>,
    pub primary_sale_happened: Option<bool>,
    pub is_mutable: Option<bool>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for create call
pub struct CreateMetadataAccountArgs {
    /// Note that unique metadatas are disabled for now.
    pub data: Data,
    /// Whether you want your metadata to be updateable in the future.
    pub is_mutable: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for create call
pub struct CreateMetadataAccountArgsV2 {
    /// Note that unique metadatas are disabled for now.
    pub data: DataV2,
    /// Whether you want your metadata to be updateable in the future.
    pub is_mutable: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CreateMasterEditionArgs {
    /// If set, means that no more than this number of editions can ever be minted. This is immutable.
    pub max_supply: Option<u64>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct MintNewEditionFromMasterEditionViaTokenArgs {
    pub edition: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ApproveUseAuthorityArgs {
    pub number_of_uses: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UtilizeArgs {
    pub number_of_uses: u64,
}

/// Instructions supported by the Metadata program.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum MetadataInstruction {
    /// Create Metadata object.
    ///   0. `[writable]`  Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[]` Mint of token asset
    ///   2. `[signer]` Mint authority
    ///   3. `[signer]` payer
    ///   4. `[]` update authority info
    ///   5. `[]` System program
    ///   6. `[]` Rent info
    CreateMetadataAccount(CreateMetadataAccountArgs),

    /// Update a Metadata
    ///   0. `[writable]` Metadata account
    ///   1. `[signer]` Update authority key
    UpdateMetadataAccount(UpdateMetadataAccountArgs),

    /// Register a Metadata as a Master Edition V1, which means Editions can be minted.
    /// Henceforth, no further tokens will be mintable from this primary mint. Will throw an error if more than one
    /// token exists, and will throw an error if less than one token exists in this primary mint.
    ///   0. `[writable]` Unallocated edition V1 account with address as pda of ['metadata', program id, mint, 'edition']
    ///   1. `[writable]` Metadata mint
    ///   2. `[writable]` Printing mint - A mint you control that can mint tokens that can be exchanged for limited editions of your
    ///       master edition via the MintNewEditionFromMasterEditionViaToken endpoint
    ///   3. `[writable]` One time authorization printing mint - A mint you control that prints tokens that gives the bearer permission to mint any
    ///                  number of tokens from the printing mint one time via an endpoint with the token-metadata program for your metadata. Also burns the token.
    ///   4. `[signer]` Current Update authority key
    ///   5. `[signer]`   Printing mint authority - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY.
    ///   6. `[signer]` Mint authority on the metadata's mint - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   7. `[]` Metadata account
    ///   8. `[signer]` payer
    ///   9. `[]` Token program
    ///   10. `[]` System program
    ///   11. `[]` Rent info
    ///   13. `[signer]`   One time authorization printing mint authority - must be provided if using max supply. THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY.
    DeprecatedCreateMasterEdition(CreateMasterEditionArgs),

    /// Given an authority token minted by the Printing mint of a master edition, and a brand new non-metadata-ed mint with one token
    /// make a new Metadata + Edition that is a child of the master edition denoted by this authority token.
    ///   0. `[writable]` New Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[writable]` New Edition V1 (pda of ['metadata', program id, mint id, 'edition'])
    ///   2. `[writable]` Master Record Edition V1 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   3. `[writable]` Mint of new token - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[signer]` Mint authority of new mint
    ///   5. `[writable]` Printing Mint of master record edition
    ///   6. `[writable]` Token account containing Printing mint token to be transferred
    ///   7. `[writable]` Edition pda to mark creation - will be checked for pre-existence. (pda of ['metadata', program id, master mint id, edition_number])
    ///   8. `[signer]` Burn authority for this token
    ///   9. `[signer]` payer
    ///   10. `[]` update authority info for new metadata account
    ///   11. `[]` Master record metadata account
    ///   12. `[]` Token program
    ///   13. `[]` System program
    ///   14. `[]` Rent info
    ///   15. `[optional/writable]` Reservation List - If present, and you are on this list, you can get
    ///        an edition number given by your position on the list.
    DeprecatedMintNewEditionFromMasterEditionViaPrintingToken,

    /// Allows updating the primary sale boolean on Metadata solely through owning an account
    /// containing a token from the metadata's mint and being a signer on this transaction.
    /// A sort of limited authority for limited update capability that is required for things like
    /// Metaplex to work without needing full authority passing.
    ///
    ///   0. `[writable]` Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[signer]` Owner on the token account
    ///   2. `[]` Account containing tokens from the metadata's mint
    UpdatePrimarySaleHappenedViaToken,

    /// Reserve up to 200 editions in sequence for up to 200 addresses in an existing reservation PDA, which can then be used later by
    /// redeemers who have printing tokens as a reservation to get a specific edition number
    /// as opposed to whatever one is currently listed on the master edition. Used by Auction Manager
    /// to guarantee printing order on bid redemption. AM will call whenever the first person redeems a
    /// printing bid to reserve the whole block
    /// of winners in order and then each winner when they get their token submits their mint and account
    /// with the pda that was created by that first bidder - the token metadata can then cross reference
    /// these people with the list and see that bidder A gets edition #2, so on and so forth.
    ///
    /// NOTE: If you have more than 20 addresses in a reservation list, this may be called multiple times to build up the list,
    /// otherwise, it simply wont fit in one transaction. Only provide a total_reservation argument on the first call, which will
    /// allocate the edition space, and in follow up calls this will specifically be unnecessary (and indeed will error.)
    ///
    ///   0. `[writable]` Master Edition V1 key (pda of ['metadata', program id, mint id, 'edition'])
    ///   1. `[writable]` PDA for ReservationList of ['metadata', program id, master edition key, 'reservation', resource-key]
    ///   2. `[signer]` The resource you tied the reservation list too
    DeprecatedSetReservationList(SetReservationListArgs),

    /// Create an empty reservation list for a resource who can come back later as a signer and fill the reservation list
    /// with reservations to ensure that people who come to get editions get the number they expect. See SetReservationList for more.
    ///
    ///   0. `[writable]` PDA for ReservationList of ['metadata', program id, master edition key, 'reservation', resource-key]
    ///   1. `[signer]` Payer
    ///   2. `[signer]` Update authority
    ///   3. `[]` Master Edition V1 key (pda of ['metadata', program id, mint id, 'edition'])
    ///   4. `[]` A resource you wish to tie the reservation list to. This is so your later visitors who come to
    ///       redeem can derive your reservation list PDA with something they can easily get at. You choose what this should be.
    ///   5. `[]` Metadata key (pda of ['metadata', program id, mint id])
    ///   6. `[]` System program
    ///   7. `[]` Rent info
    DeprecatedCreateReservationList,

    /// Sign a piece of metadata that has you as an unverified creator so that it is now verified.
    ///   0. `[writable]` Metadata (pda of ['metadata', program id, mint id])
    ///   1. `[signer]` Creator
    SignMetadata,

    /// Using a one time authorization token from a master edition v1, print any number of printing tokens from the printing_mint
    /// one time, burning the one time authorization token.
    ///
    ///   0. `[writable]` Destination account
    ///   1. `[writable]` Token account containing one time authorization token
    ///   2. `[writable]` One time authorization mint
    ///   3. `[writable]` Printing mint
    ///   4. `[signer]` Burn authority
    ///   5. `[]` Metadata key (pda of ['metadata', program id, mint id])
    ///   6. `[]` Master Edition V1 key (pda of ['metadata', program id, mint id, 'edition'])
    ///   7. `[]` Token program
    ///   8. `[]` Rent
    DeprecatedMintPrintingTokensViaToken(MintPrintingTokensViaTokenArgs),

    /// Using your update authority, mint printing tokens for your master edition.
    ///
    ///   0. `[writable]` Destination account
    ///   1. `[writable]` Printing mint
    ///   2. `[signer]` Update authority
    ///   3. `[]` Metadata key (pda of ['metadata', program id, mint id])
    ///   4. `[]` Master Edition V1 key (pda of ['metadata', program id, mint id, 'edition'])
    ///   5. `[]` Token program
    ///   6. `[]` Rent
    DeprecatedMintPrintingTokens(MintPrintingTokensViaTokenArgs),

    /// Register a Metadata as a Master Edition V2, which means Edition V2s can be minted.
    /// Henceforth, no further tokens will be mintable from this primary mint. Will throw an error if more than one
    /// token exists, and will throw an error if less than one token exists in this primary mint.
    ///   0. `[writable]` Unallocated edition V2 account with address as pda of ['metadata', program id, mint, 'edition']
    ///   1. `[writable]` Metadata mint
    ///   2. `[signer]` Update authority
    ///   3. `[signer]` Mint authority on the metadata's mint - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[signer]` payer
    ///   5. `[]` Metadata account
    ///   6. `[]` Token program
    ///   7. `[]` System program
    ///   8. `[]` Rent info
    CreateMasterEdition(CreateMasterEditionArgs),

    /// Given a token account containing the master edition token to prove authority, and a brand new non-metadata-ed mint with one token
    /// make a new Metadata + Edition that is a child of the master edition denoted by this authority token.
    ///   0. `[writable]` New Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[writable]` New Edition (pda of ['metadata', program id, mint id, 'edition'])
    ///   2. `[writable]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   3. `[writable]` Mint of new token - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[writable]` Edition pda to mark creation - will be checked for pre-existence. (pda of ['metadata', program id, master metadata mint id, 'edition', edition_number])
    ///   where edition_number is NOT the edition number you pass in args but actually edition_number = floor(edition/EDITION_MARKER_BIT_SIZE).
    ///   5. `[signer]` Mint authority of new mint
    ///   6. `[signer]` payer
    ///   7. `[signer]` owner of token account containing master token (#8)
    ///   8. `[]` token account containing token from master metadata mint
    ///   9. `[]` Update authority info for new metadata
    ///   10. `[]` Master record metadata account
    ///   11. `[]` Token program
    ///   12. `[]` System program
    ///   13. `[]` Rent info
    MintNewEditionFromMasterEditionViaToken(MintNewEditionFromMasterEditionViaTokenArgs),

    /// Converts the Master Edition V1 to a Master Edition V2, draining lamports from the two printing mints
    /// to the owner of the token account holding the master edition token. Permissionless.
    /// Can only be called if there are currenly no printing tokens or one time authorization tokens in circulation.
    ///
    ///   0. `[writable]` Master Record Edition V1 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   1. `[writable]` One time authorization mint
    ///   2. `[writable]` Printing mint
    ConvertMasterEditionV1ToV2,

    /// Proxy Call to Mint Edition using a Store Token Account as a Vault Authority.
    ///
    ///   0. `[writable]` New Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[writable]` New Edition (pda of ['metadata', program id, mint id, 'edition'])
    ///   2. `[writable]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition']
    ///   3. `[writable]` Mint of new token - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[writable]` Edition pda to mark creation - will be checked for pre-existence. (pda of ['metadata', program id, master metadata mint id, 'edition', edition_number])
    ///   where edition_number is NOT the edition number you pass in args but actually edition_number = floor(edition/EDITION_MARKER_BIT_SIZE).
    ///   5. `[signer]` Mint authority of new mint
    ///   6. `[signer]` payer
    ///   7. `[signer]` Vault authority
    ///   8. `[]` Safety deposit token store account
    ///   9. `[]` Safety deposit box
    ///   10. `[]` Vault
    ///   11. `[]` Update authority info for new metadata
    ///   12. `[]` Master record metadata account
    ///   13. `[]` Token program
    ///   14. `[]` Token vault program
    ///   15. `[]` System program
    ///   16. `[]` Rent info
    MintNewEditionFromMasterEditionViaVaultProxy(MintNewEditionFromMasterEditionViaTokenArgs),

    /// Puff a Metadata - make all of it's variable length fields (name/uri/symbol) a fixed length using a null character
    /// so that it can be found using offset searches by the RPC to make client lookups cheaper.
    ///   0. `[writable]` Metadata account
    PuffMetadata,

    /// Update a Metadata with is_mutable as a parameter
    ///   0. `[writable]` Metadata account
    ///   1. `[signer]` Update authority key
    UpdateMetadataAccountV2(UpdateMetadataAccountArgsV2),

    /// Create Metadata object.
    ///   0. `[writable]`  Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[]` Mint of token asset
    ///   2. `[signer]` Mint authority
    ///   3. `[signer]` payer
    ///   4. `[]` update authority info
    ///   5. `[]` System program
    ///   6. `[]` Rent info
    CreateMetadataAccountV2(CreateMetadataAccountArgsV2),
    /// Register a Metadata as a Master Edition V2, which means Edition V2s can be minted.
    /// Henceforth, no further tokens will be mintable from this primary mint. Will throw an error if more than one
    /// token exists, and will throw an error if less than one token exists in this primary mint.
    ///   0. `[writable]` Unallocated edition V2 account with address as pda of ['metadata', program id, mint, 'edition']
    ///   1. `[writable]` Metadata mint
    ///   2. `[signer]` Update authority
    ///   3. `[signer]` Mint authority on the metadata's mint - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[signer]` payer
    ///   5.  [writable] Metadata account
    ///   6. `[]` Token program
    ///   7. `[]` System program
    ///   8. `[]` Rent info
    CreateMasterEditionV3(CreateMasterEditionArgs),
    ///See [verify_collection] for Doc
    VerifyCollection,
    ///See [utilize] for Doc
    Utilize(UtilizeArgs),
    ///See [approve_use_authority] for Doc
    ApproveUseAuthority(ApproveUseAuthorityArgs),
    ///See [revoke_use_authority] for Doc
    RevokeUseAuthority,
    ///See [unverify_collection] for Doc
    UnverifyCollection,
    ///See [approve_collection_authority] for Doc
    ApproveCollectionAuthority,
    ///See [revoke_collection_authority] for Doc
    RevokeCollectionAuthority,
    ///See [set_and_verify_collection] for Doc
    SetAndVerifyCollection,

    ///See [freeze_delegated_account] for Doc
    FreezeDelegatedAccount,
    ///See [thaw_delegated_account] for Doc
    ThawDelegatedAccount,
}

/// Creates an CreateMetadataAccounts instruction
/// #[deprecated(since="1.1.0", note="please use `create_metadata_accounts_v2` instead")]
#[allow(clippy::too_many_arguments)]
pub fn create_metadata_accounts(
    program_id: Pubkey,
    metadata_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>,
    seller_fee_basis_points: u16,
    update_authority_is_signer: bool,
    is_mutable: bool,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(update_authority, update_authority_is_signer),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::CreateMetadataAccount(CreateMetadataAccountArgs {
            data: Data {
                name,
                symbol,
                uri,
                seller_fee_basis_points,
                creators,
            },
            is_mutable,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// Creates an CreateMetadataAccounts instruction
#[allow(clippy::too_many_arguments)]
pub fn create_metadata_accounts_v2(
    program_id: Pubkey,
    metadata_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>,
    seller_fee_basis_points: u16,
    update_authority_is_signer: bool,
    is_mutable: bool,
    collection: Option<Collection>,
    uses: Option<Uses>,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(update_authority, update_authority_is_signer),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::CreateMetadataAccountV2(CreateMetadataAccountArgsV2 {
            data: DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points,
                creators,
                collection,
                uses,
            },
            is_mutable,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// update metadata account instruction
/// #[deprecated(since="1.1.0", note="please use `update_metadata_accounts_v2` instead")]
pub fn update_metadata_accounts(
    program_id: Pubkey,
    metadata_account: Pubkey,
    update_authority: Pubkey,
    new_update_authority: Option<Pubkey>,
    data: Option<Data>,
    primary_sale_happened: Option<bool>,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata_account, false),
            AccountMeta::new_readonly(update_authority, true),
        ],
        data: MetadataInstruction::UpdateMetadataAccount(UpdateMetadataAccountArgs {
            data,
            update_authority: new_update_authority,
            primary_sale_happened,
        })
        .try_to_vec()
        .unwrap(),
    }
}

// update metadata account v2 instruction
pub fn update_metadata_accounts_v2(
    program_id: Pubkey,
    metadata_account: Pubkey,
    update_authority: Pubkey,
    new_update_authority: Option<Pubkey>,
    data: Option<DataV2>,
    primary_sale_happened: Option<bool>,
    is_mutable: Option<bool>,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata_account, false),
            AccountMeta::new_readonly(update_authority, true),
        ],
        data: MetadataInstruction::UpdateMetadataAccountV2(UpdateMetadataAccountArgsV2 {
            data,
            update_authority: new_update_authority,
            primary_sale_happened,
            is_mutable,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// puff metadata account instruction
pub fn puff_metadata_account(program_id: Pubkey, metadata_account: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![AccountMeta::new(metadata_account, false)],
        data: MetadataInstruction::PuffMetadata.try_to_vec().unwrap(),
    }
}

/// creates a update_primary_sale_happened_via_token instruction
#[allow(clippy::too_many_arguments)]
pub fn update_primary_sale_happened_via_token(
    program_id: Pubkey,
    metadata: Pubkey,
    owner: Pubkey,
    token: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(token, false),
        ],
        data: MetadataInstruction::UpdatePrimarySaleHappenedViaToken
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a create_master_edition instruction
#[allow(clippy::too_many_arguments)]
/// [deprecated(since="1.1.0", note="please use `create_master_edition_v3` instead")]
pub fn create_master_edition(
    program_id: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
    update_authority: Pubkey,
    mint_authority: Pubkey,
    metadata: Pubkey,
    payer: Pubkey,
    max_supply: Option<u64>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(edition, false),
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(update_authority, true),
        AccountMeta::new_readonly(mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::CreateMasterEdition(CreateMasterEditionArgs { max_supply })
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a create_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn create_master_edition_v3(
    program_id: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
    update_authority: Pubkey,
    mint_authority: Pubkey,
    metadata: Pubkey,
    payer: Pubkey,
    max_supply: Option<u64>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(edition, false),
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(update_authority, true),
        AccountMeta::new_readonly(mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new(metadata, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::CreateMasterEditionV3(CreateMasterEditionArgs { max_supply })
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a mint_new_edition_from_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn mint_new_edition_from_master_edition_via_token(
    program_id: Pubkey,
    new_metadata: Pubkey,
    new_edition: Pubkey,
    master_edition: Pubkey,
    new_mint: Pubkey,
    new_mint_authority: Pubkey,
    payer: Pubkey,
    token_account_owner: Pubkey,
    token_account: Pubkey,
    new_metadata_update_authority: Pubkey,
    metadata: Pubkey,
    metadata_mint: Pubkey,
    edition: u64,
) -> Instruction {
    let edition_number = edition.checked_div(EDITION_MARKER_BIT_SIZE).unwrap();
    let as_string = edition_number.to_string();
    let (edition_mark_pda, _) = Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            metadata_mint.as_ref(),
            EDITION.as_bytes(),
            as_string.as_bytes(),
        ],
        &program_id,
    );

    let accounts = vec![
        AccountMeta::new(new_metadata, false),
        AccountMeta::new(new_edition, false),
        AccountMeta::new(master_edition, false),
        AccountMeta::new(new_mint, false),
        AccountMeta::new(edition_mark_pda, false),
        AccountMeta::new_readonly(new_mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(token_account_owner, true),
        AccountMeta::new_readonly(token_account, false),
        AccountMeta::new_readonly(new_metadata_update_authority, false),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::MintNewEditionFromMasterEditionViaToken(
            MintNewEditionFromMasterEditionViaTokenArgs { edition },
        )
        .try_to_vec()
        .unwrap(),
    }
}

/// Sign Metadata
#[allow(clippy::too_many_arguments)]
pub fn sign_metadata(program_id: Pubkey, metadata: Pubkey, creator: Pubkey) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new_readonly(creator, true),
        ],
        data: MetadataInstruction::SignMetadata.try_to_vec().unwrap(),
    }
}

/// Converts a master edition v1 to v2
#[allow(clippy::too_many_arguments)]
pub fn convert_master_edition_v1_to_v2(
    program_id: Pubkey,
    master_edition: Pubkey,
    one_time_auth: Pubkey,
    printing_mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(master_edition, false),
            AccountMeta::new(one_time_auth, false),
            AccountMeta::new(printing_mint, false),
        ],
        data: MetadataInstruction::ConvertMasterEditionV1ToV2
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a mint_edition_proxy instruction
#[allow(clippy::too_many_arguments)]
pub fn mint_edition_from_master_edition_via_vault_proxy(
    program_id: Pubkey,
    new_metadata: Pubkey,
    new_edition: Pubkey,
    master_edition: Pubkey,
    new_mint: Pubkey,
    edition_mark_pda: Pubkey,
    new_mint_authority: Pubkey,
    payer: Pubkey,
    vault_authority: Pubkey,
    safety_deposit_store: Pubkey,
    safety_deposit_box: Pubkey,
    vault: Pubkey,
    new_metadata_update_authority: Pubkey,
    metadata: Pubkey,
    token_program: Pubkey,
    token_vault_program_info: Pubkey,
    edition: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(new_metadata, false),
        AccountMeta::new(new_edition, false),
        AccountMeta::new(master_edition, false),
        AccountMeta::new(new_mint, false),
        AccountMeta::new(edition_mark_pda, false),
        AccountMeta::new_readonly(new_mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(vault_authority, true),
        AccountMeta::new_readonly(safety_deposit_store, false),
        AccountMeta::new_readonly(safety_deposit_box, false),
        AccountMeta::new_readonly(vault, false),
        AccountMeta::new_readonly(new_metadata_update_authority, false),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(token_vault_program_info, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::MintNewEditionFromMasterEditionViaVaultProxy(
            MintNewEditionFromMasterEditionViaTokenArgs { edition },
        )
        .try_to_vec()
        .unwrap(),
    }
}

/// # Verify Collection
///
/// If a MetadataAccount Has a Collection allow the UpdateAuthority of the Collection to Verify the NFT Belongs in the Collection
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Update authority
///   2. `[signer]` payer
///   3. `[]` Mint of the Collection
///   4. `[]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn verify_collection(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    payer: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(collection_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new_readonly(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if collection_authority_record.is_some() {
        accounts.push(AccountMeta::new_readonly(
            collection_authority_record.unwrap(),
            false,
        ));
    }
    Instruction {
        program_id,
        accounts: accounts,
        data: MetadataInstruction::VerifyCollection.try_to_vec().unwrap(),
    }
}

/// # Unverify Collection
///
/// If a MetadataAccount Has a Collection allow an Authority of the Collection to unverify an NFT in a Collection
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Authority
///   2. `[signer]` payer
///   3. `[]` Mint of the Collection
///   4. `[]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn unverify_collection(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(collection_authority, true),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new_readonly(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if collection_authority_record.is_some() {
        accounts.push(AccountMeta::new_readonly(
            collection_authority_record.unwrap(),
            false,
        ));
    }
    Instruction {
        program_id,
        accounts: accounts,
        data: MetadataInstruction::UnverifyCollection
            .try_to_vec()
            .unwrap(),
    }
}

///# Utilize
///
///Utilize or Use an NFT , burns the NFT and returns the lamports to the update authority if the use method is burn and its out of uses.
///Use Authority can be the Holder of the NFT, or a Delegated Use Authority.
///
///### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[writable]` Token Account Of NFT
///   2. `[writable]` Mint of the Metadata
///   2. `[signer]` A Use Authority / Can be the current Owner of the NFT
///   3. `[signer]` Payer
///   4. `[]` Owner
///   5. `[]` Token program
///   6. `[]` Associated Token program
///   7. `[]` System program
///   8. `[]` Rent info
///   9. Optional `[writable]` Use Authority Record PDA If present the program Assumes a delegated use authority
#[allow(clippy::too_many_arguments)]
pub fn utilize(
    program_id: Pubkey,
    metadata: Pubkey,
    token_account: Pubkey,
    mint: Pubkey,
    use_authority_record_pda: Option<Pubkey>,
    use_authority: Pubkey,
    owner: Pubkey,
    burner: Option<Pubkey>,
    number_of_uses: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(token_account, false),
        AccountMeta::new(mint, false),
        AccountMeta::new(use_authority, true),
        AccountMeta::new_readonly(owner, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    if use_authority_record_pda.is_some() {
        accounts.push(AccountMeta::new(use_authority_record_pda.unwrap(), false));
    }
    if burner.is_some() {
        accounts.push(AccountMeta::new_readonly(burner.unwrap(), false));
    }

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::Utilize(UtilizeArgs { number_of_uses })
            .try_to_vec()
            .unwrap(),
    }
}

///# Approve Use Authority
///
///Approve another account to call [utilize] on this NFT
///
///### Args:
///
///See: [ApproveUseAuthorityArgs]
///
///### Accounts:
///
///   0. `[writable]` Use Authority Record PDA
///   1. `[writable]` Owned Token Account Of Mint
///   2. `[signer]` Owner
///   3. `[signer]` Payer
///   4. `[]` A Use Authority
///   5. `[]` Metadata account
///   6. `[]` Mint of Metadata
///   7. `[]` Program As Signer (Burner)
///   8. `[]` Token program
///   9. `[]` System program
///   10. `[]` Rent info
#[allow(clippy::too_many_arguments)]
pub fn approve_use_authority(
    program_id: Pubkey,
    use_authority_record: Pubkey,
    user: Pubkey,
    owner: Pubkey,
    payer: Pubkey,
    owner_token_account: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
    burner: Pubkey,
    number_of_uses: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(use_authority_record, false),
            AccountMeta::new(owner, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(user, false),
            AccountMeta::new(owner_token_account, false),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(burner, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::ApproveUseAuthority(ApproveUseAuthorityArgs { number_of_uses })
            .try_to_vec()
            .unwrap(),
    }
}

//# Revoke Use Authority
///
///Revoke account to call [utilize] on this NFT
///
///### Accounts:
///
///   0. `[writable]` Use Authority Record PDA
///   1. `[writable]` Owned Token Account Of Mint
///   2. `[signer]` Owner
///   3. `[signer]` Payer
///   4. `[]` A Use Authority
///   5. `[]` Metadata account
///   6. `[]` Mint of Metadata
///   7. `[]` Token program
///   8. `[]` System program
///   9. `[]` Rent info
#[allow(clippy::too_many_arguments)]
pub fn revoke_use_authority(
    program_id: Pubkey,
    use_authority_record: Pubkey,
    user: Pubkey,
    owner: Pubkey,
    owner_token_account: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(use_authority_record, false),
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(user, false),
            AccountMeta::new(owner_token_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::RevokeUseAuthority
            .try_to_vec()
            .unwrap(),
    }
}

///# Approve Collection Authority
///
///Approve another account to verify NFTs belonging to a collection, [verify_collection] on the collection NFT
///
///### Accounts:
///   0. `[writable]` Collection Authority Record PDA
///   1. `[signer]` Update Authority of Collection NFT
///   2. `[signer]` Payer
///   3. `[]` A Collection Authority
///   4. `[]` Collection Metadata account
///   5. `[]` Mint of Collection Metadata
///   6. `[]` Token program
///   7. `[]` System program
///   8. `[]` Rent info
#[allow(clippy::too_many_arguments)]
pub fn approve_collection_authority(
    program_id: Pubkey,
    collection_authority_record: Pubkey,
    new_collection_authority: Pubkey,
    update_authority: Pubkey,
    payer: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(collection_authority_record, false),
            AccountMeta::new_readonly(new_collection_authority, false),
            AccountMeta::new(update_authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: MetadataInstruction::ApproveCollectionAuthority
            .try_to_vec()
            .unwrap(),
    }
}

//# Revoke Collection Authority
///
///Revoke account to call [verify_collection] on this NFT
///
///### Accounts:
///
///   0. `[writable]` Use Authority Record PDA
///   1. `[writable]` Owned Token Account Of Mint
///   2. `[]` Metadata account
///   3. `[]` Mint of Metadata
#[allow(clippy::too_many_arguments)]
pub fn revoke_collection_authority(
    program_id: Pubkey,
    collection_authority_record: Pubkey,
    update_authority: Pubkey,
    metadata: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(collection_authority_record, false),
            AccountMeta::new(update_authority, true),
            AccountMeta::new_readonly(metadata, false),
            AccountMeta::new_readonly(mint, false),
        ],
        data: MetadataInstruction::RevokeCollectionAuthority
            .try_to_vec()
            .unwrap(),
    }
}

//# Set And Verify Collection
///
///Allows the same Update Authority (Or Delegated Authority) on an NFT and Collection to perform [update_metadata_accounts_v2] with collection and [verify_collection] on the NFT/Collection in one instruction
///
/// ### Accounts:
///
///   0. `[writable]` Metadata account
///   1. `[signer]` Collection Update authority
///   2. `[signer]` payer
///   3. `[] Update Authority of Collection NFT and NFT
///   3. `[]` Mint of the Collection
///   4. `[]` Metadata Account of the Collection
///   5. `[]` MasterEdition2 Account of the Collection Token
#[allow(clippy::too_many_arguments)]
pub fn set_and_verify_collection(
    program_id: Pubkey,
    metadata: Pubkey,
    collection_authority: Pubkey,
    payer: Pubkey,
    update_authority: Pubkey,
    collection_mint: Pubkey,
    collection: Pubkey,
    collection_master_edition_account: Pubkey,
    collection_authority_record: Option<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(metadata, false),
        AccountMeta::new(collection_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(update_authority, false),
        AccountMeta::new_readonly(collection_mint, false),
        AccountMeta::new_readonly(collection, false),
        AccountMeta::new_readonly(collection_master_edition_account, false),
    ];

    if collection_authority_record.is_some() {
        accounts.push(AccountMeta::new_readonly(
            collection_authority_record.unwrap(),
            false,
        ));
    }
    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::SetAndVerifyCollection
            .try_to_vec()
            .unwrap(),
    }
}

///# Freeze delegated account
///
///Allow freezing of an NFT if this user is the delegate of the NFT
///
///### Accounts:
///   0. `[signer]` Delegate
///   1. `[writable]` Token account to freeze
///   2. `[]` Edition
///   3. `[]` Token mint
#[allow(clippy::too_many_arguments)]
pub fn freeze_delegated_account(
    program_id: Pubkey,
    delegate: Pubkey,
    token_account: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(delegate, true),
            AccountMeta::new(token_account, false),
            AccountMeta::new_readonly(edition, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: MetadataInstruction::FreezeDelegatedAccount
            .try_to_vec()
            .unwrap(),
    }
}

///# Thaw delegated account
///
///Allow thawing of an NFT if this user is the delegate of the NFT
///
///### Accounts:
///   0. `[signer]` Delegate
///   1. `[writable]` Token account to thaw
///   2. `[]` Edition
///   3. `[]` Token mint
#[allow(clippy::too_many_arguments)]
pub fn thaw_delegated_account(
    program_id: Pubkey,
    delegate: Pubkey,
    token_account: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(delegate, true),
            AccountMeta::new(token_account, false),
            AccountMeta::new_readonly(edition, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: MetadataInstruction::ThawDelegatedAccount
            .try_to_vec()
            .unwrap(),
    }
}
