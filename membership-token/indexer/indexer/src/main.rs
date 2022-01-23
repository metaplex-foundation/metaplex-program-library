pub mod workers;

use tokio::{runtime::Builder, signal};

#[tokio::main]
async fn main() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("indexer-main-worker")
        .enable_time()
        .build()
        .unwrap();

    let _ = runtime.spawn(workers::dispatcher::run());

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
