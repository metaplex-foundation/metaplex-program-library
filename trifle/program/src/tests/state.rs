use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_sdk::{signature::Keypair, signer::Signer};

mod escrow {
    use crate::state::{EscrowConstraint, EscrowConstraintModel, EscrowConstraintType, Key};

    use super::*;

    #[test]
    fn test_escrow_constraints_model_len() {
        let ect_none = EscrowConstraintType::None;
        let ect_collection = EscrowConstraintType::Collection(Keypair::new().pubkey());
        let ect_tokens = EscrowConstraintType::tokens_from_slice(&[
            Keypair::new().pubkey(),
            Keypair::new().pubkey(),
            Keypair::new().pubkey(),
            Keypair::new().pubkey(),
        ]);

        let mut buf_ect_none = Vec::new();
        let mut buf_ect_collection = Vec::new();
        let mut buf_ect_tokens = Vec::new();

        ect_none.serialize(&mut buf_ect_none).unwrap();
        ect_collection.serialize(&mut buf_ect_collection).unwrap();
        ect_tokens.serialize(&mut buf_ect_tokens).unwrap();

        assert_eq!(
            ect_none.try_len().unwrap(),
            buf_ect_none.len(),
            "EscrowConstraintType::None length is not equal to serialized length"
        );

        assert_eq!(
            ect_collection.try_len().unwrap(),
            buf_ect_collection.len(),
            "EscrowConstraintType::Collection length is not equal to serialized length"
        );

        assert_eq!(
            ect_tokens.try_len().unwrap(),
            buf_ect_tokens.len(),
            "EscrowConstraintType::tokens length is not equal to serialized length"
        );

        let escrow_constraint_none = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: ect_none,
            token_limit: 1,
        };

        let escrow_constraint_collection = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: ect_collection,
            token_limit: 1,
        };

        let escrow_constraint_tokens = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: ect_tokens,
            token_limit: 1,
        };

        let mut buf_escrow_constraint_none = Vec::new();
        let mut buf_escrow_constraint_collection = Vec::new();
        let mut buf_escrow_constraint_tokens = Vec::new();

        escrow_constraint_none
            .serialize(&mut buf_escrow_constraint_none)
            .unwrap();

        escrow_constraint_collection
            .serialize(&mut buf_escrow_constraint_collection)
            .unwrap();

        escrow_constraint_tokens
            .serialize(&mut buf_escrow_constraint_tokens)
            .unwrap();

        assert_eq!(
            escrow_constraint_none.try_len().unwrap(),
            buf_escrow_constraint_none.len(),
            "EscrowConstraint::None length is not equal to serialized length"
        );

        assert_eq!(
            escrow_constraint_collection.try_len().unwrap(),
            buf_escrow_constraint_collection.len(),
            "EscrowConstraint::Collection length is not equal to serialized length"
        );

        assert_eq!(
            escrow_constraint_tokens.try_len().unwrap(),
            buf_escrow_constraint_tokens.len(),
            "EscrowConstraint::tokens length is not equal to serialized length"
        );

        let escrow_constraints_model = EscrowConstraintModel {
            key: Key::EscrowConstraintModel,
            name: "test".to_string(),
            count: 0,
            update_authority: Keypair::new().pubkey(),
            creator: Keypair::new().pubkey(),
            constraints: vec![
                escrow_constraint_none,
                escrow_constraint_collection,
                escrow_constraint_tokens,
            ],
        };

        let mut buf_escrow_constraints_model = Vec::new();

        escrow_constraints_model
            .serialize(&mut buf_escrow_constraints_model)
            .unwrap();

        assert_eq!(
            escrow_constraints_model.try_len().unwrap(),
            buf_escrow_constraints_model.len(),
            "EscrowConstraintModel length is not equal to serialized length"
        );
    }

    #[test]
    fn test_validate_constraint() {
        let keypair_1 = Keypair::new();
        let keypair_2 = Keypair::new();
        let keypair_3 = Keypair::new();

        let ec_none = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: EscrowConstraintType::None,
            token_limit: 1,
        };

        let ec_collection = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: EscrowConstraintType::Collection(keypair_1.pubkey()),
            token_limit: 1,
        };

        let ec_tokens = EscrowConstraint {
            name: "test".to_string(),
            constraint_type: EscrowConstraintType::tokens_from_slice(&[
                keypair_2.pubkey(),
                keypair_3.pubkey(),
            ]),

            token_limit: 1,
        };
        let escrow_constraints_model = EscrowConstraintModel {
            key: Key::EscrowConstraintModel,
            name: "test".to_string(),
            count: 0,
            update_authority: Keypair::new().pubkey(),
            creator: Keypair::new().pubkey(),
            constraints: vec![ec_none, ec_collection, ec_tokens],
        };

        escrow_constraints_model
            .validate_at(&keypair_1.pubkey(), 0)
            .expect("None constraint failed");

        escrow_constraints_model
            .validate_at(&keypair_1.pubkey(), 1)
            .expect("Collection constraint failed");

        escrow_constraints_model
            .validate_at(&keypair_2.pubkey(), 1)
            .expect_err("Collection constraint failed");

        escrow_constraints_model
            .validate_at(&keypair_2.pubkey(), 2)
            .expect("Tokens constraint failed");

        escrow_constraints_model
            .validate_at(&keypair_1.pubkey(), 2)
            .expect_err("Tokens constraint failed");
    }
}
