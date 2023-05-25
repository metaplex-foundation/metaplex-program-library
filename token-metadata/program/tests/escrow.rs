#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};
use utils::*;

mod escrow {
    use mpl_token_metadata::{escrow::find_escrow_account, state::EscrowAuthority};
    use solana_program::program_pack::Pack;

    use super::*;

    #[tokio::test]
    async fn smoke_test_success() {
        let mut context = program_test().start_with_context().await;

        // Create Escrow
        print!("\n=====Create Escrow=====\n");
        let parent_test_metadata = Metadata::new();
        let parent_test_master_edition = MasterEditionV2::new(&parent_test_metadata);
        parent_test_metadata
            .create_v3(
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

        let escrow_address = find_escrow_account(
            &parent_test_metadata.mint.pubkey(),
            &EscrowAuthority::TokenOwner,
        );
        print!("\nEscrow Address: {:#?}\n", escrow_address);

        let ix0 = mpl_token_metadata::escrow::create_escrow_account(
            mpl_token_metadata::ID,
            escrow_address.0,
            parent_test_metadata.pubkey,
            parent_test_metadata.mint.pubkey(),
            parent_test_metadata.token.pubkey(),
            parent_test_master_edition.pubkey,
            context.payer.pubkey(),
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

        // Transfer In
        print!("\n=====Transfer In=====\n");
        let attribute_test_metadata = Metadata::new();
        let attribute_test_master_edition = MasterEditionV2::new(&attribute_test_metadata);
        attribute_test_metadata
            .create_v3(
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

        let ix0 = spl_associated_token_account::instruction::create_associated_token_account(
            &context.payer.pubkey(),
            &escrow_address.0,
            &attribute_test_metadata.mint.pubkey(),
            &spl_token::ID,
        );
        let ix1 = spl_token::instruction::transfer(
            &spl_token::ID,
            &attribute_test_metadata.token.pubkey(),
            &escrow_attribute_token_account,
            &context.payer.pubkey(),
            &[&context.payer.pubkey()],
            1,
        )
        .unwrap();
        println!("{:?} {:?}", &context.payer, &attribute_test_metadata.token);

        let tx1 = Transaction::new_signed_with_payer(
            &[ix0, ix1],
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

        let ix2 = mpl_token_metadata::escrow::transfer_out_of_escrow(
            mpl_token_metadata::ID,
            escrow_address.0,
            parent_test_metadata.pubkey,
            context.payer.pubkey(),
            attribute_test_metadata.mint.pubkey(),
            escrow_attribute_token_account,
            payer_attribute_token_account,
            parent_test_metadata.mint.pubkey(),
            parent_test_metadata.token.pubkey(),
            None,
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

        context.banks_client.process_transaction(tx2).await.unwrap();

        let _metadata = attribute_test_metadata.get_data(&mut context).await;
        let escrow_account = get_account(&mut context, &escrow_address.0).await;
        let escrow: mpl_token_metadata::state::TokenOwnedEscrow =
            try_from_slice_unchecked(&escrow_account.data).unwrap();

        print!("\n{:#?}\n", escrow);
        println!("attribute_src:{:#?}", attribute_src);
        let attribute_dst_account = get_account(&mut context, &payer_attribute_token_account).await;
        let attribute_dst =
            spl_token::state::Account::unpack_from_slice(&attribute_dst_account.data).unwrap();
        assert!(attribute_dst.amount == 1);
        assert!(attribute_dst.mint == attribute_src.mint);
        assert!(attribute_dst.owner == context.payer.pubkey());
        println!("{:#?}", attribute_dst);
    }

    #[tokio::test]
    async fn double_transfer_out() {
        let mut context = program_test().start_with_context().await;

        // Create Escrow
        print!("\n=====Create Escrow=====\n");
        let parent_test_metadata = Metadata::new();
        let parent_test_master_edition = MasterEditionV2::new(&parent_test_metadata);
        parent_test_metadata
            .create_v3(
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

        let escrow_address = find_escrow_account(
            &parent_test_metadata.mint.pubkey(),
            &EscrowAuthority::TokenOwner,
        );
        print!("\nEscrow Address: {:#?}\n", escrow_address);

        let ix0 = mpl_token_metadata::escrow::create_escrow_account(
            mpl_token_metadata::ID,
            escrow_address.0,
            parent_test_metadata.pubkey,
            parent_test_metadata.mint.pubkey(),
            parent_test_metadata.token.pubkey(),
            parent_test_master_edition.pubkey,
            context.payer.pubkey(),
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

        // Transfer In
        print!("\n=====Transfer In=====\n");
        let attribute_test_metadata = Metadata::new();
        attribute_test_metadata
            .create_fungible_v3(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                true,
                None,
                None,
            )
            .await
            .unwrap();

        let escrow_attribute_token_account =
            spl_associated_token_account::get_associated_token_address(
                &escrow_address.0,
                &attribute_test_metadata.mint.pubkey(),
            );

        let ix0 = spl_associated_token_account::instruction::create_associated_token_account(
            &context.payer.pubkey(),
            &escrow_address.0,
            &attribute_test_metadata.mint.pubkey(),
            &spl_token::ID,
        );
        let ix1 = spl_token::instruction::transfer(
            &spl_token::ID,
            &attribute_test_metadata.token.pubkey(),
            &escrow_attribute_token_account,
            &context.payer.pubkey(),
            &[&context.payer.pubkey()],
            2,
        )
        .unwrap();
        println!("{:?} {:?}", &context.payer, &attribute_test_metadata.token);

        let tx1 = Transaction::new_signed_with_payer(
            &[ix0, ix1],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let attribute_src_account =
            get_account(&mut context, &attribute_test_metadata.token.pubkey()).await;
        let attribute_src =
            spl_token::state::Account::unpack_from_slice(&attribute_src_account.data).unwrap();
        println!("Attribute Source: {:#?}", attribute_src);
        assert!(attribute_src.amount == 10);

        let _metadata = attribute_test_metadata.get_data(&mut context).await;
        let escrow_account = get_account(&mut context, &escrow_address.0).await;
        let escrow: mpl_token_metadata::state::TokenOwnedEscrow =
            try_from_slice_unchecked(&escrow_account.data).unwrap();

        context.banks_client.process_transaction(tx1).await.unwrap();

        print!("\n{:#?}\n", escrow);
        let attribute_src_account =
            get_account(&mut context, &attribute_test_metadata.token.pubkey()).await;
        let attribute_src =
            spl_token::state::Account::unpack_from_slice(&attribute_src_account.data).unwrap();
        assert!(attribute_src.amount == 8);
        println!("{:#?}", attribute_src);
        let attribute_dst_account =
            get_account(&mut context, &escrow_attribute_token_account).await;
        let attribute_dst =
            spl_token::state::Account::unpack_from_slice(&attribute_dst_account.data).unwrap();
        assert!(attribute_dst.amount == 2);
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

        let ix2 = mpl_token_metadata::escrow::transfer_out_of_escrow(
            mpl_token_metadata::ID,
            escrow_address.0,
            parent_test_metadata.pubkey,
            context.payer.pubkey(),
            attribute_test_metadata.mint.pubkey(),
            escrow_attribute_token_account,
            payer_attribute_token_account,
            parent_test_metadata.mint.pubkey(),
            parent_test_metadata.token.pubkey(),
            None,
            1,
        );

        let ix3 = mpl_token_metadata::escrow::transfer_out_of_escrow(
            mpl_token_metadata::ID,
            escrow_address.0,
            parent_test_metadata.pubkey,
            context.payer.pubkey(),
            attribute_test_metadata.mint.pubkey(),
            escrow_attribute_token_account,
            payer_attribute_token_account,
            parent_test_metadata.mint.pubkey(),
            parent_test_metadata.token.pubkey(),
            None,
            1,
        );

        let tx2 = Transaction::new_signed_with_payer(
            &[ix2, ix3],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        let attribute_src_account =
            get_account(&mut context, &escrow_attribute_token_account).await;
        let attribute_src =
            spl_token::state::Account::unpack_from_slice(&attribute_src_account.data).unwrap();
        assert!(attribute_src.amount == 2);
        println!("Escrow Transfer Out: {:#?}", attribute_src);

        context.banks_client.process_transaction(tx2).await.unwrap();

        println!("Escrow Post-Transfer: {:#?}", attribute_src);
        let attribute_dst_account = get_account(&mut context, &payer_attribute_token_account).await;
        let attribute_dst =
            spl_token::state::Account::unpack_from_slice(&attribute_dst_account.data).unwrap();
        assert!(attribute_dst.amount == 2);
        assert!(attribute_dst.mint == attribute_src.mint);
        assert!(attribute_dst.owner == context.payer.pubkey());
        println!("Payer Post-Transfer: {:#?}", attribute_dst);
    }
}
