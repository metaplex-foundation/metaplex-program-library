use std::collections::HashMap;

use mpl_token_auth_rules::{
    payload::PayloadKey,
    state::{Operation, Rule, RuleSet},
};
use mpl_token_metadata::processor::AuthorizationData;
use rmp_serde::Serializer;
use serde::Serialize;
use solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};

use crate::*;

pub async fn create_royalty_ruleset(
    context: &mut ProgramTestContext,
) -> (Pubkey, AuthorizationData) {
    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Find RuleSet PDA.
    let name = "basic_royalty_enforcement".to_string();

    let (ruleset_addr, _ruleset_bump) =
        mpl_token_auth_rules::pda::find_rule_set_address(context.payer.pubkey(), name.clone());

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
    let mut basic_royalty_enforcement_rule_set = RuleSet::new();
    basic_royalty_enforcement_rule_set.add(Operation::Transfer, owned_by_token_metadata);
    basic_royalty_enforcement_rule_set.add(Operation::Delegate, leaf_in_marketplace_tree.clone());
    basic_royalty_enforcement_rule_set.add(Operation::SaleTransfer, leaf_in_marketplace_tree);

    // Serialize the RuleSet using RMP serde.
    let mut serialized_data = Vec::new();
    basic_royalty_enforcement_rule_set
        .serialize(&mut Serializer::new(&mut serialized_data))
        .unwrap();

    // Create a `create` instruction.
    let create_ix = mpl_token_auth_rules::instruction::create(
        mpl_token_auth_rules::id(),
        context.payer.pubkey(),
        ruleset_addr,
        name.clone(),
        serialized_data,
    );

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect("creation should succeed");

    // Client can add additional rules to the Payload but does not need to in this case.
    let payload = HashMap::new();

    let auth_data = AuthorizationData { payload, name };

    (ruleset_addr, auth_data)
}
