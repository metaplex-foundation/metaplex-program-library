use crate::state::EscrowConstraint;
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CreateEscrowConstraintModelAccountArgs {
    pub name: String,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct AddConstraintToEscrowConstraintModelArgs {
    pub constraint: EscrowConstraint,
}

#[derive(ShankInstruction, Debug, BorshDeserialize)]
pub enum TrifleInstruction {
    /// Create an constraint model to be used by one or many escrow accounts.
    #[account(
        0,
        writable,
        name = "escrow_constraint_model",
        desc = "Constraint model account"
    )]
    #[account(
        1,
        writable,
        signer,
        name = "payer",
        desc = "Wallet paying for the transaction and new account, will be set as the creator of the constraint model"
    )]
    #[account(
        2,
        name = "update_authority",
        desc = "Update authority of the constraint model"
    )]
    #[account(3, name = "system_program", desc = "System program")]
    #[account(4, name = "rent", desc = "Rent info")]
    CreateEscrowConstraintModelAccount(CreateEscrowConstraintModelAccountArgs),

    /// Add a constraint to an escrow constraint model.
    #[account(
        0,
        writable,
        name = "escrow_constraint_model",
        desc = "Constraint model account"
    )]
    #[account(
        1,
        writable,
        signer,
        name = "payer",
        desc = "Wallet paying for the transaction and new account, will be set as the creator of the constraint model"
    )]
    #[account(
        2,
        signer,
        name = "update_authority",
        desc = "Update authority of the constraint model"
    )]
    #[account(3, name = "system_program", desc = "System program")]
    AddConstraintToEscrowConstraintModel(AddConstraintToEscrowConstraintModelArgs),
}
