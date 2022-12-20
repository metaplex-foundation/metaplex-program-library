use mpl_token_auth_rules::{
    payload::{Payload, PayloadKey},
    state::{Rule, RuleSet},
};
use mpl_token_metadata::{processor::AuthorizationData, state::Operation};
use num_traits::ToPrimitive;
use rmp_serde::Serializer;
use serde::Serialize;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::*;

pub async fn create_test_ruleset(
    context: &mut ProgramTestContext,
    payer: Keypair,
    name: String,
) -> (Pubkey, AuthorizationData) {
    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Find RuleSet PDA.
    let (ruleset_addr, _ruleset_bump) =
        mpl_token_auth_rules::pda::find_rule_set_address(payer.pubkey(), name.clone());

    // Rule for Transfers: Allow transfers to a Token Owned Escrow account.
    let owned_by_token_metadata = Rule::ProgramOwned {
        program: mpl_token_metadata::id(),
        field: PayloadKey::Target,
    };

    // Merkle tree root generated in a different test program.
    let marketplace_tree_root: [u8; 32] = [
        132, 141, 27, 31, 23, 154, 145, 128, 32, 62, 122, 224, 248, 128, 37, 139, 200, 46, 163,
        238, 76, 123, 155, 141, 73, 12, 111, 192, 122, 80, 126, 155,
    ];

    // Rule for Delegate and SaleTransfer: The provided leaf node must be a
    // member of the marketplace Merkle tree.
    let leaf_in_marketplace_tree = Rule::PubkeyTreeMatch {
        root: marketplace_tree_root,
        field: PayloadKey::Target,
    };

    // Create Basic Royalty Enforcement Ruleset.
    let mut basic_royalty_enforcement_rule_set = RuleSet::new(name, payer.pubkey());
    basic_royalty_enforcement_rule_set
        .add(
            Operation::Transfer.to_u16().unwrap(),
            owned_by_token_metadata,
        )
        .unwrap();
    basic_royalty_enforcement_rule_set
        .add(
            Operation::Delegate.to_u16().unwrap(),
            leaf_in_marketplace_tree.clone(),
        )
        .unwrap();
    basic_royalty_enforcement_rule_set
        .add(
            Operation::DelegatedTransfer.to_u16().unwrap(),
            leaf_in_marketplace_tree,
        )
        .unwrap();

    // Serialize the RuleSet using RMP serde.
    let mut serialized_data = Vec::new();
    basic_royalty_enforcement_rule_set
        .serialize(&mut Serializer::new(&mut serialized_data))
        .unwrap();

    // Create a `create` instruction.
    let create_ix = mpl_token_auth_rules::instruction::create(
        mpl_token_auth_rules::ID,
        payer.pubkey(),
        ruleset_addr,
        serialized_data,
        vec![],
    );

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&payer.pubkey()),
        &[&payer],
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect("creation should succeed");

    // Client can add additional rules to the Payload but does not need to in this case.
    let payload = Payload::new();

    let auth_data = AuthorizationData { payload };

    (ruleset_addr, auth_data)
}
