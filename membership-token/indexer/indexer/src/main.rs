pub mod workers;

use tokio::{
    signal,
    sync::{broadcast, mpsc},
};

#[tokio::main]
async fn main() {
    let (stop_tx, _stop_rx) = broadcast::channel::<u8>(32);
    let (stop_fb_tx, mut stop_fb_rx) = mpsc::channel::<()>(1);

    let rx = stop_tx.subscribe();
    let stp_fb_tx = stop_fb_tx.clone();
    let _ = tokio::spawn(async move { workers::dispatcher::run(rx, stp_fb_tx).await });

    drop(stop_fb_tx);

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    stop_tx.send(0).unwrap();
    let _ = stop_fb_rx.recv().await;
}
