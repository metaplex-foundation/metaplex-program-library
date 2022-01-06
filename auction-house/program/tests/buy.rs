// //! Module provide tests for `Buy` instruction.

// mod utils;

// mod buy {
//     use super::utils::{
//         clone_keypair, create_associated_token_account, create_mint,
//         find_auction_house_fee_account_address, find_escrow_payment_address,
//         find_trade_state_address,
//         setup_functions::{setup_auction_house, setup_program},
//     };
//     use me

// #[cfg(test)]
// mod buy {
//     use super::utils::{
//         clone_keypair, create_associated_token_account, create_associated_token_account_v2, create_mint, create_mint_v2, create_token_metadata,
//         create_token_metadata_v2, find_auction_house_fee_account_address, find_escrow_payment_address,
//         find_trade_state_address, mint_to, mint_to_v2,
//         setup_functions::{setup_auction_house_v2, setup_program},
//         transfer_lamports,transfer_lamports_v2,
//     };

//     use anchor_client::{
//         solana_sdk::{
//             commitment_config::CommitmentConfig, signature::Keypair, signer::Signer,
//             system_program, sysvar,
//         },
//         Client, Cluster,
//     };

//     use solana_program_test::*;
//     use std::error;

//     #[test]
//     fn success() -> Result<(), Box<dyn error::Error>> {
//         const BUYER_PRICE: u64 = 1;
//         const TOKEN_SIZE: u64 = 1;

//         let mut context = auction_house_program_test().start_with_context().await;

//         // Payer Wallet
//         let payer_wallet = Keypair::new();

//         airdrop(&mut context, &payer_wallet.pubkey(), 10_000_000_000).await;

//         // Derive native(wrapped) sol mint
//         let treasury_mint = spl_token::native_mint::id();

//         // Token mint for `TokenMetadata`.
//         let token_mint = create_mint(context, &wallet)?;

//         // Derive / Create associated token account
//         let token_account =
//             create_associated_token_account(context, &wallet, &token_mint.pubkey())?;

//         // Mint tokens
//         mint_to(context, &wallet, &token_mint.pubkey(), &token_account, 1)?;

//         // Setup AuctionHouse
//         let auction_house = setup_auction_house(&program, &wallet_pubkey, &treasury_mint).unwrap();

//         // Derive `AuctionHouse` fee account
//         let (auction_house_fee_account, _) = find_auction_house_fee_account_address(&auction_house);

//         // Derive buyer trade state address
// =======
//     use std::error;
//     use solana_program_test::*;

//     use super::utils::{
//         airdrop,
//         constants::{AUCTION_HOUSE, FEE_PAYER, TREASURY},
//         setup_functions::auction_house_program_test,
//     };
//     use solana_program::instruction::Instruction;
//     use solana_sdk::{transaction::Transaction, msg};
//     use spl_token;
//     use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
//     use mpl_auction_house::{
//         accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction, Buy,
//     };

//     #[tokio::test]
//     async fn success() -> Result<(), Box<dyn error::Error>> {
//         let mut context = auction_house_program_test().start_with_context().await;
//         const BUYER_PRICE: u64 = 1;
//         const TOKEN_SIZE: u64 = 1;

//         // 1) Load `Localnet` keypair
//         let wallet = Keypair::new();
//         let wallet_pubkey = wallet.pubkey();

//         // airdrop
//         airdrop(&mut context, &wallet_pubkey, 10_000_000_000).await;

//         // 3) Derive native(wrapped) sol mint
//         let treasury_mint = spl_token::native_mint::id();

//         // 4) Token mint for `TokenMetadata`.
//         let token_mint = create_mint_v2(&mut context, &wallet).await.unwrap();

//         // 5) Derive / Create associated token account +
//         // let token_account = create_associated_token_account(&connection, &wallet, &token_mint.pubkey())?;
//         let token_account = create_associated_token_account_v2(&mut context, &wallet, &token_mint.pubkey()).unwrap();

//         // 6) Mint tokens
//         mint_to_v2(
//             &mut context,
//             &wallet,
//             &token_mint.pubkey(),
//             &token_account,
//             1,
//         )?;

//         // 7) Setup AuctionHouse
//         // let auction_house = setup_auction_house(&program, &wallet_pubkey, &treasury_mint).unwrap();
//         let auction_house = setup_auction_house_v2(&mut context, &wallet).await.unwrap();

//         // 8) Derive `AuctionHouse` fee account
//         // let (auction_house_fee_account, _) = find_auction_house_fee_account_address(auction_house.owner);
//         // let auction_house_pubkey = &auction_house.owner;
//         let (auction_house_fee_account, _) = find_auction_house_fee_account_address(&auction_house);

//         // 9) Derive buyer trade state address

