#![cfg(feature = "test-bpf")]

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
    sysvar,
    transaction::Transaction,
};

#[tokio::test]
async fn test_frozen_whitelist_ata() {

    let mut pc = ProgramTest::default();

    // NB: if we pass the process_instruction directly, the CPIs get compiled as native arch
    // (not-bpf) and so go to a syscall-stub that doesn't support account size changes (as of
    // 1.9.9)
    pc.add_program("mpl_candy_machine", mpl_candy_machine::id(), None);
    pc.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);

    let (mut banks_client, payer, recent_blockhash) = pc.start().await;

    let rent = banks_client.get_rent().await;
    let rent = rent.unwrap();

    let multisig = Keypair::new();
    let mint = Keypair::new();

    let candy_machine_key = Keypair::new();
    let (creator_key, creator_bump) = Pubkey::find_program_address(
        &[
            mpl_candy_machine::PREFIX.as_ref(),
            candy_machine_key.pubkey().as_ref(),
        ],
        &mpl_candy_machine::id(),
    );

    // initialize the whitelist token for the candy machine
    println!("Initializing multisig with {}", creator_key);
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
                        &creator_key, // allow candy_machine to freeze/thaw
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


    // create the candy machine
    let candy_machine_data = mpl_candy_machine::CandyMachineData {
        uuid: "foobar".to_string(),
        price: 0,
        symbol: "".to_string(),
        seller_fee_basis_points: 0,
        max_supply: 0,
        is_mutable: true,
        retain_authority: true,
        go_live_date: None,
        end_settings: None,
        creators: vec![
            mpl_candy_machine::Creator {
                address: payer.pubkey(),
                verified: false,
                share: 100,
            },
        ],
        hidden_settings: None,
        whitelist_mint_settings: Some(mpl_candy_machine::WhitelistMintSettings {
            mode: mpl_candy_machine::WhitelistMintMode::BurnEveryTime,
            mint: mint.pubkey(),
            presale: true,
            discount_price: None,
        }),
        items_available: 2,
        gatekeeper: None,
    };
    let candy_machine_len = mpl_candy_machine::get_space_for_candy(candy_machine_data.clone()).unwrap();
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &candy_machine_key.pubkey(),
                    rent.minimum_balance(candy_machine_len),
                    candy_machine_len as u64,
                    &mpl_candy_machine::id(),
                ),
                Instruction {
                    accounts: mpl_candy_machine::accounts::InitializeCandyMachine {
                        candy_machine: candy_machine_key.pubkey(),
                        wallet: payer.pubkey(),
                        authority: payer.pubkey(),
                        payer: payer.pubkey(),
                        system_program: system_program::id(),
                        rent: sysvar::rent::id(),
                    }.to_account_metas(None),
                    data: mpl_candy_machine::instruction::InitializeCandyMachine {
                        data: candy_machine_data,
                    }.data(),
                    program_id: mpl_candy_machine::id(),
                },
                Instruction {
                    accounts: mpl_candy_machine::accounts::AddConfigLines {
                        candy_machine: candy_machine_key.pubkey(),
                        authority: payer.pubkey(),
                    }.to_account_metas(None),
                    data: mpl_candy_machine::instruction::AddConfigLines {
                        index: 0,
                        config_lines: vec![
                            mpl_candy_machine::ConfigLine {
                                name: "test 1".to_string(),
                                uri: "https://solana.com/".to_string(),
                            },
                            mpl_candy_machine::ConfigLine {
                                name: "test 2".to_string(),
                                uri: "https://solana.com/".to_string(),
                            },
                        ],
                    }.data(),
                    program_id: mpl_candy_machine::id(),
                },
            ],
            Some(&payer.pubkey()),
            &[&payer, &candy_machine_key],
            recent_blockhash,
        )
    ).await.unwrap();


    // send some whitelist tokens and SOL
    let recipient = Keypair::new();
    let recipient_ata = spl_associated_token_account::get_associated_token_address(
        &recipient.pubkey(),
        &mint.pubkey(),
    );
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
                spl_token::instruction::mint_to(
                    &spl_token::id(),
                    &mint.pubkey(),
                    &recipient_ata,
                    &payer.pubkey(), // mint auth
                    &[],
                    1,
                ).unwrap(),
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

    {
        let recipient_ata = banks_client.get_account(recipient_ata).await.unwrap().unwrap();
        let recipient_ata = spl_token::state::Account::unpack_unchecked(
            &recipient_ata.data).unwrap();

        assert_eq!(recipient_ata.amount, 1);
        assert!(recipient_ata.is_frozen());
    }


    // mint one
    let nft_mint = Keypair::new();
    let nft_metadata = mpl_token_metadata::pda::find_metadata_account(&nft_mint.pubkey()).0;
    let nft_edition = mpl_token_metadata::pda::find_master_edition_account(&nft_mint.pubkey()).0;

    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[
                // candy machine setup
                system_instruction::create_account(
                    &recipient.pubkey(),
                    &nft_mint.pubkey(),
                    rent.minimum_balance(spl_token::state::Mint::LEN),
                    spl_token::state::Mint::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_mint(
                    &spl_token::id(),
                    &nft_mint.pubkey(),
                    &recipient.pubkey(), // mint auth
                    Some(&recipient.pubkey()), // freeze auth
                    0,
                ).unwrap(),
                spl_associated_token_account::create_associated_token_account(
                    &recipient.pubkey(), // funding
                    &recipient.pubkey(), // wallet to create for
                    &nft_mint.pubkey(),
                ),
                spl_token::instruction::mint_to(
                    &spl_token::id(),
                    &nft_mint.pubkey(),
                    &spl_associated_token_account::get_associated_token_address(
                        &recipient.pubkey(),
                        &nft_mint.pubkey(),
                    ),
                    &recipient.pubkey(), // mint auth
                    &[],
                    1,
                ).unwrap(),

                // mint one
                Instruction {
                    accounts: mpl_candy_machine::accounts::MintNFT {
                        candy_machine: candy_machine_key.pubkey(),
                        candy_machine_creator: creator_key,
                        payer: recipient.pubkey(),
                        wallet: payer.pubkey(),
                        metadata: nft_metadata,
                        mint: nft_mint.pubkey(),
                        mint_authority: recipient.pubkey(),
                        update_authority: recipient.pubkey(),
                        master_edition: nft_edition,
                        token_metadata_program: mpl_token_metadata::id(),
                        token_program: spl_token::id(),
                        system_program: system_program::id(),
                        rent: sysvar::rent::id(),
                        clock: sysvar::clock::id(),
                        recent_blockhashes: sysvar::recent_blockhashes::id(),
                        instruction_sysvar_account: sysvar::instructions::id(),
                    }.to_account_metas(None).into_iter().chain(
                        [
                            // whitelist_token_account
                            AccountMeta::new(recipient_ata, false),
                            // whitelist_token_mint
                            AccountMeta::new(mint.pubkey(), false),
                            // whitelist_burn_authority. already a signer
                            AccountMeta::new_readonly(recipient.pubkey(), true),
                            // whitelist_freeze_authority
                            AccountMeta::new_readonly(multisig.pubkey(), false),
                        ]
                    ).collect(),
                    // wtf is this casing
                    data: mpl_candy_machine::instruction::MintNft {
                        creator_bump,
                    }.data(),
                    program_id: mpl_candy_machine::id(),
                },
            ],
            Some(&recipient.pubkey()),
            &[&recipient, &nft_mint],
            recent_blockhash,
        )
    ).await.unwrap();
}
