#![cfg(feature = "test-bpf")]
pub mod utils;

use solana_program_test::*;
use utils::*;

mod utility {

    use mpl_token_metadata::{
        instruction::DelegateArgs,
        pda::find_token_record_account,
        state::{TokenDelegateRole, TokenRecord, TokenStandard, TokenState},
    };
    use solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack, pubkey::Pubkey};
    use solana_sdk::signature::{Keypair, Signer};
    use spl_token::state::Account;

    use super::*;

    #[tokio::test]
    async fn delegate_unlock_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.state, TokenState::Unlocked);

        // set a utility delegate

        let delegate = Keypair::new();
        let delegate_pubkey = delegate.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                delegate_pubkey,
                DelegateArgs::UtilityV1 {
                    amount: 1,
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // lock

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let token_delegate = Keypair::from_bytes(&delegate.to_bytes()).unwrap();

        asset
            .lock(&mut context, token_delegate, Some(pda_key), payer)
            .await
            .unwrap();

        // asserts

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.state, TokenState::Locked);

        // unlock

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let token_delegate = Keypair::from_bytes(&delegate.to_bytes()).unwrap();

        asset
            .unlock(&mut context, token_delegate, Some(pda_key), payer)
            .await
            .unwrap();

        // asserts

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.state, TokenState::Unlocked);
    }

    #[tokio::test]
    async fn delegate_unlock_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(&mut context, TokenStandard::NonFungible, None, None, 1)
            .await
            .unwrap();

        // asserts

        let token_account = get_account(&mut context, &asset.token.unwrap()).await;
        let token = Account::unpack(&token_account.data).unwrap();
        // should NOT be frozen
        assert!(!token.is_frozen());

        // set a utility delegate

        let delegate = Keypair::new();
        let delegate_pubkey = delegate.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                delegate_pubkey,
                DelegateArgs::StandardV1 { amount: 1 },
            )
            .await
            .unwrap();

        // lock

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let token_delegate = Keypair::from_bytes(&delegate.to_bytes()).unwrap();

        asset
            .lock(&mut context, token_delegate, None, payer)
            .await
            .unwrap();

        let token_account = get_account(&mut context, &asset.token.unwrap()).await;
        let token = Account::unpack(&token_account.data).unwrap();
        // should be frozen
        assert!(token.is_frozen());

        // unlock

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let token_delegate = Keypair::from_bytes(&delegate.to_bytes()).unwrap();

        asset
            .unlock(&mut context, token_delegate, None, payer)
            .await
            .unwrap();

        let token_account = get_account(&mut context, &asset.token.unwrap()).await;
        let token = Account::unpack(&token_account.data).unwrap();
        // should NOT be frozen
        assert!(!token.is_frozen());
    }

    #[tokio::test]
    async fn locked_transfer_delegate_unlock_programmable_nonfungible() {
        let mut context = program_test().start_with_context().await;

        // asset

        let mut asset = DigitalAsset::default();
        asset
            .create_and_mint(
                &mut context,
                TokenStandard::ProgrammableNonFungible,
                None,
                None,
                1,
            )
            .await
            .unwrap();

        // asserts

        let (pda_key, _) = find_token_record_account(&asset.mint.pubkey(), &asset.token.unwrap());

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.state, TokenState::Unlocked);

        // set a locked transfer delegate

        let delegate = Keypair::new();
        let delegate_pubkey = delegate.pubkey();
        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();

        asset
            .delegate(
                &mut context,
                payer,
                delegate_pubkey,
                DelegateArgs::LockedTransferV1 {
                    amount: 1,
                    locked_address: Pubkey::default(),
                    authorization_data: None,
                },
            )
            .await
            .unwrap();

        // lock

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let token_delegate = Keypair::from_bytes(&delegate.to_bytes()).unwrap();

        asset
            .lock(&mut context, token_delegate, Some(pda_key), payer)
            .await
            .unwrap();

        // asserts

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.state, TokenState::Locked);
        assert_eq!(token_record.locked_transfer, Some(Pubkey::default()));

        // unlock

        let payer = Keypair::from_bytes(&context.payer.to_bytes()).unwrap();
        let token_delegate = Keypair::from_bytes(&delegate.to_bytes()).unwrap();

        asset
            .unlock(&mut context, token_delegate, Some(pda_key), payer)
            .await
            .unwrap();

        // asserts

        let pda = get_account(&mut context, &pda_key).await;
        let token_record: TokenRecord = try_from_slice_unchecked(&pda.data).unwrap();

        assert_eq!(token_record.state, TokenState::Unlocked);
        assert_eq!(
            token_record.delegate_role,
            Some(TokenDelegateRole::LockedTransfer)
        );
        assert_eq!(token_record.locked_transfer, Some(Pubkey::default()));
    }
}
