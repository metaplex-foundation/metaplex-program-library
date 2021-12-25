#![cfg(test)]
pub mod utils;

use anchor_client::{
    solana_sdk::{signature::Keypair, signer::Signer, system_program, sysvar},
    ClientError,
};
use mpl_auction_house::{
    accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction,
    AuctionHouse,
};
use rand::rngs::OsRng;
use utils::setup_functions::{setup_auction_house, setup_client, setup_payer_wallet};

mod update_auction_house {

    use super::*;

    #[test]
    fn success() -> Result<(), ClientError> {
        // Client
        let client = setup_client(setup_payer_wallet());

        // Program
        let program = client.program(mpl_auction_house::id());

        // Auction house authority
        let authority_keypair = Keypair::generate(&mut OsRng);
        let authority = authority_keypair.pubkey();
        // New auction house authority
        let new_authority = Keypair::generate(&mut OsRng).pubkey();

        // Treasury mint key
        let t_mint_key = spl_token::native_mint::id();

        let auction_house_key = setup_auction_house(&program, &authority, &t_mint_key).unwrap();

        let twd_key = program.payer();
        let fwd_key = program.payer();
        let tdw_ata = twd_key;

        let seller_fee_basis_points: u16 = 345;

        program
            .request()
            .signer(&authority_keypair)
            .accounts(mpl_auction_house_accounts::UpdateAuctionHouse {
                treasury_mint: t_mint_key,
                payer: program.payer(),
                authority,
                new_authority,
                fee_withdrawal_destination: fwd_key,
                treasury_withdrawal_destination: tdw_ata,
                treasury_withdrawal_destination_owner: twd_key,
                auction_house: auction_house_key,
                token_program: spl_token::id(),
                system_program: system_program::id(),
                ata_program: spl_associated_token_account::id(),
                rent: sysvar::rent::id(),
            })
            .args(mpl_auction_house_instruction::UpdateAuctionHouse {
                seller_fee_basis_points: Some(seller_fee_basis_points),
                requires_sign_off: Some(false),
                can_change_sale_price: Some(false),
            })
            .send()?;

        let auction_house_account: AuctionHouse = program.account(auction_house_key)?;

        assert_eq!(
            seller_fee_basis_points,
            auction_house_account.seller_fee_basis_points
        );

        assert_eq!(false, auction_house_account.requires_sign_off);
        assert_eq!(false, auction_house_account.can_change_sale_price);

        Ok(())
    }
}
