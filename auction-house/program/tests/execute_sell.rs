// //! Module provide tests for `ExecuteSell` instruction.

// mod utils;

// use anchor_client::{
//     solana_client::rpc_client::RpcClient,
//     solana_sdk::{
//         signer::{
//             keypair::{read_keypair_file, Keypair},
//             Signer,
//         },
//         system_program, sysvar,
//     },
//     Client, Cluster,
// };
// use std::{env, error};

// #[test]
// fn success() -> Result<(), Box<dyn error::Error>> {
//     const BUYER_PRICE: u64 = 1;
//     const TOKEN_SIZE: u64 = 1;

//     // Load `Localnet` keypair
//     let wallet = read_keypair_file(env::var("LOCALNET_PAYER_WALLET")?)?;
//     let wallet_pubkey = wallet.pubkey();

//     // Create buyer keypair
//     let buyer_wallet = Keypair::new();

//     // Initialize anchor RPC `Client`
//     let client = Client::new(Cluster::Localnet, utils::clone_keypair(&wallet));

//     // Initialize vanilla `RpcClient`
//     let connection = RpcClient::new(Cluster::Localnet.url().to_string());

//     // Transfer enough lamports to create buyer wallet
//     utils::transfer_lamports(&connection, &wallet, &buyer_wallet.pubkey(), 1000000000)?;

//     // Initialize `Program` handle
//     let program = client.program(mpl_auction_house::id());

//     // Derive native(wrapped) sol mint
//     let treasury_mint = spl_token::native_mint::id();

//     // Token mint for `TokenMetadata`.
//     let token_mint = utils::create_mint(&connection, &wallet)?;

//     // Derive / Create associated token account
//     let token_account =
//         utils::create_associated_token_account(&connection, &wallet, &token_mint.pubkey())?;

//     // Derive / Create buyer associated token account
//     let buyer_token_account =
//         utils::create_associated_token_account(&connection, &buyer_wallet, &token_mint.pubkey())?;

//     // Mint tokens to seller
//     utils::mint_to(
//         &connection,
//         &wallet,
//         &token_mint.pubkey(),
//         &token_account,
//         1,
//     )?;

//     // Mint tokens to buyer
//     utils::mint_to(
//         &connection,
//         &wallet,
//         &token_mint.pubkey(),
//         &buyer_token_account,
//         1,
//     )?;

//     // Derive `AuctionHouse` address
//     let (auction_house, _) = utils::find_auction_house_address(&wallet_pubkey, &treasury_mint);

//     // Derive `AuctionHouse` treasury address
//     let (auction_house_treasury, _) = utils::find_auction_house_treasury_address(&auction_house);

//     // Derive `AuctionHouse` fee account
//     let (auction_house_fee_account, _) =
//         utils::find_auction_house_fee_account_address(&auction_house);

//     // Derive `Program` as a signer address
//     let (program_as_signer, program_as_signer_bump) = utils::find_program_as_signer_address();

//     // Derive free seller trade state address
//     let (free_seller_trade_state, free_seller_trade_state_bump) = utils::find_trade_state_address(
//         &wallet_pubkey,
//         &auction_house,
//         &token_account,
//         &treasury_mint,
//         &token_mint.pubkey(),
//         0,
//         TOKEN_SIZE,
//     );

//     // Derive seller trade state address
//     let (seller_trade_state, seller_trade_state_bump) = utils::find_trade_state_address(
//         &wallet_pubkey,
//         &auction_house,
//         &token_account,
//         &treasury_mint,
//         &token_mint.pubkey(),
//         BUYER_PRICE,
//         TOKEN_SIZE,
//     );

//     // Create `TokenMetadata`
//     let metadata = utils::create_token_metadata(
//         &connection,
//         &wallet,
//         &token_mint.pubkey(),
//         String::from("TEST"),
//         String::from("TST"),
//         String::from("https://github.com"),
//         5000,
//     )?;

//     // Transfer enough lamports to create seller trade state
//     utils::transfer_lamports(&connection, &wallet, &auction_house_fee_account, 10000000)?;

