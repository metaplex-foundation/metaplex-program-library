#[cfg(test)]
mod escrow {
    use std::collections::HashMap;

    use crate::{
        error::TrifleError,
        state::{
            escrow_constraints::{EscrowConstraint, EscrowConstraintModel, EscrowConstraintType},
            transfer_effects::TransferEffects,
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

        let escrow_constraint_none = EscrowConstraint {
            constraint_type: ect_none,
            token_limit: 1,
            transfer_effects: TransferEffects::default().into(),
        };

        let escrow_constraint_collection = EscrowConstraint {
            constraint_type: ect_collection,
            token_limit: 1,
            transfer_effects: TransferEffects::default().into(),
        };

        let escrow_constraint_tokens = EscrowConstraint {
            constraint_type: ect_tokens,
            token_limit: 1,
            transfer_effects: TransferEffects::default().into(),
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
            royalties: HashMap::new(),
            royalty_balance: 0,
            padding: [0; 32],
        };

        let mut buf_escrow_constraints_model = Vec::new();

        escrow_constraints_model
            .serialize(&mut buf_escrow_constraints_model)
            .unwrap();
    }

    #[test]
    fn test_validate_constraint() {
        let keypair_1 = Keypair::new();
        let keypair_2 = Keypair::new();
        let keypair_3 = Keypair::new();

        let ec_none = EscrowConstraint {
            constraint_type: EscrowConstraintType::None,
            token_limit: 1,
            transfer_effects: TransferEffects::default().into(),
        };

        let ec_none_unlimited = EscrowConstraint {
            constraint_type: EscrowConstraintType::None,
            token_limit: 0,
            transfer_effects: TransferEffects::default().into(),
        };

        let ec_collection = EscrowConstraint {
            constraint_type: EscrowConstraintType::Collection(keypair_1.pubkey()),
            token_limit: 1,
            transfer_effects: TransferEffects::default().into(),
        };

        let ec_tokens = EscrowConstraint {
            constraint_type: EscrowConstraintType::tokens_from_slice(&[
                keypair_2.pubkey(),
                keypair_3.pubkey(),
            ]),
            token_limit: 10,
            transfer_effects: TransferEffects::default().into(),
        };

        let mut constraints = HashMap::new();
        constraints.insert("none".to_string(), ec_none.clone());
        constraints.insert("none_unlimited".to_string(), ec_none_unlimited.clone());
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
            royalties: HashMap::new(),
            royalty_balance: 0,
            padding: [0; 32],
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
            trifle.try_add(&ec_none, "none".to_string(), keypair_1.pubkey(), 1),
            Ok(())
        );
        assert_eq!(
            trifle.try_add(&ec_none, "none".to_string(), keypair_1.pubkey(), 1),
            Err(TrifleError::TokenLimitExceeded)
        );

        // EC::None unlimited
        assert_eq!(
            trifle.try_add(
                &ec_none_unlimited,
                "none_unlimited".to_string(),
                keypair_1.pubkey(),
                1
            ),
            Ok(())
        );
        assert_eq!(
            trifle.try_add(
                &ec_none_unlimited,
                "none_unlimited".to_string(),
                keypair_1.pubkey(),
                1
            ),
            Ok(())
        );

        // assert_eq!(
        //     trifle.try_add(&ec_tokens, "tokens".to_string(), keypair_1.pubkey(), 5),
        //     Err(TrifleError::EscrowConstraintViolation)
        // );

        // limit is 10
        assert_eq!(
            trifle.try_add(&ec_tokens, "tokens".to_string(), keypair_2.pubkey(), 5),
            Ok(())
        );
        assert_eq!(
            trifle.try_add(&ec_tokens, "tokens".to_string(), keypair_3.pubkey(), 5),
            Ok(())
        );
        assert_eq!(
            trifle.try_add(&ec_tokens, "tokens".to_string(), keypair_3.pubkey(), 5),
            Err(TrifleError::TokenLimitExceeded)
        );
    }
}
