use std::str::FromStr;

use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::Account,
    program_pack::Pack,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    transport,
};
use spl_token::state::Mint;

use crate::core::{master_edition_v2::MasterEditionV2, metadata};

/// Perform native lamports transfer.
#[allow(dead_code)]
pub async fn transfer_lamports(
    client: &mut ProgramTestContext,
    wallet: &Keypair,
    to: &Pubkey,
    amount: u64,
) -> transport::Result<()> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(&wallet.pubkey(), to, amount)],
        Some(&wallet.pubkey()),
        &[wallet],
        client.last_blockhash,
    );

    client.banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn get_token_account(
    client: &mut ProgramTestContext,
    token_account: &Pubkey,
) -> transport::Result<spl_token::state::Account> {
    let account = client.banks_client.get_account(*token_account).await?;
    Ok(spl_token::state::Account::unpack(&account.unwrap().data).unwrap())
}

pub async fn get_balance(context: &mut ProgramTestContext, pubkey: &Pubkey) -> u64 {
    context.banks_client.get_balance(*pubkey).await.unwrap()
}

pub async fn get_token_balance(context: &mut ProgramTestContext, token_account: &Pubkey) -> u64 {
    get_token_account(context, token_account)
        .await
        .unwrap()
        .amount
}

pub async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> transport::Result<()> {
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

    context.banks_client.process_transaction(tx).await.unwrap();
    Ok(())
}

pub fn clone_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}

pub fn clone_pubkey(pubkey: &Pubkey) -> Pubkey {
    Pubkey::from_str(&pubkey.to_string()).unwrap()
}

pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn assert_account_empty(context: &mut ProgramTestContext, pubkey: &Pubkey) {
    let account = context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("Could not get account!");
    assert_eq!(account, None);
}

#[allow(dead_code)]
pub async fn get_mint(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Mint {
    let account = get_account(context, pubkey).await;
    Mint::unpack(&account.data).unwrap()
}

#[allow(dead_code)]
pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
) -> transport::Result<()> {
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

    context.banks_client.process_transaction(tx).await
}

pub async fn create_associated_token_account(
    context: &mut ProgramTestContext,
    wallet: &Pubkey,
    token_mint: &Pubkey,
) -> transport::Result<Pubkey> {
    let recent_blockhash = context.last_blockhash;

    let tx = Transaction::new_signed_with_payer(
        &[
            spl_associated_token_account::create_associated_token_account(
                &context.payer.pubkey(),
                wallet,
                token_mint,
            ),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        recent_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    Ok(spl_associated_token_account::get_associated_token_address(
        wallet, token_mint,
    ))
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    mint: Option<Keypair>,
) -> transport::Result<Keypair> {
    let mint = mint.unwrap_or_else(Keypair::new);
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                authority,
                freeze_authority,
                decimals,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
    Ok(mint)
}

pub async fn mint_to_wallets(
    context: &mut ProgramTestContext,
    mint_pubkey: &Pubkey,
    authority: &Keypair,
    allocations: Vec<(Pubkey, u64)>,
) -> transport::Result<Vec<Pubkey>> {
    let mut atas = Vec::with_capacity(allocations.len());

    #[allow(clippy::needless_range_loop)]
    for i in 0..allocations.len() {
        let ata = create_associated_token_account(context, &allocations[i].0, mint_pubkey).await?;
        // println!("Minting to wallet {}", i);
        // println!(
        //     "Token account ATA: {:#?}",
        //     get_token_account(context, &ata).await.unwrap()
        // );
        mint_tokens(
            context,
            authority,
            mint_pubkey,
            &ata,
            allocations[i].1,
            None,
        )
        .await?;
        atas.push(ata);
    }
    Ok(atas)
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
    additional_signer: Option<&Keypair>,
) -> transport::Result<()> {
    let mut signing_keypairs = vec![authority, &context.payer];
    if let Some(signer) = additional_signer {
        signing_keypairs.push(signer);
    }

    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        account,
        &authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

#[allow(dead_code)]
pub async fn transfer(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    from: &Keypair,
    to: &Keypair,
) -> transport::Result<()> {
    create_associated_token_account(context, &to.pubkey(), mint).await?;
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::transfer(
            &spl_token::id(),
            &from.pubkey(),
            &to.pubkey(),
            &from.pubkey(),
            &[&from.pubkey()],
            0,
        )
        .unwrap()],
        Some(&from.pubkey()),
        &[from],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn prepare_nft(context: &mut ProgramTestContext, minter: &Keypair) -> MasterEditionV2 {
    let nft_info = metadata::Metadata::new(minter);
    create_mint(
        context,
        &minter.pubkey(),
        Some(&minter.pubkey()),
        0,
        Some(clone_keypair(&nft_info.mint)),
    )
    .await
    .unwrap();
    mint_to_wallets(
        context,
        &nft_info.mint.pubkey(),
        minter,
        vec![(nft_info.token, 1)],
    )
    .await
    .unwrap();
    MasterEditionV2::new(&nft_info)
}
