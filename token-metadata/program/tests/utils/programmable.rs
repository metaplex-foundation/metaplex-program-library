use mpl_token_auth_rules::{
    instruction::{builders::CreateOrUpdateBuilder, CreateOrUpdateArgs, InstructionBuilder},
    payload::Payload,
    state::{CompareOp, Rule, RuleSet},
};
use mpl_token_metadata::{
    processor::{AuthorizationData, TransferAuthority},
    state::{Operation, PayloadKey},
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::system_program;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::*;

static PROGRAM_ALLOW_LIST: [Pubkey; 1] = [mpl_token_auth_rules::ID];

macro_rules! get_primitive_rules {
    (
        $source_owned_by_sys_program:ident,
        $dest_program_allow_list:ident,
        $dest_pda_match:ident,
        $source_program_allow_list:ident,
        $source_pda_match:ident,
        $dest_owned_by_sys_program:ident,
        $nft_amount:ident,
    ) => {
        let $source_owned_by_sys_program = Rule::ProgramOwned {
            program: system_program::ID,
            field: PayloadKey::Source.to_string(),
        };

        let $dest_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Destination.to_string(),
        };

        let $dest_pda_match = Rule::PDAMatch {
            program: None,
            pda_field: PayloadKey::Destination.to_string(),
            seeds_field: PayloadKey::DestinationSeeds.to_string(),
        };

        let $source_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Source.to_string(),
        };

        let $source_pda_match = Rule::PDAMatch {
            program: None,
            pda_field: PayloadKey::Source.to_string(),
            seeds_field: PayloadKey::SourceSeeds.to_string(),
        };

        let $dest_owned_by_sys_program = Rule::ProgramOwned {
            program: system_program::ID,
            field: PayloadKey::Destination.to_string(),
        };
        let $nft_amount = Rule::Amount {
            field: PayloadKey::Amount.to_string(),
            amount: 1,
            operator: CompareOp::Eq,
        };
    };
}

pub async fn create_default_metaplex_rule_set(
    context: &mut ProgramTestContext,
    creator: Keypair,
) -> (Pubkey, AuthorizationData) {
    let name = String::from("Metaplex Royalty Enforcement");
    let (ruleset_addr, _ruleset_bump) =
        mpl_token_auth_rules::pda::find_rule_set_address(creator.pubkey(), name.clone());

    get_primitive_rules!(
        source_owned_by_sys_program,
        dest_program_allow_list,
        dest_pda_match,
        source_program_allow_list,
        source_pda_match,
        dest_owned_by_sys_program,
        nft_amount,
    );

    // (source is owned by system program && dest is on allow list && destination is a PDA) ||
    // (source is on allow list && source is a PDA && dest is owned by system program)
    let transfer_rule = Rule::Any {
        rules: vec![
            Rule::All {
                rules: vec![
                    source_owned_by_sys_program,
                    dest_program_allow_list,
                    dest_pda_match,
                    nft_amount.clone(),
                ],
            },
            Rule::All {
                rules: vec![
                    source_program_allow_list,
                    source_pda_match,
                    dest_owned_by_sys_program,
                    nft_amount,
                ],
            },
        ],
    };

    let operation = Operation::Transfer {
        scenario: TransferAuthority::Owner,
    };

    let mut royalty_rule_set = RuleSet::new(name, creator.pubkey());
    royalty_rule_set
        .add(operation.to_string(), transfer_rule)
        .unwrap();

    // Serialize the RuleSet using RMP serde.
    let mut serialized_data = Vec::new();
    royalty_rule_set
        .serialize(&mut Serializer::new(&mut serialized_data))
        .unwrap();

    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .rule_set_pda(ruleset_addr)
        .payer(creator.pubkey())
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set: serialized_data,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&creator.pubkey()),
        &[&creator],
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

// pub async fn create_test_ruleset(
//     context: &mut ProgramTestContext,
//     payer: Keypair,
//     name: String,
// ) -> (Pubkey, AuthorizationData) {
//     // --------------------------------
//     // Create RuleSet
//     // --------------------------------
//     // Find RuleSet PDA.
//     let (ruleset_addr, _ruleset_bump) =
//         mpl_token_auth_rules::pda::find_rule_set_address(payer.pubkey(), name.clone());

//     // Rule for Transfers: Allow transfers to a Token Owned Escrow account.
//     let owned_by_token_metadata = Rule::ProgramOwned {
//         program: mpl_token_metadata::ID,
//         field: PayloadKey::Destination.to_string(),
//     };

//     // Merkle tree root generated in a different test program.
//     let marketplace_tree_root: [u8; 32] = [
//         132, 141, 27, 31, 23, 154, 145, 128, 32, 62, 122, 224, 248, 128, 37, 139, 200, 46, 163,
//         238, 76, 123, 155, 141, 73, 12, 111, 192, 122, 80, 126, 155,
//     ];

//     // Rule for Delegate and SaleTransfer: The provided leaf node must be a
//     // member of the marketplace Merkle tree.
//     let leaf_in_marketplace_tree = Rule::PubkeyTreeMatch {
//         root: marketplace_tree_root,
//         field: PayloadKey::Destination.to_string(),
//     };

//     // Create Basic Royalty Enforcement Ruleset.
//     let mut basic_royalty_enforcement_rule_set = RuleSet::new(name, payer.pubkey());
//     basic_royalty_enforcement_rule_set
//         .add(Operation::Transfer.to_string(), owned_by_token_metadata)
//         .unwrap();
//     basic_royalty_enforcement_rule_set
//         .add(
//             Operation::Delegate.to_string(),
//             leaf_in_marketplace_tree.clone(),
//         )
//         .unwrap();
//     basic_royalty_enforcement_rule_set
//         .add(Operation::Sale.to_string(), leaf_in_marketplace_tree)
//         .unwrap();

//     // Serialize the RuleSet using RMP serde.
//     let mut serialized_data = Vec::new();
//     basic_royalty_enforcement_rule_set
//         .serialize(&mut Serializer::new(&mut serialized_data))
//         .unwrap();

//     // Create a `create` instruction.
//     let create_ix = CreateOrUpdateBuilder::new()
//         .rule_set_pda(ruleset_addr)
//         .payer(payer.pubkey())
//         .build(CreateOrUpdateArgs::V1 {
//             serialized_rule_set: serialized_data,
//         })
//         .unwrap()
//         .instruction();

//     // Add it to a transaction.
//     let create_tx = Transaction::new_signed_with_payer(
//         &[create_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         context.last_blockhash,
//     );

//     // Process the transaction.
//     context
//         .banks_client
//         .process_transaction(create_tx)
//         .await
//         .expect("creation should succeed");

//     // Client can add additional rules to the Payload but does not need to in this case.
//     let payload = Payload::new();

//     let auth_data = AuthorizationData { payload };

//     (ruleset_addr, auth_data)
// }
