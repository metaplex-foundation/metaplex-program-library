#![cfg(feature = "test-bpf")]
pub mod utils;

use borsh::BorshDeserialize;
use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use utils::*;

mod escrow {
    use mpl_token_metadata::{
        pda::{find_escrow_account, find_escrow_constraints_model_account},
        state::EscrowConstraintsModel,
    };
    use solana_program::program_pack::Pack;

    use super::*;

    // #[tokio::test]
    // async fn create_escrow_account_success() {
    //     let mut context = program_test().start_with_context().await;
    //     let test_metadata = Metadata::new();
    //     let test_master_edition = MasterEditionV2::new(&test_metadata);
    //     test_metadata
    //         .create_v2(
    //             &mut context,
    //             "Test".to_string(),
    //             "TST".to_string(),
    //             "uri".to_string(),
    //             None,
    //             10,
    //             true,
    //             None,
    //             None,
    //             None,
    //         )
    //         .await
    //         .unwrap();

    //     test_master_edition
    //         .create_v3(&mut context, Some(0))
    //         .await
    //         .unwrap();

    //     let escrow_address = find_escrow_account(&test_metadata.mint.pubkey());

    //     let ix = mpl_token_metadata::instruction::create_escrow_account(
    //         mpl_token_metadata::id(),
    //         escrow_address.0,
    //         test_metadata.pubkey,
    //         test_metadata.mint.pubkey(),
    //         test_master_edition.pubkey,
    //         context.payer.pubkey(),
    //         // solana_program::system_program::id(),
    //         // solana_program::sysvar::rent::id(),
    //         None,
    //     );
    //     println!("{:?} {:?}", &context.payer, &test_metadata.token);

    //     let tx = Transaction::new_signed_with_payer(
    //         &[ix],
    //         Some(&context.payer.pubkey()),
    //         &[&context.payer],
    //         context.last_blockhash,
    //     );

    //     context.banks_client.process_transaction(tx).await.unwrap();

    //     let metadata = test_metadata.get_data(&mut context).await;
    //     let escrow_account = get_account(&mut context, &escrow_address.0).await;
    //     let escrow: mpl_token_metadata::state::TokenOwnedEscrow =
    //         try_from_slice_unchecked(&escrow_account.data).unwrap();
    //     print!("\n{:#?}\n", escrow);
    //     assert!(escrow.tokens.is_empty());
    //     assert!(escrow.tokens.is_empty());
    //     assert!(escrow.model == None);
    // }

