
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use warp::Filter;

use crate::cache::Cache;

use crate::database::OnlineStatus;
use crate::node_pool::NodePool;

use serde::Serialize;

#[derive(Serialize, Debug)]
struct NodeResponse {
    edkey: String,
    version: String,
    total_stored: i32,
    online: bool,
    connections_in: u32,
    store_requests: u32, // in the previous hour-long period
    retrieve_requests: u32, // in the previous hour-long period
}

struct Swarm {
    swarm_id: String, 
    nodes: Vec<NodeResponse>
}

#[derive(Serialize)]
struct StatsResponse {
    nodes: Vec<NodeResponse>
}

// TODO:
// 0. add onion requests (not provided by SS yet)
// 1. work out uptime of nodes
// 2. organise into swarms?

fn get_stats(cache: Arc<Mutex<Cache>>, pool: Arc<Mutex<NodePool>>) -> String {

    // Just clone the whole thing so we can releas the lock asap
    // I'm pretty sure that I don't need a double lock here...
    let all_nodes = pool.lock().unwrap().all_nodes.lock().unwrap().clone();

    let cache = cache.lock().unwrap();

    let mut swarms : HashMap<String, Vec<NodeResponse>> = HashMap::new();

    let res : Vec<_> = all_nodes.iter().map(|(edkey, info)| {

        let mut version = "?".to_owned();
        let mut online = false;
        let mut total_stored: i32 = -1;
        let mut connections_in: u32 = 0;

        let mut store_requests = 0;
        let mut retrieve_requests = 0;

        if let Some(v) = cache.status_map.get(edkey) {
            online = if v.status == OnlineStatus::ONLINE { true } else { false };
        }

        if let Some(v) = cache.stats_map.get(edkey) {
            version = v.version.clone();
            total_stored = v.total_stored as i32;
            connections_in = v.connections_in;
            store_requests = v.previous_period_store_requests;
            retrieve_requests = v.previous_period_retrieve_requests;
        }

        let nodes = swarms.entry(info.swarm_id.to_string()).or_insert(vec![]);

        let node_entry = NodeResponse {
            edkey: edkey.clone(),
            version,
            total_stored,
            online,
            connections_in,
            store_requests,
            retrieve_requests,
        };

        nodes.push(node_entry);

    }).collect();

    // Organise by swarm id

    return serde_json::to_string(&swarms).expect("Could not construct json");
}

pub async fn serve(cache: Arc<Mutex<Cache>>, pool: Arc<Mutex<NodePool>>) {

    let hello = warp::path!("get_status").map(move || get_stats(cache.clone(), pool.clone()));

    warp::serve(hello).run(([0, 0, 0, 0], 3030)).await;

}



