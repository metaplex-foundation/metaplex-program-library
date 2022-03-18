use {
    anchor_lang::{
        InstructionData,
        ToAccountMetas,
    },
    stealth::{
        encryption::elgamal::{
            ElGamalCiphertext,
            ElGamalKeypair,
            CipherKey,
        },
        instruction::get_stealth_address,
        pod::PodAccountInfo,
        state::StealthAccount,
    },
    rand_core::OsRng,
    solana_program_test::*,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        program_pack::Pack,
        pubkey::Pubkey,
        signer::keypair::Keypair,
        signature::Signer,
        system_instruction,
        system_program,
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
) -> Result<(Transaction, Pubkey), Box<dyn std::error::Error>> {
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

    let payer_pubkey = payer.pubkey();
    let instructions = vec![
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
                Some(&payer_pubkey), // freeze auth
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
            spl_token::instruction::approve(
                &spl_token::id(),
                &spl_associated_token_account::get_associated_token_address(
                    &payer.pubkey(),
                    &mint.pubkey(),
                ),
                &get_stealth_address(&mint.pubkey()).0, // delegate
                &payer.pubkey(), // owner
                &[],
                1,
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
            stealth::instruction::configure_metadata(
                payer.pubkey(),
                mint.pubkey(),
                elgamal_kp.public.into(),
                &elgamal_kp.public.encrypt(*cipher_key).into(),
                &[],
            ),
        ];

    Ok((Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, mint],
        *recent_blockhash,
    ), public_metadata_key))
}


pub struct TestEnv {
    mint: Pubkey,
    escrow_key: Pubkey,
    public_metadata_key: Pubkey,
    cipher_key: CipherKey,
    // we need the randomness used in the opening...
    encrypted_cipher_key: stealth::zk_token_elgamal::pod::ElGamalCiphertext,
    seller_elgamal_kp: ElGamalKeypair,
    buyer_elgamal_kp: ElGamalKeypair,
}

async fn setup_test(
    banks_client: &mut BanksClient,
    seller: &Keypair,
    buyer: &Keypair,
    last_blockhash: &solana_sdk::hash::Hash,
) -> Result<TestEnv, Box<dyn std::error::Error>> {
    let rent = banks_client.get_rent().await;
    let rent = rent?;

    let mint = Keypair::from_base58_string("47WBGggARowPAzDVdCMCGxTVhNBqXhxgyDcFFyGrVx3VqUyPU7UZTz9umQifQA8yXxKNX8sKGujtDKu7kKX1rLB8");

    let elgamal_kp = ElGamalKeypair::new(seller, &mint.pubkey())?;
    let cipher_key = CipherKey::random(&mut OsRng);

    println!("mint {:?}", mint);

    // smoke test
    assert_eq!(
        elgamal_kp.public.encrypt(cipher_key).decrypt(&elgamal_kp.secret),
        Ok(cipher_key),
    );

    let (nft_setup, public_metadata_key) = nft_setup_transaction(
        seller,
        &mint,
        last_blockhash,
        &rent,
        &elgamal_kp,
        &cipher_key,
    ).await?;

    banks_client.process_transaction(nft_setup).await?;

    // data landed...
    let stealth_key = get_stealth_address(&mint.pubkey()).0;
    let stealth_account = banks_client.get_account(stealth_key).await?
        .ok_or("Failed to fetch stealth account")?;
    let stealth = StealthAccount::from_bytes(stealth_account.data.as_slice())
        .ok_or("Failed to decode stealth account")?;
    assert_eq!(
        stealth.encrypted_cipher_key.try_into().and_then(
            |ct: ElGamalCiphertext| ct.decrypt(&elgamal_kp.secret)),
        Ok(cipher_key),
    );

    // seed buyer and initialize mint
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                system_instruction::transfer(
                    &seller.pubkey(),
                    &buyer.pubkey(),
                    11 * LAMPORTS_PER_SOL,
                ),
            ],
            Some(&seller.pubkey()),
            &[seller],
            *last_blockhash,
        ),
    ).await?;

    let (escrow_key, _escrow_bump) = Pubkey::find_program_address(
        &[
            b"BidEscrow",
            buyer.pubkey().as_ref(),
            mint.pubkey().as_ref(),
        ],
        &stealth_escrow::ID,
    );

    let buyer_elgamal_kp = ElGamalKeypair::new(buyer, &mint.pubkey())?;

    // buyer makes a bid of 10 SOL
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                system_instruction::transfer(
                    &buyer.pubkey(),
                    &escrow_key,
                    10 * LAMPORTS_PER_SOL,
                ),
                Instruction {
                    program_id: stealth_escrow::id(),
                    data: stealth_escrow::instruction::InitEscrow {
                        collateral: LAMPORTS_PER_SOL,
                        slots: 1000,
                    }.data(),
                    accounts: stealth_escrow::accounts::InitEscrow {
                        bidder: buyer.pubkey(),
                        mint: mint.pubkey(),
                        escrow: escrow_key,
                        system_program: system_program::id(),
                    }.to_account_metas(None),
                },
                stealth::instruction::publish_elgamal_pubkey(
                    &buyer.pubkey(),
                    &mint.pubkey(),
                    buyer_elgamal_kp.public.into(),
                ),
            ],
            Some(&buyer.pubkey()),
            &[buyer],
            *last_blockhash,
        ),
    ).await?;

    Ok(TestEnv {
        mint: mint.pubkey(),
        escrow_key,
        cipher_key,
        encrypted_cipher_key: stealth.encrypted_cipher_key,
        seller_elgamal_kp: elgamal_kp,
        buyer_elgamal_kp,
        public_metadata_key,
    })
}

