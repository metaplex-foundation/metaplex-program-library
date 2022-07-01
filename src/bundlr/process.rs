use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer};
use bundlr_sdk::deep_hash::{deep_hash, DeepHashChunk};
use console::style;
use data_encoding::BASE64URL;

use crate::{
    candy_machine::CANDY_MACHINE_ID, cli::BundlrAction, common::*, config::*,
    upload::methods::BundlrMethod, utils::*,
};

// The minimum amount required for withdraw.
const LIMIT: u64 = 5000;

pub struct BundlrArgs {
    pub keypair: Option<String>,
    pub rpc_url: Option<String>,
    pub action: BundlrAction,
}

pub async fn process_bundlr(args: BundlrArgs) -> Result<()> {
    let sugar_config = sugar_setup(args.keypair, args.rpc_url)?;
    let client = setup_client(&sugar_config)?;

    // retrieving balance

    println!(
        "{} {}Retrieving balance",
        style(if let BundlrAction::Withdraw = args.action {
            "[1/2]"
        } else {
            "[1/1]"
        })
        .bold()
        .dim(),
        COMPUTER_EMOJI
    );

    let pb = spinner_with_style();
    pb.set_message("Connecting...");

    let program = client.program(CANDY_MACHINE_ID);
    let solana_cluster: Cluster = get_cluster(program.rpc())?;

    let http_client = reqwest::Client::new();
    let keypair = sugar_config.keypair;
    let address = keypair.pubkey().to_string();
    let bundlr_node = match solana_cluster {
        Cluster::Devnet => BUNDLR_DEVNET,
        Cluster::Mainnet => BUNDLR_MAINNET,
    };

    let balance = BundlrMethod::get_bundlr_balance(&http_client, &address, bundlr_node).await?;

    pb.finish_and_clear();

    println!("\nFunding address:");
    println!("  -> pubkey: {}", address);
    println!(
        "  -> lamports: {} (◎ {})",
        balance,
        balance as f64 / LAMPORTS_PER_SOL as f64
    );

    // withdrawing funds

    if let BundlrAction::Withdraw = args.action {
        println!(
            "\n{} {}Withdrawing funds",
            style("[2/2]").bold().dim(),
            WITHDRAW_EMOJI
        );

        if balance == 0 {
            println!("\nNo funds to withdraw.");
        } else if (balance - LIMIT) > 0 {
            let pb = spinner_with_style();
            pb.set_message("Connecting...");

            let balance = balance - LIMIT;

            // nonce
            let url = format!("{bundlr_node}/account/withdrawals/solana/?address={address}");
            let nonce = if let Some(value) = http_client
                .get(&url)
                .send()
                .await?
                .json::<Value>()
                .await?
                .as_u64()
            {
                value
            } else {
                pb.finish_and_clear();
                return Err(anyhow!("Failed to retrieve nonce for withdraw"));
            };

            // sign the message

            let message = deep_hash(DeepHashChunk::Chunks(vec![
                DeepHashChunk::Chunk("solana".to_string().as_bytes().to_vec().into()),
                DeepHashChunk::Chunk(balance.to_string().as_bytes().to_vec().into()),
                DeepHashChunk::Chunk(nonce.to_string().as_bytes().to_vec().into()),
            ]))
            .await?;
            let signature = keypair.sign_message(&message);

            let mut data = HashMap::new();
            data.insert("publicKey", BASE64URL.encode(&keypair.pubkey().to_bytes()));
            data.insert("currency", "solana".to_string());
            data.insert("amount", balance.to_string());
            data.insert("nonce", nonce.to_string());
            data.insert("signature", BASE64URL.encode(signature.as_ref()));
            data.insert("sigType", "2".to_string());

            let url = format!("{bundlr_node}/account/withdraw");
            let response = http_client.post(&url).json(&data).send().await?;

            pb.finish_and_clear();

            if response.status() == 200 {
                println!("\nWithdraw completed.");
            } else {
                println!("\n{}", style("Withdraw failed.").red().bold());
                let error = response.text().await?;
                return Err(anyhow!("Failed to complete withdraw ({})", error));
            }
        } else {
            println!(
                "\n{}",
                style("Insufficient balance for withdraw:").red().bold()
            );
            println!(
                "  -> required balance > {} (◎ {})",
                LIMIT,
                LIMIT as f64 / LAMPORTS_PER_SOL as f64
            );
        }
    }

    Ok(())
}
