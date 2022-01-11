use {
    private_metadata::{
        encryption::elgamal::{
            ElGamalCiphertext,
            ElGamalKeypair,
            CipherKey,
        },
        instruction::get_private_metadata_address,
        pod::PodAccountInfo,
        state::PrivateMetadataAccount,
    },
    rand_core::OsRng,
    solana_program_test::*,
    solana_sdk::{
        program_pack::Pack,
        pubkey::Pubkey,
        signer::keypair::Keypair,
        signature::Signer,
        system_instruction,
        transaction::Transaction,
    },
    std::convert::TryInto,
};

async fn nft_setup_transaction(
    payer: &dyn Signer,
    mint: &dyn Signer,
    recent_blockhash: &solana_sdk::hash::Hash,
    rent: &solana_sdk::sysvar::rent::Rent,
    elgamal_kp: &ElGamalKeypair,
    cipher_key: &CipherKey,
) -> Result<Transaction, Box<dyn std::error::Error>> {
    let (public_metadata_key, _public_metadata_bump) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.pubkey().as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let (public_edition_key, _public_edition_bump) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.pubkey().as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    Ok(Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &payer.pubkey(), // mint auth
                None, // freeze auth
                0,
            )?,
            spl_associated_token_account::create_associated_token_account(
                &payer.pubkey(), // funding
                &payer.pubkey(), // wallet to create for
                &mint.pubkey(),
            ),
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint.pubkey(),
                &spl_associated_token_account::get_associated_token_address(
                    &payer.pubkey(),
                    &mint.pubkey(),
                ),
                &payer.pubkey(),
                &[],
                1
            )?,
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
                true, // update_auth_is_signer
                true, // is_mutable
            ),
            mpl_token_metadata::instruction::create_master_edition(
                mpl_token_metadata::id(),
                public_edition_key,
                mint.pubkey(),
                payer.pubkey(), // update auth
                payer.pubkey(), // mint auth
                public_metadata_key,
                payer.pubkey(), // payer
                None, // limited edition supply
            ),
            private_metadata::instruction::configure_metadata(
                payer.pubkey(),
                mint.pubkey(),
                elgamal_kp.public.into(),
                &elgamal_kp.public.encrypt(*cipher_key).into(),
                &[],
            ),
        ],
        Some(&payer.pubkey()),
        &[payer, mint],
        *recent_blockhash,
    ))
}

#[tokio::test]
async fn test_transfer() {
    let mut pc = ProgramTest::default();

    pc.prefer_bpf(true);

    pc.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    pc.add_program("private_metadata", private_metadata::id(), None);

    // 100x lol
    pc.set_compute_max_units(20_000_000);

    let (mut banks_client, payer, recent_blockhash) = pc.start().await;

    let rent = banks_client.get_rent().await;
    let rent = rent.unwrap();

    let mint = Keypair::from_base58_string("47WBGggARowPAzDVdCMCGxTVhNBqXhxgyDcFFyGrVx3VqUyPU7UZTz9umQifQA8yXxKNX8sKGujtDKu7kKX1rLB8");

    let elgamal_kp = ElGamalKeypair::new(&payer, &mint.pubkey()).unwrap();
    let cipher_key = CipherKey::random(&mut OsRng);

    println!("mint {:?}", mint);

    // smoke test
    assert_eq!(
        elgamal_kp.public.encrypt(cipher_key).decrypt(&elgamal_kp.secret),
        Ok(cipher_key),
    );

    let nft_setup = nft_setup_transaction(
        &payer,
        &mint,
        &recent_blockhash,
        &rent,
        &elgamal_kp,
        &cipher_key,
    ).await.unwrap();

    banks_client.process_transaction(nft_setup).await.unwrap();

    // data landed...
    let private_metadata_account = banks_client.get_account(
        get_private_metadata_address(&mint.pubkey()).0).await.unwrap().unwrap();
    let private_metadata = PrivateMetadataAccount::from_bytes(
        private_metadata_account.data.as_slice()).unwrap();
    assert_eq!(
        private_metadata.encrypted_cipher_key.try_into().and_then(
            |ct: ElGamalCiphertext| ct.decrypt(&elgamal_kp.secret)),
        Ok(cipher_key),
    );

    let dest = Keypair::new();
    let dest_elgamal_kp = ElGamalKeypair::new(&dest, &mint.pubkey()).unwrap();
    let transfer_buffer_key = private_metadata::instruction::get_transfer_buffer_address(
        &dest.pubkey(), &mint.pubkey()).0;

    let transfer = Transaction::new_signed_with_payer(
        &[
            // seed destination and publish elgamal pk to encrypt with
            system_instruction::transfer(
                &payer.pubkey(),
                &dest.pubkey(),
                solana_sdk::native_token::LAMPORTS_PER_SOL,
            ),
            private_metadata::instruction::publish_elgamal_pubkey(
                &dest.pubkey(),
                &mint.pubkey(),
                dest_elgamal_kp.public.into(),
            ),
            private_metadata::instruction::publish_elgamal_pubkey(
                &payer.pubkey(),
                &mint.pubkey(),
                elgamal_kp.public.into(),
            ),

            // do the transfer prep
            private_metadata::instruction::init_transfer(
                &payer.pubkey(),
                &mint.pubkey(),
                &dest.pubkey(),
            ),
            private_metadata::instruction::transfer_chunk(
                payer.pubkey(),
                mint.pubkey(),
                transfer_buffer_key,
                private_metadata::instruction::TransferChunkData {
                    transfer: private_metadata::transfer_proof::TransferData::new(
                        &elgamal_kp,
                        dest_elgamal_kp.public.into(),
                        cipher_key,
                        private_metadata.encrypted_cipher_key.try_into().unwrap(),
                    ),
                },
            ),

            // finish and transfer
            private_metadata::instruction::fini_transfer(
                payer.pubkey(),
                mint.pubkey(),
                transfer_buffer_key,
            ),
            spl_associated_token_account::create_associated_token_account(
                &payer.pubkey(), // funding
                &dest.pubkey(), // wallet to create for
                &mint.pubkey(),
            ),
            spl_token::instruction::transfer(
                &spl_token::id(),
                &spl_associated_token_account::get_associated_token_address(
                    &payer.pubkey(),
                    &mint.pubkey(),
                ),
                &spl_associated_token_account::get_associated_token_address(
                    &dest.pubkey(),
                    &mint.pubkey(),
                ),
                &payer.pubkey(),
                &[],
                1,
            ).unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &dest],
        recent_blockhash,
    );

    banks_client.process_transaction(transfer).await.unwrap();

    // transfer landed...
    let private_metadata_account = banks_client.get_account(
        get_private_metadata_address(&mint.pubkey()).0).await.unwrap().unwrap();
    let private_metadata = PrivateMetadataAccount::from_bytes(
        private_metadata_account.data.as_slice()).unwrap();
    // successfully decrypt with dest_elgamal_kp
    assert_eq!(
        private_metadata.encrypted_cipher_key.try_into().and_then(
            |ct: ElGamalCiphertext| ct.decrypt(&dest_elgamal_kp.secret)),
        Ok(cipher_key),
    );
    // and old fails to decrypt...
    assert!(
        private_metadata.encrypted_cipher_key.try_into().and_then(
            |ct: ElGamalCiphertext| ct.decrypt(&elgamal_kp.secret)).is_err(),
    );
}
