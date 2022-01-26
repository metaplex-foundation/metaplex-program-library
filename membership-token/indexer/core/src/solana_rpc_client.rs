use std::str::FromStr;

use solana_client::{
    client_error::ClientError,
    rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransaction, UiTransactionEncoding};

pub const TRANSACTIONS_BATCH_LEN: usize = 100;

pub struct SolanaRpcClientConfig {
    pub url: String,
    pub program_address: Pubkey,
}
pub struct SolanaRpcClient {
    rpc_client: RpcClient,
    program_address: Pubkey,
}

impl SolanaRpcClient {
    pub fn new_with_config(config: SolanaRpcClientConfig) -> Self {
        SolanaRpcClient {
            rpc_client: RpcClient::new(config.url),
            program_address: config.program_address,
        }
    }

    pub fn load_signatures_batch(
        &self,
        before: Option<Signature>,
        until: Option<Signature>,
    ) -> Vec<RpcConfirmedTransactionStatusWithSignature> {
        let config = GetConfirmedSignaturesForAddress2Config {
            before,
            until,
            limit: Some(TRANSACTIONS_BATCH_LEN),
            commitment: Some(CommitmentConfig::finalized()),
        };

        self.rpc_client
            .get_signatures_for_address_with_config(&self.program_address, config)
            .unwrap()
    }

    pub fn load_trqansaction_info(
        &self,
        signature: &str,
    ) -> Result<EncodedConfirmedTransaction, ClientError> {
        let signature = Signature::from_str(signature).unwrap();

        self.rpc_client
            .get_transaction(&signature, UiTransactionEncoding::Json)
    }
}
