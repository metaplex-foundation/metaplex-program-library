mod utils;

use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        system_program, sysvar,
    },
    Client, ClientError, Cluster,
};
use mpl_auction_house::{
    accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction,
    AuctionHouse,
};
use rand::rngs::OsRng;
use spl_token;
use std::env;

use utils::constants::{AUCTION_HOUSE, FEE_PAYER, TREASURY};

#[cfg(test)]
mod create_auction_house {

    use super::*;

    #[test]
    fn success() -> Result<(), ClientError> {
        let wallet_path = match env::var("LOCALNET_PAYER_WALLET") {
            Ok(val) => val,
            Err(_) => shellexpand::tilde("~/.config/solana/id.json").to_string(),
        };

        let payer_wallet = read_keypair_file(wallet_path).unwrap();

        // Client.
        let client = Client::new_with_options(
            Cluster::Localnet,
            payer_wallet,
            CommitmentConfig::processed(),
        );

        // Program client.
        let program = client.program(mpl_auction_house::id());

        let twd_key = program.payer();
        let fwd_key = program.payer();
        let t_mint_key = spl_token::native_mint::id();
        let tdw_ata = twd_key;

        let payer = program.payer();
        let seller_fee_basis_points: u16 = 100;

        let authority = Keypair::generate(&mut OsRng).pubkey();

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

        program
            .request()
            .accounts(mpl_auction_house_accounts::CreateAuctionHouse {
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
            })
            .args(mpl_auction_house_instruction::CreateAuctionHouse {
                bump,
                fee_payer_bump,
                treasury_bump,
                seller_fee_basis_points,
                requires_sign_off: true,
                can_change_sale_price: true,
            })
            .send()?;

        let auction_house_account: AuctionHouse = program.account(auction_house_key)?;

        assert_eq!(
            auction_fee_account_key,
            auction_house_account.auction_house_fee_account
        );
        assert_eq!(
            auction_house_treasury_key,
            auction_house_account.auction_house_treasury
        );
        assert_eq!(
            tdw_ata,
            auction_house_account.treasury_withdrawal_destination
        );
        assert_eq!(fwd_key, auction_house_account.fee_withdrawal_destination);
        assert_eq!(t_mint_key, auction_house_account.treasury_mint);
        assert_eq!(authority, auction_house_account.authority);
        assert_eq!(authority, auction_house_account.creator);

        assert_eq!(bump, auction_house_account.bump);
        assert_eq!(treasury_bump, auction_house_account.treasury_bump);
        assert_eq!(fee_payer_bump, auction_house_account.fee_payer_bump);
        assert_eq!(
            seller_fee_basis_points,
            auction_house_account.seller_fee_basis_points
        );
        assert_eq!(true, auction_house_account.requires_sign_off);
        assert_eq!(true, auction_house_account.can_change_sale_price);

        Ok(())
    }
}
