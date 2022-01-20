#![allow(unused)]

use super::helpers::{
    airdrop, create_master_edition, create_mint, create_token_account, create_token_metadata,
    mint_to,
};
use anchor_lang::{InstructionData, ToAccountMetas};
use chrono::NaiveDate;
use mpl_membership_token::{
    accounts as mpl_membership_token_accounts, instruction as mpl_membership_token_instruction,
    utils::{find_treasury_owner_address, find_vault_owner_address},
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    instruction::Instruction, signature::Keypair, signer::Signer, system_program, sysvar,
    transaction::Transaction,
};
use std::time::SystemTime;

/// Seup Program Test Context
#[macro_export]
macro_rules! setup_context {
    ( $context:ident, $( $program_name:ident ),+ ) => {
        let mut program_test = ProgramTest::default();
        $(
            program_test.add_program(stringify!($program_name), $program_name::id(), None);
        )+
        let mut $context = program_test.start_with_context().await;
    };
}

/// Setup Store with default options
pub async fn setup_store(context: &mut ProgramTestContext) -> (Keypair, Keypair) {
    let admin_wallet = Keypair::new();
    let store_keypair = Keypair::new();

    airdrop(context, &admin_wallet.pubkey(), 10_000_000_000).await;

    let name = "Test store".to_string();
    let description = "Just a test store".to_string();

    let accounts = mpl_membership_token_accounts::CreateStore {
        admin: admin_wallet.pubkey(),
        store: store_keypair.pubkey(),
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = mpl_membership_token_instruction::CreateStore {
        name: name.to_owned(),
        description: description.to_owned(),
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_membership_token::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &admin_wallet, &store_keypair],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    (admin_wallet, store_keypair)
}

/// Setup default selling resource
pub async fn setup_selling_resource(
    context: &mut ProgramTestContext,
    admin_wallet: &Keypair,
    store_keypair: &Keypair,
) -> (Keypair, Keypair, Keypair) {
    // Create `SellingResource`
    let resource_mint = Keypair::new();
    create_mint(context, &resource_mint, &admin_wallet.pubkey(), 0).await;

    let resource_token = Keypair::new();
    create_token_account(
        context,
        &resource_token,
        &resource_mint.pubkey(),
        &admin_wallet.pubkey(),
    )
    .await;

    let (vault_owner, vault_owner_bump) =
        find_vault_owner_address(&resource_mint.pubkey(), &store_keypair.pubkey());

    let vault = Keypair::new();
    create_token_account(context, &vault, &resource_mint.pubkey(), &vault_owner).await;

    mint_to(
        context,
        &resource_mint.pubkey(),
        &resource_token.pubkey(),
        &admin_wallet,
        1,
    )
    .await;

    // Create metadata
    let metadata = create_token_metadata(
        context,
        &resource_mint.pubkey(),
        &admin_wallet,
        String::from("TEST"),
        String::from("TST"),
        String::from("https://github.com/"),
        100,
        false,
        false,
    )
    .await;

    // Create MasterEdition
    let (master_edition, master_edition_bump) = create_master_edition(
        context,
        &resource_mint.pubkey(),
        &admin_wallet,
        &metadata,
        Some(1),
    )
    .await;

    let selling_resource_keypair = Keypair::new();
    let selling_resource_owner_keypair = Keypair::new();

    airdrop(
        context,
        &selling_resource_owner_keypair.pubkey(),
        10_000_000_000,
    )
    .await;

    let accounts = mpl_membership_token_accounts::InitSellingResource {
        store: store_keypair.pubkey(),
        admin: admin_wallet.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        resource_mint: resource_mint.pubkey(),
        master_edition,
        vault: vault.pubkey(),
        owner: vault_owner,
        resource_token: resource_token.pubkey(),
        rent: sysvar::rent::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = mpl_membership_token_instruction::InitSellingResource {
        _master_edition_bump: master_edition_bump,
        _vault_owner_bump: vault_owner_bump,
        max_supply: Some(1),
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_membership_token::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &admin_wallet, &selling_resource_keypair],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    (
        selling_resource_keypair,
        selling_resource_owner_keypair,
        vault,
    )
}

pub async fn setup_market(
    context: &mut ProgramTestContext,
    admin_wallet: &Keypair,
    store_keypair: &Keypair,
    selling_resource_keypair: &Keypair,
    selling_resource_owner_keypair: &Keypair,
) -> Keypair {
    let market_keypair = Keypair::new();

    let treasury_mint_keypair = Keypair::new();
    create_mint(context, &treasury_mint_keypair, &admin_wallet.pubkey(), 0).await;

    let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
        &treasury_mint_keypair.pubkey(),
        &selling_resource_keypair.pubkey(),
    );

    let treasury_holder_keypair = Keypair::new();
    create_token_account(
        context,
        &treasury_holder_keypair,
        &treasury_mint_keypair.pubkey(),
        &treasury_owner,
    )
    .await;

    let start_date = std::time::SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 5;

    let name = "Marktname".to_string();
    let description = "Marktbeschreibung".to_string();
    let mutable = true;
    let price = 1_000_000;
    let pieces_in_one_wallet = Some(1);

    let accounts = mpl_membership_token_accounts::CreateMarket {
        market: market_keypair.pubkey(),
        store: store_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        mint: treasury_mint_keypair.pubkey(),
        treasury_holder: treasury_holder_keypair.pubkey(),
        owner: treasury_owner,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = mpl_membership_token_instruction::CreateMarket {
        _treasyry_owner_bump: treasyry_owner_bump,
        name: name.to_owned(),
        description: description.to_owned(),
        mutable,
        price,
        pieces_in_one_wallet,
        start_date,
        end_date: None,
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_membership_token::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[
            &context.payer,
            &market_keypair,
            &selling_resource_owner_keypair,
        ],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    market_keypair
}
