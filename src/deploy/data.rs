use std::sync::{atomic::AtomicBool, Arc};

pub struct DeployArgs {
    pub config: String,
    pub cache: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub handler: Arc<AtomicBool>,
}
