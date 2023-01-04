mod assert;
mod edition_marker;
mod external_price;
mod master_edition_v2;
mod metadata;
mod vault;

pub use assert::*;
pub use edition_marker::EditionMarker;
pub use external_price::ExternalPrice;
pub use master_edition_v2::MasterEditionV2;
pub use metadata::{assert_collection_size, Metadata};
pub use mpl_token_metadata::instruction;
use mpl_token_metadata::state::CollectionDetails;
use solana_program_test::*;
use solana_sdk::{
    account::Account, program_pack::Pack, pubkey::Pubkey, signature::Signer,
    signer::keypair::Keypair, system_instruction, transaction::Transaction,
};
use spl_token_2022::{
    extension::{ExtensionType, StateWithExtensions},
    state::Mint,
};
pub use vault::Vault;

pub const DEFAULT_COLLECTION_DETAILS: Option<CollectionDetails> =
    Some(CollectionDetails::V1 { size: 0 });

pub fn program_test() -> ProgramTest {
    let mut program_test = ProgramTest::new("mpl_token_metadata", mpl_token_metadata::id(), None);
    program_test.prefer_bpf(false);
    program_test.add_program(
        "spl_token_2022",
        spl_token_2022::id(),
        processor!(spl_token_2022::processor::Processor::process),
    );
    program_test
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
    StateWithExtensions::<Mint>::unpack(&account.data)
        .unwrap()
        .base
}

pub async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
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

pub async fn burn(
    context: &mut ProgramTestContext,
    metadata: Pubkey,
    owner: &Keypair,
    mint: Pubkey,
    token: Pubkey,
    edition: Pubkey,
    collection_metadata: Option<Pubkey>,
) -> Result<(), BanksClientError> {
    let token_program_id = get_account(context, &mint).await.owner;
    let tx = Transaction::new_signed_with_payer(
        &[instruction::burn_nft(
            mpl_token_metadata::ID,
            metadata,
            owner.pubkey(),
            mint,
            token,
            edition,
            token_program_id,
            collection_metadata,
        )],
        Some(&owner.pubkey()),
        &[owner],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn burn_edition(
    context: &mut ProgramTestContext,
    metadata: Pubkey,
    owner: &Keypair,
    print_edition_mint: Pubkey,
    master_edition_mint: Pubkey,
    print_edition_token: Pubkey,
    master_edition_token: Pubkey,
    master_edition: Pubkey,
    print_edition: Pubkey,
    edition_marker: Pubkey,
) -> Result<(), BanksClientError> {
    let token_program_id = get_account(context, &print_edition_mint).await.owner;
    let tx = Transaction::new_signed_with_payer(
        &[instruction::burn_edition_nft(
            mpl_token_metadata::ID,
            metadata,
            owner.pubkey(),
            print_edition_mint,
            master_edition_mint,
            print_edition_token,
            master_edition_token,
            master_edition,
            print_edition,
            edition_marker,
            token_program_id,
        )],
        Some(&owner.pubkey()),
        &[owner],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
    owner: &Pubkey,
    additional_signer: Option<&Keypair>,
) -> Result<(), BanksClientError> {
    let token_program_id = get_account(context, mint).await.owner;
    let mut signing_keypairs = vec![&context.payer];
    if let Some(signer) = additional_signer {
        signing_keypairs.push(signer);
    }

    let tx = Transaction::new_signed_with_payer(
        &[spl_token_2022::instruction::mint_to(
            &token_program_id,
            mint,
            account,
            owner,
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
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
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();
    let token_program_id = get_account(context, mint).await.owner;

    let space = if token_program_id == spl_token_2022::id() {
        ExtensionType::get_account_len::<spl_token_2022::state::Account>(&[
            ExtensionType::ImmutableOwner,
        ])
    } else {
        spl_token_2022::state::Account::get_packed_len()
    };

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(space),
                space as u64,
                &token_program_id,
            ),
            // no-ops in `Tokenkeg...`, so safe to do either way
            spl_token_2022::instruction::initialize_immutable_owner(
                &token_program_id,
                &account.pubkey(),
            )
            .unwrap(),
            spl_token_2022::instruction::initialize_account(
                &token_program_id,
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

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    manager: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    token_program_id: &Pubkey,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    // add mint close authority to token-2022 mints
    let space = if *token_program_id == spl_token_2022::id() {
        ExtensionType::get_account_len::<Mint>(&[ExtensionType::MintCloseAuthority])
    } else {
        Mint::get_packed_len()
    };

    let mut instructions = vec![system_instruction::create_account(
        &context.payer.pubkey(),
        &mint.pubkey(),
        rent.minimum_balance(space),
        space as u64,
        token_program_id,
    )];
    if *token_program_id == spl_token_2022::id() {
        instructions.push(
            spl_token_2022::instruction::initialize_mint_close_authority(
                token_program_id,
                &mint.pubkey(),
                freeze_authority,
            )
            .unwrap(),
        );
    }
    instructions.push(
        spl_token_2022::instruction::initialize_mint(
            token_program_id,
            &mint.pubkey(),
            manager,
            freeze_authority,
            decimals,
        )
        .unwrap(),
    );

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
