use std::sync::{Arc, Mutex};

use super::{signatures_loader, transactions_loader};

use indexer_core::Db;
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    time::{sleep, Duration},
};

struct Connection<C, M> {
    _tx: Sender<C>,
    rx: Receiver<M>,
}

pub async fn run() {
    println!("Dispatcher::run()");

    let mut dispatcher_sgnloader_connection = setup_and_start_signatures_loader().await;
    let mut dispatcher_trnsloaders_connection = setup_and_start_transactions_loaders().await;

    loop {
        if let Ok(_message) = dispatcher_sgnloader_connection.rx.try_recv() {}
        if let Ok(_message) = dispatcher_trnsloaders_connection.rx.try_recv() {}
        sleep(Duration::from_millis(500)).await;
    }
}

async fn setup_and_start_signatures_loader(
) -> Connection<signatures_loader::Command, signatures_loader::Message> {
    // The channel for sending messages from main to signatures_loader
    let (dispatcher_sgnloader_tx, dispatcher_sgnloader_rx) =
        broadcast::channel::<signatures_loader::Command>(32);

    // The channel for sending messages from signatures_loader to main
    let (sgnloader_dispatcher_tx, sgnloader_dispatcher_rx) =
        broadcast::channel::<signatures_loader::Message>(32);

    tokio::spawn(async move {
        super::signatures_loader::run(1, sgnloader_dispatcher_tx, dispatcher_sgnloader_rx).await
    });

    let config = signatures_loader::ConnectionConfig {
        url: "https://api.mainnet-beta.solana.com",
    };
    let cmd = signatures_loader::Command::Start { config };

    dispatcher_sgnloader_tx.send(cmd).unwrap();

    Connection {
        _tx: dispatcher_sgnloader_tx,
        rx: sgnloader_dispatcher_rx,
    }
}

async fn setup_and_start_transactions_loaders(
) -> Connection<transactions_loader::Command, transactions_loader::Message> {
    // The channel for sending messages from main to signatures_loader
    let (dispatcher_trnsloader_tx, _dispatcher_trnsloader_rx) =
        broadcast::channel::<transactions_loader::Command>(32);

    // The channel for sending messages from signatures_loader to main
    let (trnsloader_dispatcher_tx, trnsloader_dispatcher_rx) =
        broadcast::channel::<transactions_loader::Message>(32);

    let db = Db::default();
    let db_mutex = Arc::new(Mutex::new(db));

    for channel_id in 1..3 {
        let tx = trnsloader_dispatcher_tx.clone();
        let rx = dispatcher_trnsloader_tx.subscribe();
        let guarded_db = db_mutex.clone();
        tokio::spawn(async move {
            super::transactions_loader::run(channel_id, tx, rx, guarded_db).await
        });

        let config = transactions_loader::ConnectionConfig {
            url: "https://api.mainnet-beta.solana.com",
        };

        let cmd = transactions_loader::Command::Start { channel_id, config };

        dispatcher_trnsloader_tx.send(cmd).unwrap();
    }

    Connection {
        _tx: dispatcher_trnsloader_tx,
        rx: trnsloader_dispatcher_rx,
    }
}
