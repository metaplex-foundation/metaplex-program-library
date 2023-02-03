use mpl_token_auth_rules::{
    instruction::{builders::CreateOrUpdateBuilder, CreateOrUpdateArgs, InstructionBuilder},
    payload::Payload,
    state::{CompareOp, Rule, RuleSetV1},
};
use mpl_token_metadata::{
    processor::{AuthorizationData, TransferScenario},
    state::{Operation, PayloadKey},
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::system_program;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::*;

static PROGRAM_ALLOW_LIST: [Pubkey; 3] = [
    mpl_token_auth_rules::ID,
    mpl_token_metadata::ID,
    rooster::ID,
];

macro_rules! get_primitive_rules {
    (
        $source_owned_by_sys_program:ident,
        $dest_program_allow_list:ident,
        $dest_pda_match:ident,
        $source_program_allow_list:ident,
        $source_pda_match:ident,
        $dest_owned_by_sys_program:ident,
        $authority_program_allow_list:ident,
        $authority_pda_match:ident,
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

        let $authority_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Authority.to_string(),
        };
        let $authority_pda_match = Rule::PDAMatch {
            program: None,
            pda_field: PayloadKey::Authority.to_string(),
            seeds_field: PayloadKey::AuthoritySeeds.to_string(),
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
        authority_program_allow_list,
        authority_pda_match,
    );

    // (source is owned by system program && dest is on allow list && destination is a PDA) ||
    // (source is on allow list && source is a PDA && dest is owned by system program)
    let transfer_rule = Rule::Any {
        rules: vec![
            Rule::All {
                rules: vec![
                    // source_owned_by_sys_program,
                    dest_program_allow_list,
                    dest_pda_match,
                    nft_amount.clone(),
                ],
            },
            Rule::All {
                rules: vec![
                    source_program_allow_list,
                    source_pda_match,
                    // dest_owned_by_sys_program,
                    nft_amount.clone(),
                ],
            },
            Rule::All {
                rules: vec![
                    authority_program_allow_list,
                    authority_pda_match,
                    nft_amount,
                ],
            },
        ],
    };

    let owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    let transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    let sale_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::SaleDelegate,
    };

    let locked_transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::LockedTransferDelegate,
    };

    let mut royalty_rule_set = RuleSetV1::new(name, creator.pubkey());
    royalty_rule_set
        .add(owner_operation.to_string(), transfer_rule.clone())
        .unwrap();
    royalty_rule_set
        .add(
            transfer_delegate_operation.to_string(),
            transfer_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(sale_delegate_operation.to_string(), transfer_rule.clone())
        .unwrap();
    royalty_rule_set
        .add(
            locked_transfer_delegate_operation.to_string(),
            transfer_rule.clone(),
        )
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

    let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[compute_ix, create_ix],
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
