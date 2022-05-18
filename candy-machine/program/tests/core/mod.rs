use std::str::FromStr;

use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::Account, program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction, transaction::Transaction, transport::TransportError,
};
use spl_token::state::Mint;

pub mod master_edition_v2;
pub mod metadata;

/// Perform native lamports transfer.
pub async fn transfer_lamports(
    client: &mut ProgramTestContext,
    wallet: &Keypair,
    to: &Pubkey,
    amount: u64,
) -> Result<(), TransportError> {
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
) -> Result<spl_token::state::Account, TransportError> {
    let account = client.banks_client.get_account(*token_account).await?;
    Ok(spl_token::state::Account::unpack(&account.unwrap().data).unwrap())
}

pub async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), TransportError> {
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

pub async fn create_associated_token_account(
    context: &mut ProgramTestContext,
    wallet: &Pubkey,
    token_mint: &Pubkey,
) -> Result<Pubkey, TransportError> {
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

    // connection.send_and_confirm_transaction(&tx)?;
    context.banks_client.process_transaction(tx).await.unwrap();

    Ok(spl_associated_token_account::get_associated_token_address(
        wallet, token_mint,
    ))
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

pub async fn get_mint(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Mint {
    let account = get_account(context, pubkey).await;
    Mint::unpack(&account.data).unwrap()
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
    owner: &Pubkey,
    additional_signer: Option<&Keypair>,
) -> Result<(), TransportError> {
    let mut signing_keypairs = vec![authority];
    if let Some(signer) = additional_signer {
        signing_keypairs.push(signer);
    }

    let tx = Transaction::new_signed_with_payer(
        &[
            spl_token::instruction::mint_to(&spl_token::id(), mint, account, owner, &[], amount)
                .unwrap(),
        ],
        Some(&authority.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
) -> Result<(), TransportError> {
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
        &[&context.payer, &account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Result<(), TransportError> {
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
                &authority,
                freeze_authority,
                decimals,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub struct TokenAllocation {
    pub pubkey: Pubkey,
    pub amount: u64,
}

pub async fn new_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    authority: &Keypair,
    decimals: u8,
    amount: u64,
    owner: &Pubkey,
) -> Result<Pubkey, TransportError> {
    create_mint(
        context,
        mint,
        &authority.pubkey(),
        Some(&authority.pubkey()),
        decimals,
    )
    .await?;
    let mint_pubkey = mint.pubkey();
    let ata = create_associated_token_account(context, owner, &mint_pubkey).await?;
    mint_tokens(context, authority, &mint_pubkey, &ata, amount, owner, None).await?;
    Ok(ata)
}

pub async fn ata_and_mint(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    mint_pubkey: &Pubkey,
    allocation: TokenAllocation,
) -> Result<Pubkey, TransportError> {
    let ata = create_associated_token_account(context, &allocation.pubkey, mint_pubkey).await?;
    mint_tokens(
        context,
        authority,
        mint_pubkey,
        &ata,
        allocation.amount,
        &allocation.pubkey,
        None,
    )
    .await?;
    Ok(ata)
}

pub async fn transfer(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    from: &Keypair,
    to: &Keypair,
) -> Result<(), TransportError> {
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
