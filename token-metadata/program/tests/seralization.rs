#![cfg(feature = "test-bpf")]
mod utils;

use mpl_token_metadata::state::MasterEditionV2 as ProgramME;
use mpl_token_metadata::{error::MetadataError, instruction, state::Key};
use mpl_token_metadata::{state::MAX_MASTER_EDITION_LEN, utils::try_from_slice_checked};
use num_traits::FromPrimitive;
use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use utils::*;
mod serialization {

    use super::*;

    async fn setup(context: &mut ProgramTestContext) -> (Vec<u8>, Vec<u8>) {
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);

        test_metadata
            .create(
                context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
            )
            .await
            .unwrap();

        let mint = get_mint(context, &test_master_edition.mint_pubkey).await;

        test_master_edition.create(context, Some(10)).await.unwrap();

        let account = get_account(context, &test_metadata.pubkey).await;
        let me_account = get_account(context, &test_master_edition.pubkey).await;
        return (account.data, me_account.data);
    }
    #[tokio::test]
    async fn success() {
        let mut context = program_test().start_with_context().await;
        let (nft, master) = setup(&mut context).await;
        let otherbytes = master.clone();
        let me: ProgramME = try_from_slice_unchecked(&master).unwrap();
        let me2: ProgramME =
            try_from_slice_checked(&otherbytes, Key::MasterEditionV2, MAX_MASTER_EDITION_LEN)
                .unwrap();
        let me2: ProgramME =
            try_from_slice_checked(&otherbytes, Key::MasterEditionV2, MAX_MASTER_EDITION_LEN)
                .unwrap();
    }
}
