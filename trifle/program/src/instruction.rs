use crate::state::escrow_constraints::EscrowConstraint;
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

    /// Creates a Trifle Account -- used to model token inventory in a Token Escrow account.
    #[account(0, writable, name = "escrow", desc = "Escrow account")]
    #[account(1, name = "metadata", desc = "Metadata account")]
    #[account(2, name = "mint", desc = "Mint account")]
    #[account(
        3,
        writable,
        name = "token_account",
        desc = "Token account (base token)"
    )]
    #[account(4, name = "edition", desc = "Edition account")]
    #[account(5, writable, name = "trifle_account", desc = "Trifle account")]
    #[account(
        6,
        signer,
        name = "trifle_authority",
        desc = "Trifle Authority - the account that can sign transactions for the trifle account"
    )]
    #[account(7, name = "escrow_constraint_model", desc = "Escrow constraint model")]
    #[account(
        8,
        writable,
        signer,
        name = "payer",
        desc = "Wallet paying for the transaction"
    )]
    #[account(9, name = "system_program", desc = "System program")]
    CreateTrifleAccount,
}
