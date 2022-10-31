#![allow(unused)]

use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use mpl_testing_utils::assert_error;
use mpl_token_metadata::state::Collection;
use solana_program::instruction::InstructionError;
use solana_program::{clock::Clock, system_instruction};
use solana_program_test::*;
use solana_sdk::{
    commitment_config::CommitmentLevel, program_pack::Pack, transaction::Transaction,
};
use solana_sdk::{transaction::TransactionError, transport::TransportError};
use std::convert::TryFrom;
use std::env;

pub async fn mint_to(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    to: &Pubkey,
    owner: &Keypair,
    amount: u64,
) {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            to,
            &owner.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, owner],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(
            tx,
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
        .unwrap();
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
) {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, account],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(
            tx,
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
        .unwrap();
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    authority: &Pubkey,
    decimals: u8,
) {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                authority,
                Some(&authority),
                decimals,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(
            tx,
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
        .unwrap();
}

pub async fn create_master_edition(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    update_authority: &Keypair,
    mint_authority: &Keypair,
    metadata: &Pubkey,
    max_supply: Option<u64>,
) -> (Pubkey, u8) {
    let (edition, edition_bump) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[mpl_token_metadata::instruction::create_master_edition_v3(
            mpl_token_metadata::id(),
            edition,
            *mint,
            update_authority.pubkey(),
            mint_authority.pubkey(),
            *metadata,
            context.payer.pubkey(),
            max_supply,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint_authority, update_authority],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(
            tx,
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
        .unwrap();

    (edition, edition_bump)
}

pub async fn create_token_metadata(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    mint_authority: &Keypair,
    update_authority: &Keypair,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<mpl_token_metadata::state::Creator>>,
    seller_fee_basis_points: u16,
    update_authority_is_signer: bool,
    is_mutable: bool,
    collection: Option<Collection>,
) -> Pubkey {
    let (metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            mpl_token_metadata::instruction::create_metadata_accounts_v2(
                mpl_token_metadata::id(),
                metadata,
                *mint,
                mint_authority.pubkey(),
                context.payer.pubkey(),
                update_authority.pubkey(),
                name,
                symbol,
                uri,
                creators,
                seller_fee_basis_points,
                update_authority_is_signer,
                is_mutable,
                collection,
                None,
            ),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint_authority, update_authority],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(tx, CommitmentLevel::Confirmed)
        .await
        .unwrap();

    metadata
}

pub async fn verify_collection(
    context: &mut ProgramTestContext,
    metadata: &Pubkey,
    collection_authority: &Keypair,
    collection_mint: &Pubkey,
) {
    let (collection_metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            collection_mint.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let (collection_master, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            collection_mint.as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[mpl_token_metadata::instruction::verify_collection(
            mpl_token_metadata::id(),
            *metadata,
            collection_authority.pubkey(),
            context.payer.pubkey(),
            *collection_mint,
            collection_metadata,
            collection_master,
            None,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, collection_authority],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(tx, CommitmentLevel::Confirmed)
        .await
        .unwrap();
}

pub async fn airdrop(context: &mut ProgramTestContext, receiver: &Pubkey, amount: u64) {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(tx, CommitmentLevel::Confirmed)
        .await
        .unwrap();
}

pub async fn create_collection(
    context: &mut ProgramTestContext,
    mint_authority: &Keypair,
) -> (Pubkey, Pubkey) {
    let mint = Keypair::new();
    create_mint(context, &mint, &mint_authority.pubkey(), 0).await;

    let token_account = Keypair::new();
    create_token_account(
        context,
        &token_account,
        &mint.pubkey(),
        &mint_authority.pubkey(),
    )
    .await;

    mint_to(
        context,
        &mint.pubkey(),
        &token_account.pubkey(),
        mint_authority,
        1,
    )
    .await;

    let collection_metadata = create_token_metadata(
        context,
        &mint.pubkey(),
        mint_authority,
        mint_authority,
        String::from("Collection"),
        String::from("CLC"),
        String::from("URL"),
        None,
        0,
        true,
        true,
        None,
    )
    .await;

    create_master_edition(
        context,
        &mint.pubkey(),
        mint_authority,
        mint_authority,
        &collection_metadata,
        Some(0),
    )
    .await;

    (mint.pubkey(), token_account.pubkey())
}

pub async fn create_master_nft(
    context: &mut ProgramTestContext,
    authority: &Keypair,     // new nft owner
    collection_key: &Pubkey, // collection mint
    collection_authority: &Keypair,
    verify: bool,
) -> (Pubkey, Pubkey, Pubkey) {
    let mint = Keypair::new();
    create_mint(context, &mint, &authority.pubkey(), 0).await;

    let token_account = Keypair::new();
    create_token_account(context, &token_account, &mint.pubkey(), &authority.pubkey()).await;

    mint_to(
        context,
        &mint.pubkey(),
        &token_account.pubkey(),
        authority,
        1,
    )
    .await;

    let collection = Collection {
        verified: false,
        key: *collection_key,
    };

    let metadata = create_token_metadata(
        context,
        &mint.pubkey(),
        authority,
        authority,
        String::from("Collection item"),
        String::from("CLCIT"),
        String::from("URI"),
        None,
        0,
        true,
        true,
        Some(collection),
    )
    .await;

    create_master_edition(
        context,
        &mint.pubkey(),
        authority,
        authority,
        &metadata,
        Some(0),
    )
    .await;

    if verify {
        verify_collection(context, &metadata, collection_authority, collection_key).await;
    }

    (mint.pubkey(), token_account.pubkey(), metadata)
}

/// In CI we're running into IoError(the request exceeded its deadline) which is most likely a
/// timing issue that happens due to decreased performance.
/// Increasing compute limits seems to have made this happen less often, but for a few tests we
/// still observe this behavior which makes tests fail in CI for the wrong reason.
/// The below is a workaround to make it even less likely.
/// Tests are still brittle, but fail much less often which is the best we can do for now aside
/// from disabling the problematic tests in CI entirely.
pub fn assert_error_ignoring_io_error_in_ci(error: &BanksClientError, error_code: u32) {
    match error {
        BanksClientError::Io(err) if env::var("CI").is_ok() => {
            match err.kind() {
                std::io::ErrorKind::Other
                    if &err.to_string() == "the request exceeded its deadline" =>
                {
                    eprintln!("Encountered {:#?} error", err);
                    eprintln!("However since we are running in CI this is acceptable and we can ignore it");
                }
                _ => {
                    eprintln!("Encountered {:#?} error ({})", err, err);
                    panic!("Encountered unknown IoError");
                }
            }
        }
        _ => {
            assert_error!(error, &error_code)
        }
    }
}

/// See `assert_error_ignoring_io_error_in_ci` for more details regarding this workaround
pub fn unwrap_ignoring_io_error_in_ci(result: Result<(), BanksClientError>) {
    match result {
        Ok(()) => (),
        Err(error) => match error {
            BanksClientError::Io(err) if env::var("CI").is_ok() => match err.kind() {
                std::io::ErrorKind::Other
                    if &err.to_string() == "the request exceeded its deadline" =>
                {
                    eprintln!("Encountered {:#?} error", err);
                    eprintln!("However since we are running in CI this is acceptable and we can ignore it");
                }
                _ => {
                    eprintln!("Encountered {:#?} error ({})", err, err);
                    panic!("Encountered unknown IoError");
                }
            },
            _ => {
                panic!("Encountered: {:#?}", error);
            }
        },
    }
}
