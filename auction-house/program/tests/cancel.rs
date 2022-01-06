// //! Module provide tests for `Cancel` instruction.

// mod utils;

// use anchor_client::{
//     solana_client::rpc_client::RpcClient,
//     solana_sdk::{
//         signer::{keypair::read_keypair_file, Signer},
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

//     // Initialize anchor RPC `Client`
//     let client = Client::new(Cluster::Localnet, utils::clone_keypair(&wallet));

//     // Initialize vanilla `RpcClient`
//     let connection = RpcClient::new(Cluster::Localnet.url().to_string());

//     // Initialize `Program` handle
//     let program = client.program(mpl_auction_house::id());

//     // Derive native(wrapped) sol mint
//     let treasury_mint = spl_token::native_mint::id();

//     // Token mint for `TokenMetadata`.
//     let token_mint = utils::create_mint(&connection, &wallet)?;

//     // Derive / Create associated token account
//     let token_account =
//         utils::create_associated_token_account(&connection, &wallet, &token_mint.pubkey())?;

//     // Mint tokens
//     utils::mint_to(
//         &connection,
//         &wallet,
//         &token_mint.pubkey(),
//         &token_account,
//         1,
//     )?;

//     // Derive `AuctionHouse` address
//     let (auction_house, _) = utils::find_auction_house_address(&wallet_pubkey, &treasury_mint);

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

//     let fee_payer_before_lamports = connection.get_account(&auction_house_fee_account)?.lamports;
//     let trade_state_before_lamports = connection.get_account(&seller_trade_state)?.lamports;

//     // Perform RPC `Cancel` instruction request
//     program
//         .request()
//         .accounts(mpl_auction_house::accounts::Cancel {
//             auction_house,
//             auction_house_fee_account,
//             authority: wallet_pubkey,
//             token_account,
//             token_mint: token_mint.pubkey(),
//             token_program: spl_token::id(),
//             trade_state: seller_trade_state,
//             wallet: wallet_pubkey,
//         })
//         .args(mpl_auction_house::instruction::Cancel {
//             _buyer_price: BUYER_PRICE,
//             _token_size: TOKEN_SIZE,
//         })
//         .send()?;

//     let fee_payer_after_lamports = connection.get_account(&auction_house_fee_account)?.lamports;

//     assert_eq!(
//         fee_payer_before_lamports + trade_state_before_lamports,
//         fee_payer_after_lamports
//     );
//     assert!(connection.get_account(&seller_trade_state).is_err());

//     Ok(())
// }
