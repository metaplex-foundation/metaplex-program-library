use {
    solana_program::pubkey::Pubkey,
    solana_program_test::*,
    solana_sdk::{
        signer::keypair::Keypair,
        signature::Signer,
        system_instruction,
        transaction::Transaction,
    },
    private_metadata::{
        instruction,
    },
};

#[tokio::test]
async fn test_transfer() {
    let mut pc = ProgramTest::default();

    pc.prefer_bpf(true);

    pc.add_program("private_metadata", private_metadata::id(), None);
    pc.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    pc.add_program("spl_token", spl_token::id(), None);

    // pc.set_bpf_compute_max_units(350_000);

    let (mut banks_client, payer, recent_blockhash) = pc.start().await;

    let rent = banks_client.get_rent().await;
    let rent = rent.unwrap();

    let mint = Keypair::new();
    let (public_metadata_key, _public_metadat_bump) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.pubkey().as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let token_account = Keypair::new(); // not ATA...

    let mut transaction = Transaction::new_with_payer(
        &[
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &payer.pubkey(), // mint auth
                None, // freeze auth
                0,
            ).unwrap(),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &token_account.pubkey(),
                &mint.pubkey(),
                &payer.pubkey(),
            ).unwrap(),
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint.pubkey(),
                &token_account.pubkey(),
                &payer.pubkey(),
                &[],
                1
            ).unwrap(),
            mpl_token_metadata::instruction::create_metadata_accounts(
                mpl_token_metadata::id(),
                public_metadata_key,
                mint.pubkey(),
                payer.pubkey(), // mint auth
                payer.pubkey(), // payer
                payer.pubkey(), // update auth
                "test".to_string(), // name
                "".to_string(), // symbol
                "".to_string(), // uri
                Some(vec![mpl_token_metadata::state::Creator{
                    address: payer.pubkey(),
                    verified: true,
                    share: 100,
                }]),
                0, // seller_fee_basis_points
                true,
                false,
            ),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
