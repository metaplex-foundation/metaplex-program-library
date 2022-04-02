pub struct DeployArgs {
    pub assets_dir: String,
    pub config: String,
    pub cache: String,
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
}
