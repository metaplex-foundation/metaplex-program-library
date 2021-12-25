mod utils;

use anchor_client::{
    solana_sdk::{signature::Keypair, signer::Signer, system_program, sysvar},
    ClientError,
};
use mpl_auction_house::{
    accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction,
    AuctionHouse,
};
use rand::rngs::OsRng;
use utils::{
    helpers::derive_auction_house_buyer_escrow_account_key,
    setup_functions::{setup_auction_house, setup_client, setup_payer_wallet},
};

mod deposit {

    use super::*;
    #[test]
    fn success() -> Result<(), ClientError> {
        // Client
        let client = setup_client(setup_payer_wallet());

        // Program
        let program = client.program(mpl_auction_house::id());

        // Auction house authority
        let authority = Keypair::generate(&mut OsRng).pubkey();

        // Treasury mint key
        let t_mint_key = spl_token::native_mint::id();

        let auction_house_key = setup_auction_house(&program, &authority, &t_mint_key).unwrap();
        let auction_house_account: AuctionHouse = program.account(auction_house_key)?;
        let wallet = program.payer();

        let (escrow_payment_account, escrow_payment_bump) =
            derive_auction_house_buyer_escrow_account_key(&auction_house_key, &wallet);

        let amount: u64 = 500_000_000;

        program
            .request()
            .accounts(mpl_auction_house_accounts::Deposit {
                wallet,
                payment_account: program.payer(),
                transfer_authority: system_program::id(),
                escrow_payment_account,
                treasury_mint: auction_house_account.treasury_mint,
                authority,
                auction_house: auction_house_key,
                auction_house_fee_account: auction_house_account.auction_house_fee_account,
                token_program: spl_token::id(),
                system_program: system_program::id(),
                rent: sysvar::rent::id(),
            })
            .args(mpl_auction_house_instruction::Deposit {
                escrow_payment_bump,
                amount,
            })
            .send()?;

        let escrow_payment_account_obj = program.rpc().get_account(&escrow_payment_account)?;
        assert_eq!(amount, escrow_payment_account_obj.lamports);
        Ok(())
    }
}
