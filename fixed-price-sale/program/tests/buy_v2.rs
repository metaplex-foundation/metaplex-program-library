mod utils;

#[cfg(feature = "test-bpf")]
mod buy_v2 {
    use crate::{
        setup_context,
        utils::{
            helpers::{
                airdrop, buy_one_v2, buy_setup, create_mint, create_token_account, mint_to,
                BuyManager,
            },
            setup_functions::{setup_selling_resource, setup_store},
        },
    };
    use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
    use mpl_fixed_price_sale::{
        accounts as mpl_fixed_price_sale_accounts, instruction as mpl_fixed_price_sale_instruction,
        state::{SellingResource, TradeHistory},
        utils::{
            find_trade_history_address, find_treasury_owner_address, find_vault_owner_address,
        },
    };
    use mpl_token_metadata::{
        instruction::burn_edition_nft,
        state::{MasterEditionV2, TokenMetadataAccount},
    };
    use solana_program::clock::Clock;
    use solana_program_test::*;
    use solana_sdk::{
        commitment_config::CommitmentLevel, instruction::Instruction, pubkey::Pubkey,
        signature::Keypair, signer::Signer, system_program, sysvar, transaction::Transaction,
    };

    #[tokio::test]
    async fn mint_after_edition_burn() {
        setup_context!(context, mpl_fixed_price_sale, mpl_token_metadata);
        let (admin_wallet, store_keypair) = setup_store(&mut context).await;

        let (selling_resource_keypair, selling_resource_owner_keypair, vault) =
            setup_selling_resource(
                &mut context,
                &admin_wallet,
                &store_keypair,
                100,
                None,
                true,
                false,
                10,
            )
            .await;

        airdrop(
            &mut context,
            &selling_resource_owner_keypair.pubkey(),
            10_000_000_000,
        )
        .await;

        let market_keypair = Keypair::new();

        let treasury_mint_keypair = Keypair::new();
        create_mint(
            &mut context,
            &treasury_mint_keypair,
            &admin_wallet.pubkey(),
            0,
        )
        .await;

        let (treasury_owner, treasury_owner_bump) = find_treasury_owner_address(
            &treasury_mint_keypair.pubkey(),
            &selling_resource_keypair.pubkey(),
        );

        let treasury_holder_keypair = Keypair::new();
        create_token_account(
            &mut context,
            &treasury_holder_keypair,
            &treasury_mint_keypair.pubkey(),
            &treasury_owner,
        )
        .await;

        let start_date = context
            .banks_client
            .get_sysvar::<Clock>()
            .await
            .unwrap()
            .unix_timestamp
            + 1;

        let name = "Marktname".to_string();
        let description = "Marktbeschreibung".to_string();
        let mutable = true;
        let price = 1_000_000;
        let pieces_in_one_wallet = Some(10);

        // CreateMarket
        let accounts = mpl_fixed_price_sale_accounts::CreateMarket {
            market: market_keypair.pubkey(),
            store: store_keypair.pubkey(),
            selling_resource_owner: selling_resource_owner_keypair.pubkey(),
            selling_resource: selling_resource_keypair.pubkey(),
            mint: treasury_mint_keypair.pubkey(),
            treasury_holder: treasury_holder_keypair.pubkey(),
            owner: treasury_owner,
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = mpl_fixed_price_sale_instruction::CreateMarket {
            _treasury_owner_bump: treasury_owner_bump,
            name: name.to_owned(),
            description: description.to_owned(),
            mutable,
            price,
            pieces_in_one_wallet,
            start_date: start_date as u64,
            end_date: None,
            gating_config: None,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_fixed_price_sale::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &market_keypair,
                &selling_resource_owner_keypair,
            ],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction_with_commitment(tx, CommitmentLevel::Confirmed)
            .await
            .unwrap();

        let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
        context.warp_to_slot(clock.slot + 1500).unwrap();

        // Buy setup
        let selling_resource_data = context
            .banks_client
            .get_account(selling_resource_keypair.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data;
        let selling_resource =
            SellingResource::try_deserialize(&mut selling_resource_data.as_ref()).unwrap();

        let (trade_history, trade_history_bump) =
            find_trade_history_address(&context.payer.pubkey(), &market_keypair.pubkey());
        let (owner, vault_owner_bump) =
            find_vault_owner_address(&selling_resource.resource, &selling_resource.store);

        let payer_pubkey = context.payer.pubkey();

        let user_token_account = Keypair::new();
        create_token_account(
            &mut context,
            &user_token_account,
            &treasury_mint_keypair.pubkey(),
            &payer_pubkey,
        )
        .await;

        mint_to(
            &mut context,
            &treasury_mint_keypair.pubkey(),
            &user_token_account.pubkey(),
            &admin_wallet,
            10_000_000,
        )
        .await;

        let new_mint_keypair = Keypair::new();
        create_mint(&mut context, &new_mint_keypair, &payer_pubkey, 0).await;

        let new_mint_token_account = Keypair::new();
        create_token_account(
            &mut context,
            &new_mint_token_account,
            &new_mint_keypair.pubkey(),
            &payer_pubkey,
        )
        .await;

        let payer_keypair = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        mint_to(
            &mut context,
            &new_mint_keypair.pubkey(),
            &new_mint_token_account.pubkey(),
            &payer_keypair,
            1,
        )
        .await;

        let (master_edition_metadata, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                selling_resource.resource.as_ref(),
            ],
            &mpl_token_metadata::id(),
        );

        let (master_edition, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                selling_resource.resource.as_ref(),
                mpl_token_metadata::state::EDITION.as_bytes(),
            ],
            &mpl_token_metadata::id(),
        );

        let (edition_marker, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                selling_resource.resource.as_ref(),
                mpl_token_metadata::state::EDITION.as_bytes(),
                selling_resource.supply.to_string().as_bytes(),
            ],
            &mpl_token_metadata::id(),
        );