#[tokio::test]
async fn test_successful_escrow() {
    let mut pc = ProgramTest::default();

    pc.prefer_bpf(true);

    pc.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    pc.add_program("stealth", stealth::id(), None);
    pc.add_program("stealth_escrow", stealth_escrow::id(), None);

    pc.set_compute_max_units(20_000_000);

    let (mut banks_client, payer, recent_blockhash) = pc.start().await;

    let buyer = Keypair::new();
    let seller = &payer;
    let TestEnv {
        mint,
        escrow_key,
        public_metadata_key,
        cipher_key,
        encrypted_cipher_key,
        seller_elgamal_kp,
        buyer_elgamal_kp
    } = setup_test(
        &mut banks_client,
        seller,
        &buyer,
        &recent_blockhash,
    ).await.unwrap();

    // seller accepts
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                stealth_escrow::accept_escrow(
                    buyer.pubkey(),
                    mint,
                    escrow_key,
                    seller.pubkey(),
                ),
            ],
            Some(&seller.pubkey()),
            &[seller],
            recent_blockhash,
        ),
    ).await.unwrap();

    let transfer_buffer_key = stealth::instruction::get_transfer_buffer_address(
        &buyer.pubkey(), &mint).0;

    // crank over 'many' transactions
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                stealth::instruction::transfer_chunk(
                    seller.pubkey(),
                    mint,
                    transfer_buffer_key,
                    stealth::instruction::TransferChunkData {
                        transfer: stealth::transfer_proof::TransferData::new(
                            &seller_elgamal_kp,
                            buyer_elgamal_kp.public.into(),
                            cipher_key,
                            encrypted_cipher_key.try_into().unwrap(),
                        ),
                    },
                ),
            ],
            Some(&seller.pubkey()),
            &[seller],
            recent_blockhash,
        ),
    ).await.unwrap();

    // and complete escrow
    let stealth_key = get_stealth_address(&mint).0;
    let mut complete_escrow_accounts = stealth_escrow::accounts::CompleteEscrow {
        bidder: buyer.pubkey(),
        mint,
        escrow: escrow_key,
        bidder_token_account:
            spl_associated_token_account::get_associated_token_address(
                &buyer.pubkey(), &mint),
        acceptor: seller.pubkey(),
        escrow_token_account:
            spl_associated_token_account::get_associated_token_address(
                &escrow_key, &mint),
        stealth: stealth_key,
        transfer_buffer: transfer_buffer_key,
        metadata: public_metadata_key,
        system_program: system_program::id(),
        token_program: spl_token::id(),
        stealth_program: stealth::id(),
        rent: solana_sdk::sysvar::rent::id(),
    }.to_account_metas(None);
    complete_escrow_accounts.push(
        AccountMeta::new_readonly(seller.pubkey(), false),
    );
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                // TODO: do conditionally in complete_escrow?
                spl_associated_token_account::create_associated_token_account(
                    &seller.pubkey(), // funding
                    &buyer.pubkey(), // wallet to create for
                    &mint,
                ),
                Instruction {
                    program_id: stealth_escrow::id(),
                    data: stealth_escrow::instruction::CompleteEscrow {}.data(),
                    accounts: complete_escrow_accounts,
                },
            ],
            Some(&seller.pubkey()),
            &[seller],
            recent_blockhash,
        ),
    ).await.unwrap();

    // transfer landed...
    let stealth_account = banks_client.get_account(
        stealth_key).await.unwrap().unwrap();
    let stealth = StealthAccount::from_bytes(
        stealth_account.data.as_slice()).unwrap();
    // successfully decrypt with buyer_elgamal_kp
    assert_eq!(
        stealth.encrypted_cipher_key.try_into().and_then(
            |ct: ElGamalCiphertext| ct.decrypt(&buyer_elgamal_kp.secret)),
        Ok(cipher_key),
    );
}

