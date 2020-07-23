
mod node_pool;
mod lokid_api;
mod sn_api;

use lokid_api::Network;

const FOUNDATION_TESTNET_SEED: &'static str = "http://public.loki.foundation:38157/json_rpc";
const FOUNDATION_MAINNET_SEED: &'static str = "http://public.loki.foundation:22023/json_rpc";

#[tokio::main]
async fn main() {

    use clap::{App, Arg};

    let matches = App::new("online-swarm-monitor")
        .version("0.1")
        .arg(Arg::with_name("net").takes_value(true)).get_matches();

    let testnet = Network {
        seed_url: FOUNDATION_TESTNET_SEED,
        is_testnet: true,
    };

    let mainnet = Network {
        seed_url: FOUNDATION_MAINNET_SEED,
        is_testnet: false,
    };

    let network = match matches.value_of("net") {
        Some(n) => {
            if n == "net=testnet" {
                testnet
            } else if n == "net=mainnet" {
                mainnet
            } else {
                mainnet
            }
        }
        None => mainnet,
    };

    let node_pool = lokid_api::get_n_service_nodes(2000, &network).await;

    println!("Node pool size: {}", node_pool.len());

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("building certificate");

    let stats = sn_api::get_stats(&client, &node_pool[0]).await;

    println!("Stats: {:?}", stats);

}
