use std::fmt::{self};

use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Deserialize, Debug, Clone)]
pub struct ServiceNodeRecord {
    pub public_ip: String,
    pub storage_port: u16,
    pub storage_lmq_port: u16,
    pub pubkey_x25519: String,
    pub pubkey_ed25519: String,
    pub swarm_id: u64,
}

impl fmt::Display for ServiceNodeRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // port is most useful when testing locally, might change this for mainnet/testnet
        write!(f, "{}:{}", self.public_ip, self.storage_port)
    }
}

#[derive(Clone, Debug)]
pub struct Network {
    pub seed_url: &'static str,
    pub is_testnet: bool,
}

const FOUNDATION_TESTNET_SEED: &'static str = "http://public.loki.foundation:38157/json_rpc";
const FOUNDATION_MAINNET_SEED: &'static str = "http://public.loki.foundation:22023/json_rpc";

pub static TESTNET: Network = Network {
    seed_url: FOUNDATION_TESTNET_SEED,
    is_testnet: true,
};

pub static MAINNET: Network = Network {
    seed_url: FOUNDATION_MAINNET_SEED,
    is_testnet: false,
};

pub async fn get_all_service_nodes(network: &Network) -> Result<Vec<ServiceNodeRecord>, String> {
    let client = reqwest::Client::new();

    let params = json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": "get_n_service_nodes",
        "params": {
            "fields": {
                "public_ip": true,
                "storage_port": true,
                "storage_lmq_port": true,
                "pubkey_x25519": true,
                "pubkey_ed25519": true,
                "swarm_id": true,
            },
            "active_only": false,
        },
    });

    let res = client
        .post(network.seed_url)
        .json(&params)
        .send()
        .await.map_err(|err| format!("{}", err))?;

    let res_text = res.text().await.expect("obtaining text from response");

    let v: Value = serde_json::from_str(&res_text).expect("parsing json");

    let array = &v["result"]["service_node_states"];

    let res : Vec<ServiceNodeRecord> = serde_json::from_value(array.clone()).map_err(|error| {
        format!("Could not parse json: {}", error)
    })?;

    Ok(res)
}
