use std::env;

use alloy_primitives::Address;
use alloy_provider::ProviderBuilder;
use alloy_signer_local::PrivateKeySigner;
use alloy_transport_http::reqwest::Url;
use clap::Parser;
use op_alloy_network::EthereumWallet;

use fault_proof::{
    contract::DisputeGameFactory, proposer::OPSuccinctProposer, utils::setup_logging,
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = ".env.proposer")]
    env_file: String,
}

#[tokio::main]
async fn main() {
    setup_logging();

    let args = Args::parse();
    dotenv::from_filename(args.env_file).ok();

    let wallet = EthereumWallet::from(
        env::var("PRIVATE_KEY")
            .expect("PRIVATE_KEY must be set")
            .parse::<PrivateKeySigner>()
            .unwrap(),
    );

    let l1_provider_with_wallet = ProviderBuilder::new()
        .wallet(wallet.clone())
        .on_http(env::var("L1_RPC").unwrap().parse::<Url>().unwrap());

    let factory = DisputeGameFactory::new(
        env::var("FACTORY_ADDRESS")
            .expect("FACTORY_ADDRESS must be set")
            .parse::<Address>()
            .unwrap(),
        l1_provider_with_wallet.clone(),
    );

    // Use PROVER_ADDRESS from env if available, otherwise use wallet's default signer address from the private key.
    let prover_address = env::var("PROVER_ADDRESS")
        .ok()
        .and_then(|addr| addr.parse::<Address>().ok())
        .unwrap_or_else(|| wallet.default_signer().address());

    let proposer = OPSuccinctProposer::new(prover_address, l1_provider_with_wallet, factory)
        .await
        .unwrap();
    proposer.run().await.expect("Runs in an infinite loop");
}