#[tokio::test]
async fn test_close_before_accept() {
    let mut pc = ProgramTest::default();

    pc.prefer_bpf(true);

    pc.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    pc.add_program("stealth", stealth::id(), None);
    pc.add_program("stealth_escrow", stealth_escrow::id(), None);

    pc.set_compute_max_units(20_000_000);

    let (mut banks_client, payer, recent_blockhash) = pc.start().await;

    let buyer = Keypair::new();
    let seller = &payer;
    let TestEnv { mint, escrow_key, .. } = setup_test(
        &mut banks_client,
        seller,
        &buyer,
        &recent_blockhash,
    ).await.unwrap();

    let pre_close_lamports = banks_client.get_balance(
        buyer.pubkey()).await.unwrap();

    assert!(pre_close_lamports < LAMPORTS_PER_SOL);

    // closes
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                Instruction {
                    program_id: stealth_escrow::id(),
                    data: stealth_escrow::instruction::CloseEscrow {}.data(),
                    accounts: stealth_escrow::accounts::CloseEscrow {
                        bidder: buyer.pubkey(),
                        mint,
                        escrow: escrow_key,
                        system_program: system_program::id(),
                    }.to_account_metas(None),
                },
            ],
            Some(&buyer.pubkey()),
            &[&buyer],
            recent_blockhash,
        ),
    ).await.unwrap();

    let post_close_lamports = banks_client.get_balance(
        buyer.pubkey()).await.unwrap();

    assert!(post_close_lamports > 10 * LAMPORTS_PER_SOL);
}

#[tokio::test]
async fn test_close_after_accept() {
    let mut pc = ProgramTest::default();

    pc.prefer_bpf(true);

    pc.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    pc.add_program("stealth", stealth::id(), None);
    pc.add_program("stealth_escrow", stealth_escrow::id(), None);

    pc.set_compute_max_units(20_000_000);

    let mut ptc = pc.start_with_context().await;

    let buyer = Keypair::new();
    let seller = &ptc.payer;
    let TestEnv { mint, escrow_key, .. } = setup_test(
        &mut ptc.banks_client,
        seller,
        &buyer,
        &ptc.last_blockhash,
    ).await.unwrap();

    let pre_close_lamports = ptc.banks_client.get_balance(
        buyer.pubkey()).await.unwrap();

    assert!(pre_close_lamports < LAMPORTS_PER_SOL);

    // seller accepts
    ptc.banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                stealth_escrow::accept_escrow(
                    buyer.pubkey(),
                    mint,
                    escrow_key,
                    seller.pubkey(),
                ),
            ],
            Some(&seller.pubkey()),
            &[seller],
            ptc.last_blockhash,
        ),
    ).await.unwrap();

    let mut close_escrow_accounts = stealth_escrow::accounts::CloseEscrow {
        bidder: buyer.pubkey(),
        mint,
        escrow: escrow_key,
        system_program: system_program::id(),
    }.to_account_metas(None);

    close_escrow_accounts.extend_from_slice(
        &[
            AccountMeta::new(
                spl_associated_token_account::get_associated_token_address(
                    &escrow_key, &mint),
                false,
            ),
            AccountMeta::new(
                spl_associated_token_account::get_associated_token_address(
                    &seller.pubkey(), &mint),
                false,
            ),
            AccountMeta::new_readonly(spl_token::id(), false),
        ]
    );

    // close attempt should fail
    let immediate_close_result = ptc.banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                Instruction {
                    program_id: stealth_escrow::id(),
                    data: stealth_escrow::instruction::CloseEscrow {}.data(),
                    accounts: close_escrow_accounts.clone(),
                },
            ],
            Some(&buyer.pubkey()),
            &[&buyer],
            ptc.last_blockhash,
        ),
    ).await;

    assert!(immediate_close_result.is_err());


    // seller didn't complete within time
    let current_slot = ptc.banks_client.get_root_slot().await.unwrap();
    ptc.warp_to_slot(current_slot + 1000).unwrap();

    // buyer is able to close now
    ptc.banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                Instruction {
                    program_id: stealth_escrow::id(),
                    data: stealth_escrow::instruction::CloseEscrow {}.data(),
                    accounts: close_escrow_accounts,
                },
            ],
            Some(&buyer.pubkey()),
            &[&buyer],
            ptc.last_blockhash,
        ),
    ).await.unwrap();

    let post_close_lamports = ptc.banks_client.get_balance(
        buyer.pubkey()).await.unwrap();

    // buyer also got seller collateral
    assert!(post_close_lamports > 11 * LAMPORTS_PER_SOL);
}
