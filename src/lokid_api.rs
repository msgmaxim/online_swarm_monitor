
use std::fmt::{self};

use serde_json::{json, Value};
use serde::Deserialize;

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

pub struct Network {
    pub seed_url: &'static str,
    pub is_testnet: bool,
}

pub async fn get_n_service_nodes(limit: u32, network: &Network) -> Vec<ServiceNodeRecord> {
    let client = reqwest::Client::new();

    let params = json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": "get_n_service_nodes",
        "params": {
            "limit": &limit,
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
        .await
        .expect("Failed to send get_n_service_nodes");

    let res_text = res.text().await.expect("obtaining text from response");

    let v: Value = serde_json::from_str(&res_text).expect("parsing json");

    let array = &v["result"]["service_node_states"];

    serde_json::from_value(array.clone()).expect("from json to value")
}