use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use mpl_token_metadata::state::{Collection, Creator, Uses};
use solana_program_test::ProgramTestContext;
use solana_sdk::signature::Signer;

use crate::listing_rewards_test::TEN_SOL;

#[derive(Clone, Debug)]
pub struct Params<'a> {
    pub name: &'a str,
    pub symbol: &'a str,
    pub uri: &'a str,
    pub creators: Option<Vec<Creator>>,
    pub seller_fee_basis_points: u16,
    pub is_mutable: bool,
    pub collection: Option<Collection>,
    pub uses: Option<Uses>,
}

pub async fn create<'a>(
    context: &mut ProgramTestContext,
    Params {
        name,
        symbol,
        uri,
        creators,
        seller_fee_basis_points,
        is_mutable,
        collection,
        uses,
    }: Params<'a>,
    airdrop_amount: Option<u64>,
) -> Metadata {
    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();
    let airdrop_amount = airdrop_amount.unwrap_or(TEN_SOL);

    airdrop(context, owner_pubkey, airdrop_amount)
        .await
        .unwrap();

    test_metadata
        .create_v2(
            context,
            name.to_string(),
            symbol.to_string(),
            uri.to_string(),
            creators,
            seller_fee_basis_points,
            is_mutable,
            collection,
            uses,
        )
        .await
        .unwrap();

    test_metadata
}
