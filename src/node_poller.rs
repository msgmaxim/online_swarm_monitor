use crate::lokid_api::{ServiceNodeRecord};
use crate::node_pool::NodePool;

use std::sync::{Arc, Mutex};
use crate::cache::Cache;

use crate::sn_api;

use tokio::task;

use crate::database::OnlineStatus;

pub struct NodePoller {
    cache: Arc<Mutex<Cache>>,
    pool: Arc<Mutex<NodePool>>
}

async fn get_and_process_stat(client: &reqwest::Client, sn: ServiceNodeRecord, cache: Arc<Mutex<Cache>>) {

    let stats = sn_api::get_stats(&client, &sn).await;

    let mut cache = cache.lock().unwrap();

    match stats {

        Ok(_stats) => {
            cache.update_node_stats(&sn.pubkey_ed25519, &_stats);
            cache.update_status(&sn.pubkey_ed25519, OnlineStatus::ONLINE);
        },
        Err(err) => {
            println!("Node failed: {}", err);
            cache.update_status(&sn.pubkey_ed25519, OnlineStatus::OFFLINE);
        }
    }

}

lazy_static! {

    static ref CLIENT : reqwest::Client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("building certificate");

}

async fn polling_thread(cache: Arc<Mutex<Cache>>, pool : Arc<Mutex<NodePool>>) {

    loop {

        let to_poll = {
            let mut pool = pool.lock().unwrap();
            let to_poll = pool.get_next_nodes(10);
            to_poll
        };

        for node in to_poll {
            task::spawn(get_and_process_stat(&CLIENT, node.clone(), cache.clone()));
        }

        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
    }
}

impl NodePoller {
    pub fn new(cache: Arc<Mutex<Cache>>, pool : Arc<Mutex<NodePool>>) -> NodePoller {

        NodePoller { cache, pool }
    }

    pub async fn start(&self) {

        task::spawn(polling_thread(self.cache.clone(), self.pool.clone()));

    }
}
