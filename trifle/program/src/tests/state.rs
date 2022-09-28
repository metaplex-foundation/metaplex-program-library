#[cfg(test)]
mod escrow {
    use std::collections::HashMap;

    use crate::{
        error::TrifleError,
        state::{
            escrow_constraints::{EscrowConstraint, EscrowConstraintModel, EscrowConstraintType},
            trifle::Trifle,
            Key,
        },
    };
    use borsh::BorshSerialize;
    use solana_sdk::{signature::Keypair, signer::Signer};

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
            constraint_type: ect_none,
            token_limit: 1,
        };

        let escrow_constraint_collection = EscrowConstraint {
            constraint_type: ect_collection,
            token_limit: 1,
        };

        let escrow_constraint_tokens = EscrowConstraint {
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

        let mut constraints = HashMap::new();
        constraints.insert("test1".to_string(), escrow_constraint_none);
        constraints.insert("test2".to_string(), escrow_constraint_collection);
        constraints.insert("test3".to_string(), escrow_constraint_tokens);

        let escrow_constraints_model = EscrowConstraintModel {
            key: Key::EscrowConstraintModel,
            name: "test".to_string(),
            count: 0,
            update_authority: Keypair::new().pubkey(),
            creator: Keypair::new().pubkey(),
            constraints,
            schema_uri: None,
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
            constraint_type: EscrowConstraintType::None,
            token_limit: 1,
        };

        let ec_none_unlimited = EscrowConstraint {
            constraint_type: EscrowConstraintType::None,
            token_limit: 0,
        };

        let ec_collection = EscrowConstraint {
            constraint_type: EscrowConstraintType::Collection(keypair_1.pubkey()),
            token_limit: 1,
        };

        let ec_tokens = EscrowConstraint {
            constraint_type: EscrowConstraintType::tokens_from_slice(&[
                keypair_2.pubkey(),
                keypair_3.pubkey(),
            ]),
            token_limit: 10,
        };

        let mut constraints = HashMap::new();
        constraints.insert("none".to_string(), ec_none);
        constraints.insert("none_unlimited".to_string(), ec_none_unlimited);
        constraints.insert("collection".to_string(), ec_collection);
        constraints.insert("tokens".to_string(), ec_tokens.clone());

        let escrow_constraints_model = EscrowConstraintModel {
            key: Key::EscrowConstraintModel,
            name: "test".to_string(),
            count: 0,
            update_authority: Keypair::new().pubkey(),
            creator: Keypair::new().pubkey(),
            constraints,
            schema_uri: Some("test".to_string()),
        };

        escrow_constraints_model
            .validate(&keypair_1.pubkey(), &"none".to_string())
            .expect("None constraint failed");

        escrow_constraints_model
            .validate(&keypair_1.pubkey(), &"none_unlimited".to_string())
            .expect("None constraint failed");

        escrow_constraints_model
            .validate(&keypair_1.pubkey(), &"collection".to_string())
            .expect("Collection constraint failed");

        escrow_constraints_model
            .validate(&keypair_2.pubkey(), &"collection".to_string())
            .expect_err("Collection constraint failed");

        escrow_constraints_model
            .validate(&keypair_2.pubkey(), &"tokens".to_string())
            .expect("Tokens constraint failed");

        escrow_constraints_model
            .validate(&keypair_1.pubkey(), &"tokens".to_string())
            .expect_err("Tokens constraint failed");

        let mut trifle = Trifle {
            ..Default::default()
        };

        // EC::None limit 1
        assert_eq!(
            trifle.try_add(
                &escrow_constraints_model,
                "none".to_string(),
                keypair_1.pubkey(),
                1
            ),
            Ok(())
        );
        assert_eq!(
            trifle.try_add(
                &escrow_constraints_model,
                "none".to_string(),
                keypair_1.pubkey(),
                1
            ),
            Err(TrifleError::TokenLimitExceeded)
        );

        // EC::None unlimited
        assert_eq!(
            trifle.try_add(
                &escrow_constraints_model,
                "none_unlimited".to_string(),
                keypair_1.pubkey(),
                1
            ),
            Ok(())
        );
        assert_eq!(
            trifle.try_add(
                &escrow_constraints_model,
                "none_unlimited".to_string(),
                keypair_1.pubkey(),
                1
            ),
            Ok(())
        );

        assert_eq!(
            trifle.try_add(
                &escrow_constraints_model,
                "tokens".to_string(),
                keypair_1.pubkey(),
                5
            ),
            Err(TrifleError::EscrowConstraintViolation)
        );

        // limit is 10
        assert_eq!(
            trifle.try_add(
                &escrow_constraints_model,
                "tokens".to_string(),
                keypair_2.pubkey(),
                5
            ),
            Ok(())
        );
        assert_eq!(
            trifle.try_add(
                &escrow_constraints_model,
                "tokens".to_string(),
                keypair_3.pubkey(),
                5
            ),
            Ok(())
        );
        assert_eq!(
            trifle.try_add(
                &escrow_constraints_model,
                "tokens".to_string(),
                keypair_3.pubkey(),
                5
            ),
            Err(TrifleError::TokenLimitExceeded)
        );
    }
}
