use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer};
use solana_program::system_instruction;
use solana_program_test::*;
use solana_sdk::transaction::Transaction;

pub async fn airdrop(context: &mut ProgramTestContext, receiver: &Pubkey, amount: u64) {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}
