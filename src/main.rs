mod database;
mod lokid_api;
mod sn_api;
mod node_pool;
mod node_poller;
mod cache;
mod httpserver;

use std::sync::{Arc, Mutex};

use lokid_api::Network;
use node_poller::NodePoller;
use node_pool::NodePool;

use cache::Cache;

#[macro_use]
extern crate lazy_static;


/*
TO SHOW TO USERS:

base:
- last status: online/offline
- average uptime (before it is disconnected)
- total inactive time (in the last 24 hours)

maybe:
- messages stored
- onion requests processed


----- TABLES -----

Online status change table:
node | time | status

- status is (gone online|gone offline) are recorded whenever the status changes

*/

#[tokio::main]
async fn main() {
    use clap::{App, Arg};

    let matches = App::new("online-swarm-monitor")
        .version("0.1")
        .arg(Arg::with_name("net").long("net").takes_value(true))
        .get_matches();

    let network: &Network = match matches.value_of("net") {
        Some(n) => {
            if n == "testnet" {
                &lokid_api::TESTNET
            } else if n == "mainnet" {
                &lokid_api::MAINNET
            } else {
                &lokid_api::MAINNET
            }
        }
        None => &lokid_api::MAINNET,
    };

    dbg!(&network);

    let mut db_cache = Cache::new();

    db_cache.load();

    let db_cache = Arc::new(Mutex::new(db_cache));

    let mut pool = NodePool::new(&network);

    pool.connect().await;

    let pool = Arc::new(Mutex::new(pool));

    let poller = NodePoller::new(db_cache.clone(), pool.clone());

    poller.start().await;

    httpserver::serve(db_cache, pool).await;
}
