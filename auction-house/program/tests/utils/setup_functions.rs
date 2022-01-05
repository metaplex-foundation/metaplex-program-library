use std::{env, error, str::FromStr};
use solana_sdk::account::Account;

use super::{
    constants::{AUCTION_HOUSE, FEE_PAYER, TREASURY},
    helpers::{
        derive_auction_house_fee_account_key, derive_auction_house_key,
        derive_auction_house_treasury_key,
    },
};
use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signature},
        system_program, sysvar,
    },
    Client, ClientError, Cluster, Program,
};
use mpl_auction_house::{
    accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction,
    AuctionHouse,
};
use solana_program_test::*;

pub fn setup_payer_wallet() -> Keypair {
    let wallet_path = match env::var("LOCALNET_PAYER_WALLET") {
        Ok(val) => val,
        Err(_) => shellexpand::tilde("~/.config/solana/id.json").to_string(),
    };

    read_keypair_file(wallet_path).unwrap()
}

pub fn setup_client(payer_wallet: Keypair) -> Client {
    Client::new_with_options(
        Cluster::Localnet,
        payer_wallet,
        CommitmentConfig::processed(),
    )
}

pub fn setup_program(client: Client) -> Program {
    let pid = match env::var("AUCTION_HOUSE_PID") {
        Ok(val) => val,
        Err(_) => mpl_auction_house::id().to_string(),
    };

    let auction_house_pid = Pubkey::from_str(&pid).unwrap();
    client.program(auction_house_pid)
}

// pub fn setup_auction_house(
//     program: &Program,
//     authority: &Pubkey,
//     mint_key: &Pubkey,
// ) -> Result<Pubkey, ClientError> {
//     let seller_fee_basis_points: u16 = 100;

//     let twd_key = program.payer();
//     let fwd_key = program.payer();
//     let tdw_ata = twd_key;

//     let (auction_house_key, bump) = derive_auction_house_key(authority, mint_key, &program.id());

//     let (auction_fee_account_key, fee_payer_bump) =
//         derive_auction_house_fee_account_key(&auction_house_key, &program.id());

//     let (auction_house_treasury_key, treasury_bump) =
//         derive_auction_house_treasury_key(&auction_house_key, &program.id());

//     program
//         .request()
//         .accounts(mpl_auction_house_accounts::CreateAuctionHouse {
//             treasury_mint: *mint_key,
//             payer: program.payer(),
//             authority: *authority,
//             fee_withdrawal_destination: fwd_key,
//             treasury_withdrawal_destination: tdw_ata,
//             treasury_withdrawal_destination_owner: twd_key,
//             auction_house: auction_house_key,
//             auction_house_fee_account: auction_fee_account_key,
//             auction_house_treasury: auction_house_treasury_key,
//             token_program: spl_token::id(),
//             system_program: system_program::id(),
//             ata_program: spl_associated_token_account::id(),
//             rent: sysvar::rent::id(),
//         })
//         .args(mpl_auction_house_instruction::CreateAuctionHouse {
//             bump,
//             fee_payer_bump,
//             treasury_bump,
//             seller_fee_basis_points,
//             requires_sign_off: false,
//             can_change_sale_price: true,
//         })
//         .send()
//         .unwrap();

//     Ok(auction_house_key.pubkey())
// }

pub async fn setup_auction_house_v2(
    // program: &Program,
    // authority: &Pubkey,
    // mint_key: &Pubkey,
    context: &mut ProgramTestContext,
    payer_wallet: &Keypair,
) -> Result<Pubkey, ClientError> {

    use anchor_client::{
        solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, sysvar},
        ClientError,
    };
    use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
    use mpl_auction_house::{
        accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction,
        AuctionHouse,
    };
    use solana_program::instruction::Instruction;
    use solana_sdk::transaction::Transaction;
    use spl_token;

    // Payer Wallet
    let twd_key = payer_wallet.pubkey();
    let fwd_key = payer_wallet.pubkey();
    let t_mint_key = spl_token::native_mint::id();
    let tdw_ata = twd_key;

    let payer = payer_wallet.pubkey();
    let seller_fee_basis_points: u16 = 100;

    let authority = Keypair::new().pubkey();

    // Derive Auction House Key
    let auction_house_seeds = &[
        AUCTION_HOUSE.as_bytes(),
        authority.as_ref(),
        t_mint_key.as_ref(),
    ];
    let (auction_house_key, bump) =
        Pubkey::find_program_address(auction_house_seeds, &mpl_auction_house::id());

    // Derive Auction House Fee Account key
    let auction_fee_account_seeds = &[
        AUCTION_HOUSE.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
    ];
    let (auction_fee_account_key, fee_payer_bump) =
        Pubkey::find_program_address(auction_fee_account_seeds, &mpl_auction_house::id());

    // Derive Auction House Treasury Key
    let auction_house_treasury_seeds = &[
        AUCTION_HOUSE.as_bytes(),
        auction_house_key.as_ref(),
        TREASURY.as_bytes(),
    ];
    let (auction_house_treasury_key, treasury_bump) =
        Pubkey::find_program_address(auction_house_treasury_seeds, &mpl_auction_house::id());

    let accounts = mpl_auction_house_accounts::CreateAuctionHouse {
        treasury_mint: t_mint_key,
        payer,
        authority,
        fee_withdrawal_destination: fwd_key,
        treasury_withdrawal_destination: tdw_ata,
        treasury_withdrawal_destination_owner: twd_key,
        auction_house: auction_house_key,
        auction_house_fee_account: auction_fee_account_key,
        auction_house_treasury: auction_house_treasury_key,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        ata_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = mpl_auction_house_instruction::CreateAuctionHouse {
        bump,
        fee_payer_bump,
        treasury_bump,
        seller_fee_basis_points,
        requires_sign_off: true,
        can_change_sale_price: true,
    }
    .data();

    let instruction = Instruction {
        program_id: mpl_auction_house::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &payer_wallet],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let auction_house_acc = context
        .banks_client
        .get_account(auction_house_key)
        .await
        .expect("account not found")
        .expect("account empty");

    Ok(auction_house_key)
}

pub fn auction_house_program_test() -> ProgramTest {
    let mut program = ProgramTest::new("mpl_auction_house", mpl_auction_house::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    program
}
