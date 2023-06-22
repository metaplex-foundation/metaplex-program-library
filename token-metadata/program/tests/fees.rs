#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use utils::*;

mod fees {
    use mpl_token_metadata::{
        instruction::{collect_fees, BurnArgs, UpdateArgs},
        state::{CREATE_FEE, FEE_FLAG_CLEARED, METADATA_FEE_FLAG_INDEX},
    };
    use solana_program::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
    use solana_sdk::{
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        transaction::Transaction,
    };

    use super::*;

    #[tokio::test]
    async fn charge_create_metadata_v3() {
        let mut context = program_test().start_with_context().await;

        let md = Metadata::new();
        md.create_v3_default(&mut context).await.unwrap();

        md.assert_fee_flag_set(&mut context).await.unwrap();
        md.assert_create_fees_charged(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn charge_create() {
        let mut context = program_test().start_with_context().await;

        let mut nft = DigitalAsset::new();
        nft.create(
            &mut context,
            mpl_token_metadata::state::TokenStandard::NonFungible,
            None,
        )
        .await
        .unwrap();

        nft.assert_fee_flag_set(&mut context).await.unwrap();
        nft.assert_create_fees_charged(&mut context).await.unwrap();
    }

    #[tokio::test]
    async fn update_does_not_overwrite_flag() {
        let mut context = program_test().start_with_context().await;

        let update_authority = context.payer.dirty_clone();

        let mut nft = DigitalAsset::new();
        nft.create(
            &mut context,
            mpl_token_metadata::state::TokenStandard::NonFungible,
            None,
        )
        .await
        .unwrap();

        let mut args = UpdateArgs::default_as_update_authority();
        match &mut args {
            UpdateArgs::AsUpdateAuthorityV2 { is_mutable, .. } => {
                *is_mutable = Some(false);
            }
            _ => panic!("Unexpected enum variant"),
        }

        nft.update(&mut context, update_authority, args)
            .await
            .unwrap();

        nft.assert_fee_flag_set(&mut context).await.unwrap();
        nft.assert_create_fees_charged(&mut context).await.unwrap();
    }

    #[tokio::test]
    // Used for local QA testing and requires a keypair so excluded from CI.
    #[ignore]
    async fn collect_fees_max_accounts() {
        // Create NFTs and then collect the fees from the metadata accounts.
        let mut context = program_test().start_with_context().await;

        let authority_funding = 10 * LAMPORTS_PER_SOL;

        let authority =
            read_keypair_file("/media/veracrypt1/Levytx9LLPzAtDJJD7q813Zsm8zg9e1pb53mGxTKpD7.json")
                .unwrap();
        authority
            .airdrop(&mut context, authority_funding)
            .await
            .unwrap();

        let recipient = Keypair::new();

        let num_accounts = 25;

        let mut nfts = vec![];
        for _ in 0..num_accounts {
            let mut nft = DigitalAsset::new();
            nft.create(
                &mut context,
                mpl_token_metadata::state::TokenStandard::NonFungible,
                None,
            )
            .await
            .unwrap();
            nfts.push(nft);
        }

        let fee_accounts: Vec<Pubkey> = nfts.iter().map(|nft| nft.metadata).collect();

        let ix = collect_fees(recipient.pubkey(), fee_accounts.clone());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );
        println!("Transaction size: {:?}", tx.message().serialize().len());
        context.banks_client.process_transaction(tx).await.unwrap();

        let expected_balance = num_accounts * CREATE_FEE;

        let recipient_balance = get_account(&mut context, &recipient.pubkey())
            .await
            .lamports;

        assert_eq!(recipient_balance, expected_balance);

        // Fee flag in metadata accounts is cleared.
        for account in fee_accounts {
            let account = get_account(&mut context, &account).await;

            assert_eq!(account.data[METADATA_FEE_FLAG_INDEX], FEE_FLAG_CLEARED);
        }
    }

    #[tokio::test]
    // Used for local QA testing and requires a keypair so excluded from CI.
    #[ignore]
    async fn collect_fees_burned_account() {
        // Create NFTs and then collect the fees from the metadata accounts.
        let mut context = program_test().start_with_context().await;

        let nft_authority = context.payer.dirty_clone();

        let fee_authority_funding = LAMPORTS_PER_SOL;

        let fee_authority =
            read_keypair_file("/media/veracrypt1/Levytx9LLPzAtDJJD7q813Zsm8zg9e1pb53mGxTKpD7.json")
                .unwrap();
        fee_authority
            .airdrop(&mut context, fee_authority_funding)
            .await
            .unwrap();

        let recipient = Keypair::new();

        let mut nft = DigitalAsset::new();
        nft.create_and_mint(
            &mut context,
            mpl_token_metadata::state::TokenStandard::NonFungible,
            None,
            None,
            1,
        )
        .await
        .unwrap();

        let args = BurnArgs::V1 { amount: 1 };

        nft.burn(&mut context, nft_authority, args, None, None)
            .await
            .unwrap();

        let ix = collect_fees(recipient.pubkey(), vec![nft.metadata]);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&fee_authority.pubkey()),
            &[&fee_authority],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let expected_balance = CREATE_FEE;

        let recipient_balance = get_account(&mut context, &recipient.pubkey())
            .await
            .lamports;

        assert_eq!(recipient_balance, expected_balance);
    }
}
