mod utils;

#[cfg(test)]
mod create_auction_house {

    use anchor_client::{
        solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, sysvar},
        ClientError,
    };
    use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
    use mpl_auction_house::{
        accounts as mpl_auction_house_accounts, instruction as mpl_auction_house_instruction,
        AuctionHouse,
    };
    use solana_program::{instruction::Instruction, system_instruction};
    use solana_sdk::transaction::Transaction;
    use spl_token;

    use super::utils::constants::{AUCTION_HOUSE, FEE_PAYER, TREASURY};

    use solana_program_test::*;

    pub fn auction_house_program_test<'a>() -> ProgramTest {
        let mut program = ProgramTest::new("mpl_auction_house", mpl_auction_house::id(), None);
        program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program
    }

    pub async fn airdrop(context: &mut ProgramTestContext, receiver: &Pubkey, amount: u64) {
        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::transfer(
                &context.payer.pubkey(),
                receiver,
                amount,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();
    }

    #[tokio::test]
    async fn success() -> Result<(), ClientError> {
        let mut context = auction_house_program_test().start_with_context().await;

        // Payer Wallet
        let payer_wallet = Keypair::new();

        airdrop(&mut context, &payer_wallet.pubkey(), 10_000_000_000).await;

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

        let auction_house_data =
            AuctionHouse::try_deserialize(&mut auction_house_acc.data.as_ref()).unwrap();

        assert_eq!(
            auction_fee_account_key,
            auction_house_data.auction_house_fee_account
        );
        assert_eq!(
            auction_house_treasury_key,
            auction_house_data.auction_house_treasury
        );
        assert_eq!(tdw_ata, auction_house_data.treasury_withdrawal_destination);
        assert_eq!(fwd_key, auction_house_data.fee_withdrawal_destination);
        assert_eq!(t_mint_key, auction_house_data.treasury_mint);
        assert_eq!(authority, auction_house_data.authority);
        assert_eq!(authority, auction_house_data.creator);

        assert_eq!(bump, auction_house_data.bump);
        assert_eq!(treasury_bump, auction_house_data.treasury_bump);
        assert_eq!(fee_payer_bump, auction_house_data.fee_payer_bump);
        assert_eq!(
            seller_fee_basis_points,
            auction_house_data.seller_fee_basis_points
        );
        assert_eq!(true, auction_house_data.requires_sign_off);
        assert_eq!(true, auction_house_data.can_change_sale_price);

        Ok(())
    }
}