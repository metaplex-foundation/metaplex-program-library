use mpl_token_auth_rules::payload::Payload;
use rooster::{
    instruction::{
        delegate as rooster_delegate, delegate_transfer, init, withdraw, DelegateTransferArgs,
        WithdrawArgs,
    },
    pda::find_rooster_pda,
    AuthorizationData,
};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use spl_associated_token_account::get_associated_token_address;

use super::*;

pub struct RoosterManager {
    pda: Pubkey,
    authority: Keypair,
    bump: u8,
}

impl RoosterManager {
    pub async fn init(
        context: &mut ProgramTestContext,
        authority: Keypair,
    ) -> Result<RoosterManager, BanksClientError> {
        let (pda, bump) = find_rooster_pda(&authority.pubkey());

        let ix = init(authority.pubkey(), pda);

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&authority.pubkey()),
            &[&authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        Ok(RoosterManager {
            pda,
            authority,
            bump,
        })
    }

    pub async fn withdraw(
        &self,
        context: &mut ProgramTestContext,
        authority: &Keypair,
        destination_owner: Pubkey,
        mint: Pubkey,
        metadata: Pubkey,
        edition: Pubkey,
        rule_set: Pubkey,
        payload: Payload,
    ) -> Result<(), BanksClientError> {
        let args = WithdrawArgs {
            auth_data: AuthorizationData::new(payload),
        };

        let token = get_associated_token_address(&self.pda(), &mint);
        let destination = get_associated_token_address(&destination_owner, &mint);

        let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(800_000);

        let ix = withdraw(
            authority.pubkey(),
            self.pda,
            token,
            destination_owner,
            destination,
            mint,
            metadata,
            edition,
            rule_set,
            args,
        );

        let tx = Transaction::new_signed_with_payer(
            &[compute_ix, ix],
            Some(&authority.pubkey()),
            &[authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn delegate(
        &self,
        context: &mut ProgramTestContext,
        delegate: &Keypair,
        mint: Pubkey,
        metadata: Pubkey,
        edition: Pubkey,
        authorization_rules: Option<Pubkey>,
        args: rooster::instruction::DelegateArgs,
    ) -> Result<(), BanksClientError> {
        let token = get_associated_token_address(&self.pda(), &mint);

        let ix = rooster_delegate(
            delegate.pubkey(),
            self.pda,
            token,
            mint,
            metadata,
            edition,
            authorization_rules,
            args,
        );

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&delegate.pubkey()),
            &[delegate],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn delegate_transfer(
        self,
        context: &mut ProgramTestContext,
        authority: &Keypair,
        source_owner: Pubkey,
        destination_owner: Pubkey,
        mint: Pubkey,
        rule_set: Pubkey,
        payload: Payload,
    ) -> Result<(), BanksClientError> {
        let source_token = get_associated_token_address(&source_owner, &mint);
        let destination_token = get_associated_token_address(&destination_owner, &mint);

        let compute_ix = ComputeBudgetInstruction::set_compute_unit_limit(800_000);

        let args = DelegateTransferArgs {
            amount: 1,
            auth_data: AuthorizationData::new(payload),
        };

        let ix = delegate_transfer(
            authority.pubkey(),
            self.pda,
            source_owner,
            source_token,
            destination_owner,
            destination_token,
            mint,
            rule_set,
            args,
        );

        let tx = Transaction::new_signed_with_payer(
            &[compute_ix, ix],
            Some(&authority.pubkey()),
            &[authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub fn authority(&self) -> Pubkey {
        self.authority.pubkey()
    }

    pub fn pda(&self) -> Pubkey {
        self.pda
    }

    pub fn bump(&self) -> u8 {
        self.bump
    }
}
