use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Signer,
    signer::keypair::Keypair,
    system_instruction,
    system_program,
    transaction::Transaction,
};
use std::convert::TryFrom;

fn merkle_layer(
    layer: &[solana_program::keccak::Hash],
) -> Vec<solana_program::keccak::Hash> {
    let mut next_layer = Vec::new();
    for i in 0..layer.len() {
        if i % 2 == 0 {
            if i + 1 < layer.len() {
                let lhs = &layer[i].0;
                let rhs = &layer[i+1].0;
                if lhs <= rhs {
                    next_layer.push(solana_program::keccak::hashv(&[&[0x01], lhs, rhs]));
                } else {
                    next_layer.push(solana_program::keccak::hashv(&[&[0x01], rhs, lhs]));
                }
            } else {
                next_layer.push(layer[i]);
            }
        }
    }
    next_layer
}

fn merkle_root(
    leafs: &[solana_program::keccak::Hash],
) -> solana_program::keccak::Hash {
    let mut layer = leafs.to_vec();
    loop {
        if layer.len() <= 1 { return layer[0]; }
        layer = merkle_layer(&layer);
    }
}

// TODO: dedup
fn merkle_proof(
    leafs: &[solana_program::keccak::Hash],
    mut index: usize,
) -> Vec<solana_program::keccak::Hash> {
    let mut proof = Vec::new();
    let mut layer = leafs.to_vec();
    loop {
        let sibling = index ^ 1;
        if sibling < layer.len() {
            proof.push(layer[sibling]);
        }
        index = index / 2;
        if layer.len() <= 1 { return proof; }
        layer = merkle_layer(&layer);
    }
}

