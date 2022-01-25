use parking_lot::Mutex;
use std::{str::FromStr, sync::Arc};

use indexer_core::{solana_rpc_client, Db, SolanaRpcClient};
use solana_sdk::pubkey::Pubkey;
use tokio::{
    sync::{
        broadcast::{Receiver, Sender},
        mpsc,
    },
    time::{sleep, Duration},
};

#[derive(Debug, Clone, Copy)]
pub struct ConnectionConfig {
    pub url: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Start {
        channel_id: u8,
        config: ConnectionConfig,
    },
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransactionsLoaderState {
    NotStarted,
    Started,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Started,
    Stopped,
    AlreadyStarted,
    AlreadyStopped,
}

struct TransactionsLoaderRegistry {
    channel_id: u8,
    state: TransactionsLoaderState,
    rpc_client: Option<solana_rpc_client::SolanaRpcClient>,
    db: Option<Db>,
}

pub async fn run(
    channel_id: u8,
    mut stop_rx: Receiver<u8>,
    _stop_fb_tx: mpsc::Sender<()>,
    tx: Sender<Message>,
    mut rx: Receiver<Command>,
    guarded_db: Arc<Mutex<Db>>,
) {
    println!("TransactionsLoader{}::run()", channel_id);

    let mut registry = TransactionsLoaderRegistry {
        channel_id,
        state: TransactionsLoaderState::NotStarted,
        rpc_client: None,
        db: None,
    };

    loop {
        if let Ok(command) = rx.try_recv() {
            process_command(command, &mut registry, &tx).await;
        }

        if stop_rx.try_recv().is_ok() {
            break;
        }

        sleep(Duration::from_millis(100)).await;

        // Skip all following instructions and do nothing if this actor was not started
        if TransactionsLoaderState::Started != registry.state {
            continue;
        }

        let signature: Option<String>;

        {
            let db = guarded_db.lock();

            signature = match db.get_signature_from_queue() {
                Ok(result) => {
                    db.delete_signature_from_queue(result.0);
                    result.1
                }
                _ => None,
            };
        }

        if signature.is_some() {
            let signature = signature.unwrap();
            let transaction_info = registry
                .rpc_client
                .as_ref()
                .unwrap()
                .load_trqansaction_info(&signature);

            if let Ok(encoded_transaction) = transaction_info {
                if registry.db.is_some() {
                    registry
                        .db
                        .as_ref()
                        .unwrap()
                        .store_transaction(&signature, encoded_transaction)
                        .unwrap();
                }
                println!("{} -- {}", channel_id, signature);
            }
        }
    }

    println!("TransactionsLoader{}::stop()", channel_id);
}

async fn process_command(
    command: Command,
    registry: &mut TransactionsLoaderRegistry,
    tx: &Sender<Message>,
) {
    match command {
        Command::Start { channel_id, config } => {
            if registry.channel_id == channel_id {
                start(config.url.to_string(), registry, tx).await;
            }
        }
        Command::Stop => {}
    }
}

async fn start(url: String, registry: &mut TransactionsLoaderRegistry, tx: &Sender<Message>) {
    if TransactionsLoaderState::Started == registry.state {
        tx.send(Message::AlreadyStarted).unwrap();
    } else {
        let solana_rpc_client_config = solana_rpc_client::SolanaRpcClientConfig {
            url,
            program_address: Pubkey::from_str("p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98")
                .unwrap(),
        };
        registry.rpc_client = Some(SolanaRpcClient::new_with_config(solana_rpc_client_config));
        registry.state = TransactionsLoaderState::Started;
        registry.db = Some(Db::default());
        tx.send(Message::Started).unwrap();
    }
}
