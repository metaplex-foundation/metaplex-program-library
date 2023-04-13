use mpl_token_auth_rules::{
    instruction::{builders::CreateOrUpdateBuilder, CreateOrUpdateArgs, InstructionBuilder},
    payload::Payload,
    state::{CompareOp, Rule, RuleSetV1},
};
use mpl_token_metadata::{
    processor::{AuthorizationData, DelegateScenario, TransferScenario},
    state::{Operation, PayloadKey, TokenDelegateRole},
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::*;

static PROGRAM_ALLOW_LIST: [Pubkey; 2] = [mpl_token_auth_rules::ID, rooster::ID];

macro_rules! get_primitive_rules {
    (
        $dest_program_allow_list:ident,
        $source_program_allow_list:ident,
        $authority_program_allow_list:ident,
        $nft_amount:ident,
        $delegate_program_allow_list:ident,
    ) => {
        let $dest_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Destination.to_string(),
        };

        let $source_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Source.to_string(),
        };

        let $authority_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Authority.to_string(),
        };

        let $nft_amount = Rule::Amount {
            field: PayloadKey::Amount.to_string(),
            amount: 1,
            operator: CompareOp::Eq,
        };

        let $delegate_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Delegate.to_string(),
        };
    };
}

pub async fn create_default_metaplex_rule_set(
    context: &mut ProgramTestContext,
    creator: Keypair,
    use_delegate_allow_list: bool,
) -> (Pubkey, AuthorizationData) {
    let name = String::from("Metaplex Royalty Enforcement");
    let (ruleset_addr, _ruleset_bump) =
        mpl_token_auth_rules::pda::find_rule_set_address(creator.pubkey(), name.clone());

    get_primitive_rules!(
        dest_program_allow_list,
        source_program_allow_list,
        authority_program_allow_list,
        nft_amount,
        delegate_program_allow_list,
    );

    // amount is 1 && (
    //     source owner on allow list
    //  || dest owner on allow list
    //  || authority owner on allow list
    // )
    let transfer_rule = Rule::All {
        rules: vec![
            nft_amount.clone(),
            Rule::Any {
                rules: vec![
                    source_program_allow_list,
                    dest_program_allow_list,
                    authority_program_allow_list,
                ],
            },
        ],
    };

    let delegate_rule = Rule::All {
        rules: vec![delegate_program_allow_list, nft_amount],
    };

    // operations

    let owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    let transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    let sale_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::SaleDelegate,
    };

    let delegate_sale_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::Sale),
    };

    let delegate_lockedtransfer_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::LockedTransfer),
    };

    let delegate_transfer_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::Transfer),
    };

    let delegate_utility_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::Utility),
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

    if use_delegate_allow_list {
        royalty_rule_set
            .add(delegate_sale_operation.to_string(), delegate_rule.clone())
            .unwrap();
        royalty_rule_set
            .add(
                delegate_lockedtransfer_operation.to_string(),
                delegate_rule.clone(),
            )
            .unwrap();
        royalty_rule_set
            .add(
                delegate_transfer_operation.to_string(),
                delegate_rule.clone(),
            )
            .unwrap();
        royalty_rule_set
            .add(delegate_utility_operation.to_string(), delegate_rule)
            .unwrap();
    } else {
        royalty_rule_set
            .add(delegate_sale_operation.to_string(), Rule::Pass)
            .unwrap();
        royalty_rule_set
            .add(delegate_lockedtransfer_operation.to_string(), Rule::Pass)
            .unwrap();
        royalty_rule_set
            .add(delegate_transfer_operation.to_string(), Rule::Pass)
            .unwrap();
        royalty_rule_set
            .add(delegate_utility_operation.to_string(), Rule::Pass)
            .unwrap();
    }

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
