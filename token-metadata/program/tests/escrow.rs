#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use spl_token_2022::{extension::StateWithExtensions, state::Account};
use utils::*;

mod escrow {
    use mpl_token_metadata::{escrow::find_escrow_account, state::EscrowAuthority};
    use solana_program::program_pack::Pack;
    use test_case::test_case;

    use super::*;

    #[test_case(spl_token::id(); "token")]
    #[test_case(spl_token_2022::id(); "token-2022")]
    #[tokio::test]
    async fn smoke_test_success(token_program_id: Pubkey) {
        let mut context = program_test().start_with_context().await;

        // Create Escrow
        print!("\n=====Create Escrow=====\n");
        let mut parent_test_metadata = Metadata::new();
        parent_test_metadata.token_program_id = token_program_id;
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
            mpl_token_metadata::id(),
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
        let mut attribute_test_metadata = Metadata::new();
        attribute_test_metadata.token_program_id = token_program_id;
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
            )
            .await
            .unwrap();
        attribute_test_master_edition
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();

        let escrow_attribute_token_account =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &escrow_address.0,
                &attribute_test_metadata.mint.pubkey(),
                &token_program_id,
            );

        let ix0 = spl_associated_token_account::instruction::create_associated_token_account(
            &context.payer.pubkey(),
            &escrow_address.0,
            &attribute_test_metadata.mint.pubkey(),
            &token_program_id,
        );
        #[allow(deprecated)]
        let ix1 = spl_token_2022::instruction::transfer(
            &token_program_id,
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
        let attribute_src = StateWithExtensions::<Account>::unpack(&attribute_src_account.data)
            .unwrap()
            .base;
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
        let attribute_src = StateWithExtensions::<Account>::unpack(&attribute_src_account.data)
            .unwrap()
            .base;
        assert!(attribute_src.amount == 0);
        println!("{:#?}", attribute_src);
        let attribute_dst_account =
            get_account(&mut context, &escrow_attribute_token_account).await;
        let attribute_dst = StateWithExtensions::<Account>::unpack(&attribute_dst_account.data)
            .unwrap()
            .base;
        assert!(attribute_dst.amount == 1);
        assert!(attribute_dst.mint == attribute_src.mint);
        assert!(attribute_dst.owner == escrow_address.0);
        println!("{:#?}", attribute_dst);

        // Transfer Out
        print!("\n=====Transfer Out=====\n");
        let payer_attribute_token_account =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &context.payer.pubkey(),
                &attribute_test_metadata.mint.pubkey(),
                &token_program_id,
            );

        let ix2 = mpl_token_metadata::escrow::transfer_out_of_escrow_with_token_program(
            mpl_token_metadata::id(),
            escrow_address.0,
            parent_test_metadata.pubkey,
            context.payer.pubkey(),
            attribute_test_metadata.mint.pubkey(),
            escrow_attribute_token_account,
            payer_attribute_token_account,
            parent_test_metadata.mint.pubkey(),
            parent_test_metadata.token.pubkey(),
            token_program_id,
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
        let attribute_src = StateWithExtensions::<Account>::unpack(&attribute_src_account.data)
            .unwrap()
            .base;
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
        let attribute_dst = StateWithExtensions::<Account>::unpack(&attribute_dst_account.data)
            .unwrap()
            .base;
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
            mpl_token_metadata::id(),
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
            .create_fungible_v2(
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
            &spl_token::id(),
        );
        let ix1 = spl_token::instruction::transfer(
            &spl_token::id(),
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
            mpl_token_metadata::id(),
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
            mpl_token_metadata::id(),
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