    #[tokio::test]
    async fn smoke_test_success() {
        let mut context = program_test().start_with_context().await;

        // Create Escrow
        print!("\n=====Create Escrow=====\n");
        let parent_test_metadata = Metadata::new();
        let parent_test_master_edition = MasterEditionV2::new(&parent_test_metadata);
        parent_test_metadata
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        parent_test_master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let escrow_address = find_escrow_account(&parent_test_metadata.mint.pubkey());
        print!("\nEscrow Address: {:#?}\n", escrow_address);

        let ix0 = mpl_token_metadata::instruction::create_escrow_account(
            mpl_token_metadata::id(),
            escrow_address.0,
            parent_test_metadata.pubkey,
            parent_test_metadata.mint.pubkey(),
            parent_test_master_edition.pubkey,
            context.payer.pubkey(),
            // solana_program::system_program::id(),
            // solana_program::sysvar::rent::id(),
            None,
        );
        println!("{:?} {:?}", &context.payer, &parent_test_metadata.token);

        let tx0 = Transaction::new_signed_with_payer(
            &[ix0],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx0).await.unwrap();

        let _metadata = parent_test_metadata.get_data(&mut context).await;
        let escrow_account = get_account(&mut context, &escrow_address.0).await;
        let escrow: mpl_token_metadata::state::TokenOwnedEscrow =
            try_from_slice_unchecked(&escrow_account.data).unwrap();
        print!("\n{:#?}\n", escrow);
        assert!(escrow.tokens.is_empty());
        assert!(escrow.tokens.is_empty());
        assert!(escrow.model == None);

        // Transfer In
        print!("\n=====Transfer In=====\n");
        let attribute_test_metadata = Metadata::new();
        let attribute_test_master_edition = MasterEditionV2::new(&attribute_test_metadata);
        attribute_test_metadata
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        attribute_test_master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let escrow_attribute_token_account =
            spl_associated_token_account::get_associated_token_address(
                &escrow_address.0,
                &attribute_test_metadata.mint.pubkey(),
            );

        let ix1 = mpl_token_metadata::instruction::transfer_into_escrow(
            mpl_token_metadata::id(),
            escrow_address.0,
            context.payer.pubkey(),
            attribute_test_metadata.mint.pubkey(),
            attribute_test_metadata.token.pubkey(),
            escrow_attribute_token_account,
            attribute_test_metadata.pubkey,
            parent_test_metadata.mint.pubkey(),
            parent_test_metadata.token.pubkey(),
            solana_program::system_program::id(),
            // spl_token::id(),
            // solana_program::sysvar::rent::id(),
            1,
        );
        println!("{:?} {:?}", &context.payer, &attribute_test_metadata.token);

        let tx1 = Transaction::new_signed_with_payer(
            &[ix1],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let attribute_src_account =
            get_account(&mut context, &attribute_test_metadata.token.pubkey()).await;
        let attribute_src =
            spl_token::state::Account::unpack_from_slice(&attribute_src_account.data).unwrap();
        assert!(attribute_src.amount == 1);
        println!("{:#?}", attribute_src);
        // let attribute_dst_account =
        //     get_account(&mut context, &escrow_attribute_token_account).await;
        // let attribute_dst =
        //     spl_token::state::Account::unpack_from_slice(&attribute_dst_account.data).unwrap();
        // println!("{:#?}", attribute_dst);

        context.banks_client.process_transaction(tx1).await.unwrap();

        let _metadata = attribute_test_metadata.get_data(&mut context).await;
        let escrow_account = get_account(&mut context, &escrow_address.0).await;
        let escrow: mpl_token_metadata::state::TokenOwnedEscrow =
            try_from_slice_unchecked(&escrow_account.data).unwrap();

        print!("\n{:#?}\n", escrow);
        let attribute_src_account =
            get_account(&mut context, &attribute_test_metadata.token.pubkey()).await;
        let attribute_src =
            spl_token::state::Account::unpack_from_slice(&attribute_src_account.data).unwrap();
        assert!(attribute_src.amount == 0);
        println!("{:#?}", attribute_src);
        let attribute_dst_account =
            get_account(&mut context, &escrow_attribute_token_account).await;
        let attribute_dst =
            spl_token::state::Account::unpack_from_slice(&attribute_dst_account.data).unwrap();
        assert!(attribute_dst.amount == 1);
        assert!(attribute_dst.mint == attribute_src.mint);
        assert!(attribute_dst.owner == escrow_address.0);
        println!("{:#?}", attribute_dst);

        // Transfer Out
        print!("\n=====Transfer Out=====\n");
        let payer_attribute_token_account =
            spl_associated_token_account::get_associated_token_address(
                &context.payer.pubkey(),
                &attribute_test_metadata.mint.pubkey(),
            );

        let ix2 = mpl_token_metadata::instruction::transfer_out_of_escrow(
            mpl_token_metadata::id(),
            escrow_address.0,
            context.payer.pubkey(),
            attribute_test_metadata.mint.pubkey(),
            escrow_attribute_token_account,
            payer_attribute_token_account,
            attribute_test_metadata.pubkey,
            parent_test_metadata.mint.pubkey(),
            parent_test_metadata.token.pubkey(),
            solana_program::system_program::id(),
            // spl_token::id(),
            // solana_program::sysvar::rent::id(),
            1,
        );
        println!("{:?} {:?}", &context.payer, &attribute_test_metadata.token);

        let tx2 = Transaction::new_signed_with_payer(
            &[ix2],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let attribute_src_account =
            get_account(&mut context, &escrow_attribute_token_account).await;
        let attribute_src =
            spl_token::state::Account::unpack_from_slice(&attribute_src_account.data).unwrap();
        assert!(attribute_src.amount == 1);
        println!("{:#?}", attribute_src);
        // let attribute_dst_account =
        //     get_account(&mut context, &escrow_attribute_token_account).await;
        // let attribute_dst =
        //     spl_token::state::Account::unpack_from_slice(&attribute_dst_account.data).unwrap();
        // println!("{:#?}", attribute_dst);

        context.banks_client.process_transaction(tx2).await.unwrap();

        let _metadata = attribute_test_metadata.get_data(&mut context).await;
        let escrow_account = get_account(&mut context, &escrow_address.0).await;
        let escrow: mpl_token_metadata::state::TokenOwnedEscrow =
            try_from_slice_unchecked(&escrow_account.data).unwrap();

        print!("\n{:#?}\n", escrow);
        let attribute_src_account =
            get_account(&mut context, &escrow_attribute_token_account).await;
        let attribute_src =
            spl_token::state::Account::unpack_from_slice(&attribute_src_account.data).unwrap();
        println!("attribute_src:{:#?}", attribute_src);
        assert!(attribute_src.amount == 0);
        let attribute_dst_account = get_account(&mut context, &payer_attribute_token_account).await;
        let attribute_dst =
            spl_token::state::Account::unpack_from_slice(&attribute_dst_account.data).unwrap();
        assert!(attribute_dst.amount == 1);
        assert!(attribute_dst.mint == attribute_src.mint);
        assert!(attribute_dst.owner == context.payer.pubkey());
        println!("{:#?}", attribute_dst);
    }

    #[tokio::test]
    async fn create_escrow_constraints_model() {
        let mut context = program_test().start_with_context().await;

        let (escrow_constraints_model_addr, _escrow_constraints_model_bump) =
            find_escrow_constraints_model_account(&context.payer.pubkey(), "test_model");

        let ix = mpl_token_metadata::instruction::create_escrow_constraints_model(
            mpl_token_metadata::id(),
            escrow_constraints_model_addr,
            context.payer.pubkey(),
            context.payer.pubkey(),
            solana_program::sysvar::rent::id(),
            solana_program::system_program::id(),
            "test_model",
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(tx)
            .await
            .expect("failed to process create_escrow_constraints_model_account");

        let model = context
            .banks_client
            .get_account(escrow_constraints_model_addr)
            .await
            .unwrap()
            .expect("failed to find escrow constraints model account");

        let model = EscrowConstraintsModel::deserialize(&mut model.data.as_slice())
            .expect("failed to deserialize escrow constraints model");

        assert_eq!(model.name, "test_model");
    }
}
