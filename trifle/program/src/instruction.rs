use crate::state::escrow_constraints::EscrowConstraint;
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

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

pub fn create_escrow_constraint_model_account(
    program_id: &Pubkey,
    escrow_constraint_model: &Pubkey,
    payer: &Pubkey,
    update_authority: &Pubkey,
    name: String,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*escrow_constraint_model, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(*update_authority, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: CreateEscrowConstraintModelAccountArgs { name }
            .try_to_vec()
            .unwrap(),
    }
}

// TODO: make the args more approachable for clients.
pub fn add_constraint_to_escrow_constraint_model(
    program_id: &Pubkey,
    escrow_constraint_model: &Pubkey,
    payer: &Pubkey,
    update_authority: &Pubkey,
    constraint: EscrowConstraint,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*escrow_constraint_model, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(*update_authority, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: AddConstraintToEscrowConstraintModelArgs { constraint }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn create_trifle_account(
    program_id: &Pubkey,
    escrow: &Pubkey,
    metadata: &Pubkey,
    mint: &Pubkey,
    token_account: &Pubkey,
    edition: &Pubkey,
    trifle_account: &Pubkey,
    trifle_authority: &Pubkey,
    escrow_constraint_model: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*escrow, false),
        AccountMeta::new_readonly(*metadata, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(*token_account, false),
        AccountMeta::new_readonly(*edition, false),
        AccountMeta::new(*trifle_account, false),
        AccountMeta::new_readonly(*trifle_authority, false),
        AccountMeta::new_readonly(*escrow_constraint_model, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: vec![],
    }
}
