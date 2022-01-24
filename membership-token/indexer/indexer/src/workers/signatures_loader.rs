use indexer_core::{
    db::Db,
    solana_rpc_client::{self, SolanaRpcClient},
};

use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time::{sleep, Duration},
};

#[derive(Debug, Clone, Copy)]
pub struct ConnectionConfig {
    pub url: &'static str,
}

#[derive(Copy, Clone, Debug)]
pub struct SignaturesForAddressConfig {
    _before: Option<Signature>,
    _until: Option<Signature>,
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Start { config: ConnectionConfig },
    Stop,
    Load { config: SignaturesForAddressConfig },
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Started,
    Stopped,
    AlreadyStarted,
    AlreadyStopped,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignaturesLoaderState {
    NotStarted,
    Started,
    Stopped,
}

struct SignaturesLoaderRegistry {
    state: SignaturesLoaderState,
    rpc_client: Option<solana_rpc_client::SolanaRpcClient>,
    db: Option<Db>,
}

pub async fn run(id: u8, tx: Sender<Message>, mut rx: Receiver<Command>) {
    println!("SignaturesLoader{}::run()", id);

    let mut registry = SignaturesLoaderRegistry {
        state: SignaturesLoaderState::NotStarted,
        rpc_client: None,
        db: None,
    };

    let mut newest_transaction: Option<Signature> = None;
    let mut before: Option<Signature> = None;
    let until: Option<Signature> = None;

    loop {
        if let Ok(command) = rx.try_recv() {
            process_command(command, &mut registry, &tx).await;
        }

        // Skip all following instructions and do nothing if this actor was not started
        if SignaturesLoaderState::Started != registry.state {
            continue;
        }

        // ToDo: add error processing
        let signatures = registry
            .rpc_client
            .as_ref()
            .unwrap()
            .load_signatures_batch(before, until);

        if newest_transaction.is_none() {
            newest_transaction =
                Some(Signature::from_str(&signatures.get(0).unwrap().signature).unwrap());
        }

        before = Some(Signature::from_str(&signatures.iter().last().unwrap().signature).unwrap());

        if registry.db.is_some() {
            registry
                .db
                .as_ref()
                .unwrap()
                .store_signatures_in_queue(signatures)
                .unwrap();
        }

        sleep(Duration::from_millis(500)).await;
    }
}

async fn process_command(
    command: Command,
    registry: &mut SignaturesLoaderRegistry,
    tx: &Sender<Message>,
) {
    match command {
        Command::Start { config } => {
            start(config.url.to_string(), registry, tx).await;
        }
        Command::Stop => {}
        Command::Load { .. } => {}
    }
}

async fn start(url: String, registry: &mut SignaturesLoaderRegistry, tx: &Sender<Message>) {
    if SignaturesLoaderState::Started == registry.state {
        tx.send(Message::AlreadyStarted).unwrap();
    } else {
        let solana_rpc_client_config = solana_rpc_client::SolanaRpcClientConfig {
            url,
            program_address: Pubkey::from_str("p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98")
                .unwrap(),
        };
        registry.rpc_client = Some(SolanaRpcClient::new_with_config(solana_rpc_client_config));
        registry.state = SignaturesLoaderState::Started;
        registry.db = Some(Db::default());
        tx.send(Message::Started).unwrap();
    }
}
