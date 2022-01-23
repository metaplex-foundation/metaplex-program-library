use super::{
    signatures_loader,
    signatures_loader::{Command, ConnectionConfig},
};

use tokio::sync::mpsc;

pub async fn run() {
    println!("Dispatcher::run()");

    // The channel for sending messages from main to signatures_loader
    let (dispatcher_sgnloader_tx, dispatcher_sgnloader_rx) = mpsc::channel::<Command>(8);

    // The channel for sending messages from signatures_loader to main
    let (sgnloader_dispatcher_tx, mut sgnloader_dispatcher_rx) =
        mpsc::channel::<signatures_loader::Message>(8);

    tokio::spawn(super::signatures_loader::run(
        sgnloader_dispatcher_tx,
        dispatcher_sgnloader_rx,
    ));

    let config = ConnectionConfig {
        url: "https://api.mainnet-beta.solana.com".to_string(),
    };
    let cmd = Command::Start { config };

    dispatcher_sgnloader_tx.send(cmd).await.unwrap();

    loop {
        if let Some(_message) = sgnloader_dispatcher_rx.recv().await {}
    }
}
