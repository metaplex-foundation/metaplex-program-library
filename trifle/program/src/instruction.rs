use crate::state::escrow_constraints::EscrowConstraint;
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CreateEscrowConstraintModelAccountArgs {
    pub name: String,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AddConstraintToEscrowConstraintModelArgs {
    pub constraint_name: String,
    pub constraint: EscrowConstraint,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferInArgs {
    pub slot: String,
    pub amount: u64,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransferOutArgs {
    pub slot: String,
    pub amount: u64,
}

#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(ShankInstruction, Debug, BorshSerialize, Clone, BorshDeserialize)]
#[rustfmt::skip]
pub enum TrifleInstruction {
    /// Create an constraint model to be used by one or many escrow accounts.
    #[account(0, writable, name = "escrow_constraint_model", desc = "Constraint model account")]
    #[account(1, writable, signer, name = "payer", desc = "Wallet paying for the transaction and new account, will be set as the creator of the constraint model")]
    #[account(2, name = "update_authority", desc = "Update authority of the constraint model")]
    #[account(3, name = "system_program", desc = "System program")]
    CreateEscrowConstraintModelAccount(CreateEscrowConstraintModelAccountArgs),

    /// Add a constraint to an escrow constraint model.
    #[account(0, writable, name = "escrow_constraint_model", desc = "Constraint model account")]
    #[account(1, writable, signer, name = "payer", desc = "Wallet paying for the transaction and new account, will be set as the creator of the constraint model")]
    #[account(2, signer, name = "update_authority", desc = "Update authority of the constraint model")]
    #[account(3, name = "system_program", desc = "System program")]
    AddConstraintToEscrowConstraintModel(AddConstraintToEscrowConstraintModelArgs),

    /// Creates a Trifle Account -- used to model token inventory in a Token Escrow account.
    #[account(0, writable, name = "escrow", desc = "Escrow account")]
    #[account(1, name = "metadata", desc = "Metadata account")]
    #[account(2, name = "mint", desc = "Mint account")]
    #[account(3, writable, name = "token_account", desc = "Token account (base token)")]
    #[account(4, name = "edition", desc = "Edition account")]
    #[account(5, writable, name = "trifle_account", desc = "Trifle account")]
    #[account(6, name = "trifle_authority", desc = "Trifle Authority - the account that can sign transactions for the trifle account")]
    #[account(7, name = "escrow_constraint_model", desc = "Escrow constraint model")]
    #[account(8, writable, signer, name = "payer", desc = "Wallet paying for the transaction")]
    #[account(9, name = "token_metadata_program", desc = "Token Metadata program")]
    #[account(10, name = "system_program", desc = "System program")]
    CreateTrifleAccount,

    /// Transfer tokens into the Trifle escrow account.
    #[account(0, writable, name="trifle_account", desc="The trifle account to use")]
    #[account(1, name="constraint_model", desc="The constraint model to check against")]
    #[account(2, name="escrow_account", desc="The escrow account attached to the NFT")]
    #[account(3, writable, signer, name="payer", desc="The payer for the transaction")]
    #[account(4, writable, signer, name="trifle_authority", desc="The authority of the trifle account")]
    #[account(5, name="attribute_mint", desc="The mint of the attribute")]
    #[account(6, writable, name="attribute_src_token_account", desc="The token account the attribute is being transferred from")]
    #[account(7, writable, name="attribute_dst_token_account", desc="The token account the attribute is being transferred to")]
    #[account(8, name="attribute_metadata", desc="The metadata of the attribute")]
    #[account(9, name="escrow_mint", desc="The mint the escrow is attached to")]
    #[account(10, name="escrow_token_account", desc="The token account holding the NFT the escrow is attached to")]
    #[account(11, name="system_program", desc="The system program")]
    #[account(12, name="spl_associated_token_account", desc="The associated token account program")]
    #[account(13, name="spl_token", desc="The spl token program")]
    #[account(14, name="rent", desc="The rent sysvar")]
    TransferIn(TransferInArgs),

    /// Transfer tokens out of the Trifle escrow account.
    #[account(0, writable, name="trifle_account", desc="The trifle account to use")]
    #[account(1, name="constraint_model", desc="The constraint model to check against")]
    #[account(2, name="escrow_account", desc="The escrow account attached to the NFT")]
    #[account(3, writable, signer, name="payer", desc="The payer for the transaction")]
    #[account(4, writable, signer, name="trifle_authority", desc="The authority of the trifle account")]
    #[account(5, name="attribute_mint", desc="The mint of the attribute")]
    #[account(6, writable, name="attribute_src_token_account", desc="The token account the attribute is being transferred from")]
    #[account(7, writable, name="attribute_dst_token_account", desc="The token account the attribute is being transferred to")]
    #[account(8, name="attribute_metadata", desc="The metadata of the attribute")]
    #[account(9, name="escrow_mint", desc="The mint the escrow is attached to")]
    #[account(10, name="escrow_token_account", desc="The token account holding the NFT the escrow is attached to")]
    #[account(11, name="system_program", desc="The system program")]
    #[account(12, name="spl_associated_token_account", desc="The associated token account program")]
    #[account(13, name="spl_token", desc="The spl token program")]
    #[account(14, name="rent", desc="The rent sysvar")]
    TransferOut(TransferOutArgs),
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
        data: TrifleInstruction::CreateEscrowConstraintModelAccount(
            CreateEscrowConstraintModelAccountArgs { name },
        )
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
    constraint_name: String,
    constraint: EscrowConstraint,
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
        data: TrifleInstruction::AddConstraintToEscrowConstraintModel(
            AddConstraintToEscrowConstraintModelArgs {
                constraint_name,
                constraint,
            },
        )
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
        AccountMeta::new_readonly(mpl_token_metadata::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: TrifleInstruction::CreateTrifleAccount.try_to_vec().unwrap(),
    }
}

pub fn transfer_in(
    program_id: Pubkey,
    trifle_account: Pubkey,
    constraint_model: Pubkey,
    escrow_account: Pubkey,
    payer: Pubkey,
    trifle_authority: Pubkey,
    attribute_mint: Pubkey,
    attribute_src_token_account: Pubkey,
    attribute_dst_token_account: Pubkey,
    attribute_metadata: Pubkey,
    escrow_mint: Pubkey,
    escrow_token_account: Pubkey,
    slot: String,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(trifle_account, false),
        AccountMeta::new_readonly(constraint_model, false),
        AccountMeta::new_readonly(escrow_account, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(trifle_authority, false),
        AccountMeta::new_readonly(attribute_mint, false),
        AccountMeta::new(attribute_src_token_account, false),
        AccountMeta::new(attribute_dst_token_account, false),
        AccountMeta::new_readonly(attribute_metadata, false),
        AccountMeta::new_readonly(escrow_mint, false),
        AccountMeta::new_readonly(escrow_token_account, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
    ];

    let data = TrifleInstruction::TransferIn(TransferInArgs { slot, amount })
        .try_to_vec()
        .unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

pub fn transfer_out(
    program_id: Pubkey,
    trifle_account: Pubkey,
    constraint_model: Pubkey,
    escrow_account: Pubkey,
    payer: Pubkey,
    trifle_authority: Pubkey,
    attribute_mint: Pubkey,
    attribute_src_token_account: Pubkey,
    attribute_dst_token_account: Pubkey,
    attribute_metadata: Pubkey,
    escrow_mint: Pubkey,
    escrow_token_account: Pubkey,
    slot: String,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(trifle_account, false),
        AccountMeta::new_readonly(constraint_model, false),
        AccountMeta::new_readonly(escrow_account, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(trifle_authority, false),
        AccountMeta::new_readonly(attribute_mint, false),
        AccountMeta::new(attribute_src_token_account, false),
        AccountMeta::new(attribute_dst_token_account, false),
        AccountMeta::new_readonly(attribute_metadata, false),
        AccountMeta::new_readonly(escrow_mint, false),
        AccountMeta::new_readonly(escrow_token_account, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(mpl_token_metadata::id(), false),
    ];

    let data = TrifleInstruction::TransferOut(TransferOutArgs { slot, amount })
        .try_to_vec()
        .unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}