#[tokio::test]
async fn test_token_freeze() {

    let mut pc = ProgramTest::default();

    pc.add_program("mpl_gumdrop", mpl_gumdrop::id(), processor!(mpl_gumdrop::entry));

    let (mut banks_client, payer, recent_blockhash) = pc.start().await;

    let rent = banks_client.get_rent().await;
    let rent = rent.unwrap();

    let multisig = Keypair::new();
    let mint = Keypair::new();

    let gumdrop_recipients = [
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
    ];

    let leafs = gumdrop_recipients
        .iter()
        .enumerate()
        .map(|(index, kp)|
            solana_program::keccak::hashv(&[
                &[0x02],
                &u64::try_from(index).unwrap().to_le_bytes(),
                kp.pubkey().as_ref(),
                mint.pubkey().as_ref(),
                &1u64.to_le_bytes(),
            ])
        )
        .collect::<Vec<_>>();

    let distributor_base = Keypair::new();
    let (distributor_key, distributor_bump) = Pubkey::find_program_address(
        &[
            b"MerkleDistributor".as_ref(),
            distributor_base.pubkey().as_ref(),
        ],
        &mpl_gumdrop::id(),
    );

    // initialize the whitelist token where we want to freeze tokens after minting
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &multisig.pubkey(),
                    rent.minimum_balance(spl_token::state::Multisig::LEN),
                    spl_token::state::Multisig::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_multisig(
                    &spl_token::id(),
                    &multisig.pubkey(),
                    &[
                        &payer.pubkey(),
                        &distributor_key, // allow gumdrop to freeze/thaw
                    ],
                    1, // only 1 signer required
                ).unwrap(),
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
                    Some(&multisig.pubkey()), // freeze auth
                    0,
                ).unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer, &mint, &multisig],
            recent_blockhash,
        )
    ).await.unwrap();


    let distributor_ata = spl_associated_token_account::get_associated_token_address(
        &distributor_key,
        &mint.pubkey(),
    );

    // create the gumdrop
    let root = merkle_root(&leafs).0;
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                Instruction {
                    accounts: mpl_gumdrop::accounts::NewDistributor {
                        base: distributor_base.pubkey(),
                        distributor: distributor_key,
                        payer: payer.pubkey(),
                        system_program: system_program::id(),
                    }.to_account_metas(None),
                    data: mpl_gumdrop::instruction::NewDistributor {
                        bump: distributor_bump,
                        root,
                        temporal: Pubkey::default(),
                    }.data(),
                    program_id: mpl_gumdrop::id(),
                },
                spl_associated_token_account::create_associated_token_account(
                    &payer.pubkey(), // funding
                    &distributor_key, // wallet to create for
                    &mint.pubkey(),
                ),
                spl_token::instruction::mint_to(
                    &spl_token::id(),
                    &mint.pubkey(),
                    &distributor_ata,
                    &payer.pubkey(), // mint auth
                    &[],
                    4,
                ).unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer, &distributor_base],
            recent_blockhash,
        )
    ).await.unwrap();


    let mut tested_pre_frozen = false;
    for (index, recipient) in gumdrop_recipients.iter().enumerate() {
        let index_u64 = u64::try_from(index).unwrap();
        // seed some lamports
        banks_client.process_transaction(
            Transaction::new_signed_with_payer(
                &[
                    system_instruction::transfer(
                        &payer.pubkey(),
                        &recipient.pubkey(),
                        LAMPORTS_PER_SOL,
                    ),
                    spl_associated_token_account::create_associated_token_account(
                        &payer.pubkey(), // funding
                        &recipient.pubkey(), // wallet to create for
                        &mint.pubkey(),
                    ),
                ],
                Some(&payer.pubkey()),
                &[&payer],
                recent_blockhash,
            )
        ).await.unwrap();

        let (claim_status, claim_bump) = Pubkey::find_program_address(
            &[
                mpl_gumdrop::CLAIM_STATUS.as_ref(),
                index_u64.to_le_bytes().as_ref(),
                distributor_key.as_ref(),
            ],
            &mpl_gumdrop::id(),
        );

        let recipient_ata = spl_associated_token_account::get_associated_token_address(
            &recipient.pubkey(),
            &mint.pubkey(),
        );

        if index > 2 {
            tested_pre_frozen = true;
            banks_client.process_transaction(
                Transaction::new_signed_with_payer(
                    &[
                        spl_token::instruction::freeze_account(
                            &spl_token::id(),
                            &recipient_ata,
                            &mint.pubkey(),
                            &multisig.pubkey(),
                            &[&payer.pubkey()],
                        ).unwrap(),
                    ],
                    Some(&payer.pubkey()),
                    &[&payer],
                    recent_blockhash,
                )
            ).await.unwrap();
        }

        // claim OK
        let proof = merkle_proof(&leafs, index);
        let proof_raw = proof.iter().map(|v| v.0).collect::<Vec<_>>();
        assert!(mpl_gumdrop::merkle_proof::verify(proof_raw.clone(), root, leafs[index].0));
        banks_client.process_transaction(
            Transaction::new_signed_with_payer(
                &[
                    Instruction {
                        accounts: mpl_gumdrop::accounts::Claim {
                            distributor: distributor_key,
                            claim_status,
                            from: distributor_ata,
                            to: recipient_ata,
                            temporal: recipient.pubkey(),
                            payer: recipient.pubkey(),
                            system_program: system_program::id(),
                            token_program: spl_token::id(),
                        }.to_account_metas(None).into_iter().chain(
                            [
                                AccountMeta::new_readonly(multisig.pubkey(), false),
                                AccountMeta::new_readonly(mint.pubkey(), false),
                            ]
                        ).collect(),
                        data: mpl_gumdrop::instruction::Claim {
                            claim_bump,
                            index: index_u64,
                            amount: 1u64,
                            claimant_secret: recipient.pubkey(),
                            proof: proof_raw,
                        }.data(),
                        program_id: mpl_gumdrop::id(),
                    },
                ],
                Some(&recipient.pubkey()),
                &[recipient],
                recent_blockhash,
            )
        ).await.unwrap();

        // account is frozen
        let recipient_ata = banks_client.get_account(recipient_ata).await.unwrap().unwrap();
        let recipient_ata = spl_token::state::Account::unpack_unchecked(
            &recipient_ata.data).unwrap();

        assert_eq!(recipient_ata.amount, 1);
        assert!(recipient_ata.is_frozen());
    }
    assert!(tested_pre_frozen);
}
