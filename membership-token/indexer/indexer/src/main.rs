pub mod workers;

use tokio::{runtime::Builder, signal};

#[tokio::main]
async fn main() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(100)
        .thread_name("indexer-main-pool")
        .enable_time()
        .build()
        .unwrap();

    let _ = runtime.spawn(async move { workers::dispatcher::run().await });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