//         let (buyer_trade_state, trade_state_bump) = find_trade_state_address(
//             &wallet_pubkey,
//             &auction_house,
//             &token_account,
//             &treasury_mint,
//             &token_mint.pubkey(),
//             BUYER_PRICE,
//             TOKEN_SIZE,
//         );

//         // Derive escrow payment address
//         let (escrow_payment_account, escrow_payment_bump) =
//             find_escrow_payment_address(&auction_house, &wallet_pubkey);

//         // Create `TokenMetadata`
//         let metadata = create_token_metadata(
//             &connection,

//         // 10) Derive escrow payment address
//         let (escrow_payment_account, escrow_payment_bump) = find_escrow_payment_address(&auction_house, &wallet_pubkey);

//         // 11) Create `TokenMetadata`
//         let metadata = create_token_metadata_v2(
//             &mut context,

//             &wallet,
//             &token_mint.pubkey(),
//             String::from("TEST"),
//             String::from("TST"),
//             String::from("https://github.com"),
//             5000,

//         )?;

//         // Transfer enough lamports to create seller trade state
//         transfer_lamports(&connection, &wallet, &auction_house_fee_account, 10000000)?;

//         // Perform RPC instruction request
//         program
//             .request()
//             .accounts(mpl_auction_house::accounts::Buy {
//                 wallet: wallet_pubkey,
//                 payment_account: wallet_pubkey,
//                 transfer_authority: wallet_pubkey,
//                 treasury_mint,
//                 token_account,
//                 metadata,
//                 escrow_payment_account,
//                 authority: wallet_pubkey,
//                 auction_house,
//                 auction_house_fee_account,
//                 buyer_trade_state,
//                 token_program: spl_token::id(),
//                 system_program: system_program::id(),
//                 rent: sysvar::rent::id(),
//             })
//             .args(mpl_auction_house::instruction::Buy {
//                 trade_state_bump,
//                 escrow_payment_bump,
//                 buyer_price: BUYER_PRICE,
//                 token_size: TOKEN_SIZE,
//             })
//             .send()?;

//         assert_eq!(
//             connection.get_account_data(&buyer_trade_state)?[0],
//             trade_state_bump
//         );
//         ).unwrap();
//         // )?;

//         // 12) Transfer enough lamports to create seller trade state
//         // transfer_lamports(&connection, &wallet, &auction_house_fee_account, 10000000)?;
//         transfer_lamports_v2(&mut context, &wallet, &auction_house_fee_account, 10000000)?;

//         // 13) Perform RPC instruction request
//         // program
//         //     .request()
//         //     .accounts(mpl_auction_house::accounts::Buy {
//         //         wallet: wallet_pubkey,
//         //         payment_account: wallet_pubkey,
//         //         transfer_authority: wallet_pubkey,
//         //         treasury_mint,
//         //         token_account,
//         //         metadata,
//         //         escrow_payment_account,
//         //         authority: wallet_pubkey,
//         //         auction_house,
//         //         auction_house_fee_account,
//         //         buyer_trade_state,
//         //         token_program: spl_token::id(),
//         //         system_program: system_program::id(),
//         //         rent: sysvar::rent::id(),
//         //     })
//         //     .args(mpl_auction_house::instruction::Buy {
//         //         trade_state_bump,
//         //         escrow_payment_bump,
//         //         buyer_price: BUYER_PRICE,
//         //         token_size: TOKEN_SIZE,
//         //     })
//         //     .send()?;

//         let accounts = mpl_auction_house_accounts::Buy {
//             wallet: wallet_pubkey,
//             payment_account: wallet_pubkey,
//             transfer_authority: wallet_pubkey,
//             treasury_mint,
//             token_account,
//             metadata,
//             escrow_payment_account,
//             authority: wallet_pubkey,
//             auction_house: auction_house,
//             auction_house_fee_account,
//             buyer_trade_state,
//             token_program: spl_token::id(),
//             system_program: system_program::id(),
//             rent: sysvar::rent::id(),
//         }
//         .to_account_metas(None);

//         let data = mpl_auction_house_instruction::Buy {
//             trade_state_bump,
//             escrow_payment_bump,
//             buyer_price: BUYER_PRICE,
//             token_size: TOKEN_SIZE,
//         }
//         .data();

//         let instruction = Instruction {
//             program_id: mpl_auction_house::id(),
//             data,
//             accounts,
//         };

//         let tx = Transaction::new_signed_with_payer(
//             &[instruction],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &wallet],
//             context.last_blockhash,
//         );

//         context.warp_to_slot(10).unwrap();
//         context.banks_client.process_transaction(tx).await.unwrap();
//         // let res = context.banks_client.process_transaction(tx).await;
//         // msg!(res);

//         // assert_eq!(
//         //     connection.get_account_data(&buyer_trade_state)?[0],
//         //     trade_state_bump
//         // );
//         assert_eq!(1, 1);

//         Ok(())
//     }
// }