//     // Perform RPC `Sell` instruction request
//     program
//         .request()
//         .accounts(mpl_auction_house::accounts::Sell {
//             auction_house,
//             auction_house_fee_account,
//             authority: wallet_pubkey,
//             free_seller_trade_state,
//             metadata,
//             program_as_signer,
//             rent: sysvar::rent::id(),
//             seller_trade_state,
//             system_program: system_program::id(),
//             token_account,
//             token_program: spl_token::id(),
//             wallet: wallet_pubkey,
//         })
//         .args(mpl_auction_house::instruction::Sell {
//             _free_trade_state_bump: free_seller_trade_state_bump,
//             _program_as_signer_bump: program_as_signer_bump,
//             buyer_price: BUYER_PRICE,
//             token_size: TOKEN_SIZE,
//             trade_state_bump: seller_trade_state_bump,
//         })
//         .send()?;

//     // Obtain escrow payment PDA
//     let (escrow_payment_account, escrow_payment_bump) =
//         utils::find_escrow_payment_address(&auction_house, &buyer_wallet.pubkey());

//     // Transfer enough lamports to create seller trade state
//     utils::transfer_lamports(&connection, &wallet, &escrow_payment_account, 1000000000)?;

//     // Derive buyer trade state address
//     let (buyer_trade_state, buyer_trade_state_bump) = utils::find_trade_state_address(
//         &buyer_wallet.pubkey(),
//         &auction_house,
//         &token_account,
//         &treasury_mint,
//         &token_mint.pubkey(),
//         BUYER_PRICE,
//         TOKEN_SIZE,
//     );

//     // Perform RPC `Buy` instruction request
//     program
//         .request()
//         .accounts(mpl_auction_house::accounts::Buy {
//             auction_house,
//             auction_house_fee_account,
//             authority: wallet.pubkey(),
//             buyer_trade_state,
//             escrow_payment_account,
//             metadata,
//             payment_account: buyer_wallet.pubkey(),
//             rent: sysvar::rent::id(),
//             system_program: system_program::id(),
//             token_account,
//             token_program: spl_token::id(),
//             transfer_authority: buyer_wallet.pubkey(),
//             treasury_mint,
//             wallet: buyer_wallet.pubkey(),
//         })
//         .args(mpl_auction_house::instruction::Buy {
//             buyer_price: BUYER_PRICE,
//             escrow_payment_bump,
//             token_size: TOKEN_SIZE,
//             trade_state_bump: buyer_trade_state_bump,
//         })
//         .signer(&buyer_wallet)
//         .send()?;

//     let escrow_lamports_before = connection.get_account(&escrow_payment_account)?.lamports;
//     let buyer_token_account_before =
//         utils::get_token_account(&connection, &buyer_token_account)?.amount;

//     // Perform RPC instruction request
//     program
//         .request()
//         .accounts(mpl_auction_house::accounts::ExecuteSale {
//             buyer: buyer_wallet.pubkey(),
//             seller: wallet.pubkey(),
//             token_account,
//             token_mint: token_mint.pubkey(),
//             metadata,
//             treasury_mint,
//             escrow_payment_account,
//             seller_payment_receipt_account: wallet.pubkey(),
//             buyer_receipt_token_account: buyer_token_account,
//             authority: wallet.pubkey(),
//             auction_house,
//             auction_house_fee_account,
//             auction_house_treasury,
//             buyer_trade_state,
//             seller_trade_state,
//             free_trade_state: free_seller_trade_state,
//             token_program: spl_token::id(),
//             system_program: system_program::id(),
//             ata_program: spl_associated_token_account::id(),
//             program_as_signer,
//             rent: sysvar::rent::id(),
//         })
//         .args(mpl_auction_house::instruction::ExecuteSale {
//             _free_trade_state_bump: free_seller_trade_state_bump,
//             buyer_price: BUYER_PRICE,
//             escrow_payment_bump,
//             program_as_signer_bump,
//             token_size: TOKEN_SIZE,
//         })
//         .send()?;

//     let escrow_lamports_after = connection.get_account(&escrow_payment_account)?.lamports;
//     let buyer_token_account_after =
//         utils::get_token_account(&connection, &buyer_token_account)?.amount;

//     assert_eq!(escrow_lamports_before - TOKEN_SIZE, escrow_lamports_after);
//     assert_eq!(
//         buyer_token_account_before + TOKEN_SIZE,
//         buyer_token_account_after
//     );

//     Ok(())
// }
