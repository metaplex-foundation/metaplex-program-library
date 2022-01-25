use indexer_core::{
    db::Db,
    solana_rpc_client::{self, SolanaRpcClient},
};

use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use tokio::{
    fs,
    fs::File,
    io::{self, AsyncWriteExt},
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

#[derive(Serialize, Deserialize, Debug)]
struct SavedState {
    newest_transaction: Option<Signature>,
    before: Option<Signature>,
    until: Option<Signature>,
}

pub async fn run(
    id: u8,
    mut stop_rx: Receiver<u8>,
    _stop_fb_tx: mpsc::Sender<()>,
    tx: Sender<Message>,
    mut rx: Receiver<Command>,
) {
    println!("SignaturesLoader{}::run()", id);

    let mut registry = SignaturesLoaderRegistry {
        state: SignaturesLoaderState::NotStarted,
        rpc_client: None,
        db: None,
    };

    let mut saved_state = load_state().await;

    loop {
        if let Ok(command) = rx.try_recv() {
            process_command(command, &mut registry, &tx).await;
        }

        if stop_rx.try_recv().is_ok() {
            break;
        }

        sleep(Duration::from_millis(100)).await;

        // Skip all following instructions and do nothing if this actor was not started
        if SignaturesLoaderState::Started != registry.state {
            continue;
        }

        // ToDo: add error processing
        let signatures = registry
            .rpc_client
            .as_ref()
            .unwrap()
            .load_signatures_batch(saved_state.before, saved_state.until);

        if saved_state.newest_transaction.is_none() {
            saved_state.newest_transaction =
                Some(Signature::from_str(&signatures.get(0).unwrap().signature).unwrap());
        }

        saved_state.before =
            Some(Signature::from_str(&signatures.iter().last().unwrap().signature).unwrap());

        if registry.db.is_some() {
            registry
                .db
                .as_ref()
                .unwrap()
                .store_signatures_in_queue(signatures)
                .unwrap();
        }
    }

    save_state(&saved_state).await.unwrap();

    println!("SignaturesLoader{}::stop()", id);
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

async fn load_state() -> SavedState {
    match fs::read_to_string("./stored_state.dat").await {
        Ok(stored_state) => serde_json::from_str(&stored_state).unwrap(),
        _ => SavedState {
            newest_transaction: None,
            before: None,
            until: None,
        },
    }
}

async fn save_state(state: &SavedState) -> io::Result<()> {
    let mut stored_state = File::create("./stored_state.dat").await?;
    stored_state
        .write(serde_json::to_string(state).unwrap().as_bytes())
        .await?;
    Ok(())
}
