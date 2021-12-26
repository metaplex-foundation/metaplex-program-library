mod utils;

use anchor_client::{
    solana_sdk::{signature::Keypair, signer::Signer, system_program},
    ClientError,
};
use mpl_auction_house::{
    accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction,
    AuctionHouse,
};
use rand::rngs::OsRng;
use utils::{
    clone_keypair,
    setup_functions::{setup_auction_house, setup_client, setup_payer_wallet},
    transfer_lamports,
};

#[cfg(test)]
mod withdraw_from_fee {

    use super::*;

    #[test]
    fn success() -> Result<(), ClientError> {
        // Wallet
        let wallet = setup_payer_wallet();

        // Client
        let client = setup_client(clone_keypair(&wallet));

        // Program
        let program = client.program(mpl_auction_house::id());

        // Auction house authority
        let authority_keypair = Keypair::generate(&mut OsRng);

        // Treasury mint key
        let t_mint_key = spl_token::native_mint::id();

        let auction_house_key =
            setup_auction_house(&program, &authority_keypair.pubkey(), &t_mint_key).unwrap();
        let auction_house_account: AuctionHouse = program.account(auction_house_key)?;
        let wallet_pubkey = program.payer();

        let amount: u64 = 500_000_000;

        transfer_lamports(
            &program.rpc(),
            &wallet,
            &auction_house_account.auction_house_fee_account,
            amount * 2,
        )?;

        let fee_balance_before_withdraw = program
            .rpc()
            .get_balance(&auction_house_account.auction_house_fee_account)?;

        program
            .request()
            .signer(&authority_keypair)
            .accounts(mpl_auction_house_accounts::WithdrawFromFee {
                authority: authority_keypair.pubkey(),
                fee_withdrawal_destination: wallet_pubkey,
                auction_house_fee_account: auction_house_account.auction_house_fee_account,
                auction_house: auction_house_key,
                system_program: system_program::id(),
            })
            .args(mpl_auction_house_instruction::WithdrawFromFee { amount })
            .send()?;

        let fee_balance_after_withdraw = program
            .rpc()
            .get_balance(&auction_house_account.auction_house_fee_account)?;

        assert_eq!(
            amount,
            fee_balance_before_withdraw - fee_balance_after_withdraw
        );

        Ok(())
    }
}