        let (new_metadata, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                new_mint_keypair.pubkey().as_ref(),
            ],
            &mpl_token_metadata::id(),
        );

        let (new_edition, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                new_mint_keypair.pubkey().as_ref(),
                mpl_token_metadata::state::EDITION.as_bytes(),
            ],
            &mpl_token_metadata::id(),
        );

        let edition_marker_number = 0;

        // Buy
        let accounts = mpl_fixed_price_sale_accounts::Buy {
            market: market_keypair.pubkey(),
            selling_resource: selling_resource_keypair.pubkey(),
            user_token_account: user_token_account.pubkey(),
            user_wallet: context.payer.pubkey(),
            trade_history,
            treasury_holder: treasury_holder_keypair.pubkey(),
            new_metadata,
            new_edition,
            master_edition,
            new_mint: new_mint_keypair.pubkey(),
            edition_marker,
            vault: selling_resource.vault,
            owner,
            new_token_account: new_mint_token_account.pubkey(),
            master_edition_metadata,
            clock: sysvar::clock::id(),
            rent: sysvar::rent::id(),
            token_metadata_program: mpl_token_metadata::id(),
            token_program: spl_token::id(),
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = mpl_fixed_price_sale_instruction::BuyV2 {
            _trade_history_bump: trade_history_bump,
            vault_owner_bump,
            edition_marker_number,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_fixed_price_sale::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction.clone()],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction_with_commitment(tx, CommitmentLevel::Confirmed)
            .await
            .unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition)
            .await
            .unwrap()
            .unwrap();
        let master_edition_struct =
            MasterEditionV2::safe_deserialize(&master_edition_account.data).unwrap();

        assert_eq!(master_edition_struct.supply, 1);

        /* Burn the edition */
        let ix = burn_edition_nft(
            mpl_token_metadata::ID,
            new_metadata,
            payer_pubkey,
            new_mint_keypair.pubkey(),
            selling_resource.resource,
            new_mint_token_account.pubkey(),
            vault.pubkey(),
            master_edition,
            new_edition,
            edition_marker,
            spl_token::ID,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &payer_keypair],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition)
            .await
            .unwrap()
            .unwrap();
        let master_edition_struct =
            MasterEditionV2::safe_deserialize(&master_edition_account.data).unwrap();

        assert_eq!(master_edition_struct.supply, 0); /* BURN ENDED */

        /* Buy Another */

        let new_mint_keypair = Keypair::new();
        create_mint(&mut context, &new_mint_keypair, &payer_pubkey, 0).await;

        let new_mint_token_account = Keypair::new();
        create_token_account(
            &mut context,
            &new_mint_token_account,
            &new_mint_keypair.pubkey(),
            &payer_pubkey,
        )
        .await;

        let payer_keypair = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        mint_to(
            &mut context,
            &new_mint_keypair.pubkey(),
            &new_mint_token_account.pubkey(),
            &payer_keypair,
            1,
        )
        .await;

        let (new_metadata, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                new_mint_keypair.pubkey().as_ref(),
            ],
            &mpl_token_metadata::id(),
        );

        let (new_edition, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                new_mint_keypair.pubkey().as_ref(),
                mpl_token_metadata::state::EDITION.as_bytes(),
            ],
            &mpl_token_metadata::id(),
        );

        // Buy
        let accounts = mpl_fixed_price_sale_accounts::Buy {
            market: market_keypair.pubkey(),
            selling_resource: selling_resource_keypair.pubkey(),
            user_token_account: user_token_account.pubkey(),
            user_wallet: context.payer.pubkey(),
            trade_history,
            treasury_holder: treasury_holder_keypair.pubkey(),
            new_metadata,
            new_edition,
            master_edition,
            new_mint: new_mint_keypair.pubkey(),
            edition_marker,
            vault: selling_resource.vault,
            owner,
            new_token_account: new_mint_token_account.pubkey(),
            master_edition_metadata,
            clock: sysvar::clock::id(),
            rent: sysvar::rent::id(),
            token_metadata_program: mpl_token_metadata::id(),
            token_program: spl_token::id(),
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = mpl_fixed_price_sale_instruction::BuyV2 {
            _trade_history_bump: trade_history_bump,
            vault_owner_bump,
            edition_marker_number,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_fixed_price_sale::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction.clone()],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction_with_commitment(tx, CommitmentLevel::Confirmed)
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn multiple_marker_pdas() {
        setup_context!(context, mpl_fixed_price_sale, mpl_token_metadata);
        let (admin_wallet, store_keypair) = setup_store(&mut context).await;

        let edition_mint_amount = 500;
        let max_supply = 2 * edition_mint_amount;

        let (selling_resource_keypair, selling_resource_owner_keypair, _vault) =
            setup_selling_resource(
                &mut context,
                &admin_wallet,
                &store_keypair,
                100,
                None,
                true,
                false,
                max_supply,
            )
            .await;

        airdrop(
            &mut context,
            &selling_resource_owner_keypair.pubkey(),
            10_000_000_000_000,
        )
        .await;

        let market_keypair = Keypair::new();

        let treasury_mint_keypair = Keypair::new();
        create_mint(
            &mut context,
            &treasury_mint_keypair,
            &admin_wallet.pubkey(),
            0,
        )
        .await;

        let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
            &treasury_mint_keypair.pubkey(),
            &selling_resource_keypair.pubkey(),
        );

        let treasury_holder_keypair = Keypair::new();
        create_token_account(
            &mut context,
            &treasury_holder_keypair,
            &treasury_mint_keypair.pubkey(),
            &treasury_owner,
        )
        .await;

        let start_date = context
            .banks_client
            .get_sysvar::<Clock>()
            .await
            .unwrap()
            .unix_timestamp
            + 1;

        let name = "Marktname".to_string();
        let description = "Marktbeschreibung".to_string();
        let mutable = true;
        let price = 1_000;
        let pieces_in_one_wallet = Some(edition_mint_amount);

        // CreateMarket
        let accounts = mpl_fixed_price_sale_accounts::CreateMarket {
            market: market_keypair.pubkey(),
            store: store_keypair.pubkey(),
            selling_resource_owner: selling_resource_owner_keypair.pubkey(),
            selling_resource: selling_resource_keypair.pubkey(),
            mint: treasury_mint_keypair.pubkey(),
            treasury_holder: treasury_holder_keypair.pubkey(),
            owner: treasury_owner,
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = mpl_fixed_price_sale_instruction::CreateMarket {
            _treasury_owner_bump: treasyry_owner_bump,
            name: name.to_owned(),
            description: description.to_owned(),
            mutable,
            price,
            pieces_in_one_wallet,
            start_date: start_date as u64,
            end_date: None,
            gating_config: None,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_fixed_price_sale::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &market_keypair,
                &selling_resource_owner_keypair,
            ],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction_with_commitment(tx, CommitmentLevel::Confirmed)
            .await
            .unwrap();

        let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
        context.warp_to_slot(clock.slot + 1500).unwrap();

        // Buy setup
        let mut buy_manager = BuyManager {
            context: &mut context,
            selling_resource_keypair,
            selling_resource: None,
            market_keypair,
            treasury_mint_keypair,
            treasury_holder_keypair,
            admin_wallet,
            user_token_account: None,
            trade_history: None,
            trade_history_bump: None,
            vault_owner_bump: None,
            vault_owner: None,
        };

        buy_setup(&mut buy_manager).await.unwrap();

        for i in 1..=edition_mint_amount {
            let edition_marker_number = i / 248;
            buy_one_v2(&mut buy_manager, edition_marker_number)
                .await
                .unwrap();
            if i % 5 == 0 {
                let slot = buy_manager
                    .context
                    .banks_client
                    .get_root_slot()
                    .await
                    .unwrap();
                buy_manager.context.warp_to_slot(slot + 100).unwrap();
            }
        }

        let clock = buy_manager
            .context
            .banks_client
            .get_sysvar::<Clock>()
            .await
            .unwrap();
        buy_manager.context.warp_to_slot(clock.slot + 3).unwrap();

        // Checks
        let selling_resource_acc = buy_manager
            .context
            .banks_client
            .get_account(buy_manager.selling_resource_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();
        let selling_resource_data =
            SellingResource::try_deserialize(&mut selling_resource_acc.data.as_ref()).unwrap();

        let (trade_history, _) = find_trade_history_address(
            &buy_manager.context.payer.pubkey(),
            &buy_manager.market_keypair.pubkey(),
        );

        let trade_history_acc = buy_manager
            .context
            .banks_client
            .get_account(trade_history)
            .await
            .unwrap()
            .unwrap();
        let trade_history_data =
            TradeHistory::try_deserialize(&mut trade_history_acc.data.as_ref()).unwrap();

        assert_eq!(selling_resource_data.supply, edition_mint_amount);
        assert_eq!(trade_history_data.already_bought, edition_mint_amount);
    }
}
