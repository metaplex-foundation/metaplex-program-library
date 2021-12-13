use rand::prelude::*;
use randomness_oracle_program::{id, instruction, state::RandomnessOracle};
use solana_program_test::*;
use solana_sdk::{
    program_pack::Pack, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction, transport,
};

use super::get_account;

pub struct TestRandomnessOracle {
    pub keypair: Keypair,
    pub rand: ThreadRng,
}

impl TestRandomnessOracle {
    pub fn new() -> Self {
        let rng = rand::thread_rng();
        TestRandomnessOracle {
            keypair: Keypair::new(),
            rand: rng,
        }
    }

    pub async fn init(&mut self, context: &mut ProgramTestContext) -> transport::Result<()> {
        let rent = context.banks_client.get_rent().await.unwrap();
        println!("{}", self.keypair.pubkey());
        let tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &self.keypair.pubkey(),
                    rent.minimum_balance(RandomnessOracle::LEN),
                    RandomnessOracle::LEN as u64,
                    &id(),
                ),
                instruction::init_randomness_oracle(
                    &id(),
                    &self.keypair.pubkey(),
                    &context.payer.pubkey(),
                ),
            ],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.keypair],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn update(&mut self, context: &mut ProgramTestContext) -> transport::Result<()> {
        let value: [u8; 32] = self.rand.gen();
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_randomness_oracle(
                &id(),
                &self.keypair.pubkey(),
                &context.payer.pubkey(),
                value,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn get_data(&mut self, context: &mut ProgramTestContext) -> RandomnessOracle {
        let account = get_account(context, &self.keypair.pubkey()).await;
        RandomnessOracle::unpack_unchecked(&account.data).unwrap()
    }
}
