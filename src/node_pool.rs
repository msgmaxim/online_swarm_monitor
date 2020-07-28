use crate::lokid_api::{self, Network, ServiceNodeRecord};

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use std::sync::{Arc, Mutex};

use rand::seq::SliceRandom;
use rand::thread_rng;

pub type EdKey = String;

type SafeMap<K, V> = Arc<Mutex<BTreeMap<K, V>>>;
type SafeNodeMap = SafeMap<EdKey, ServiceNodeRecord>;

pub struct NodePool {
    network: Network,

    // All nodes known on the network
    pub all_nodes: SafeMap<EdKey, ServiceNodeRecord>,

    // Timestampts of when we last tested a node
    test_queue: Vec<EdKey>,
}

impl NodePool {
    pub fn new(network: &Network) -> NodePool {
        let all_nodes = Arc::new(Mutex::new(BTreeMap::new()));
        let test_queue = Vec::new();
        NodePool {
            network: network.clone(),
            all_nodes,
            test_queue,
        }
    }

    /// Merge `nodes` new nodes into `all_nodes`
    fn update_pool(all_nodes: &mut SafeNodeMap, incoming: Vec<ServiceNodeRecord>) {
        let mut all_nodes = all_nodes.lock().expect("Failed to lock a mutex");

        // NOTE: I could have just replaced the entire list,
        // but in the future I want to be able to log indifidual
        // changes
        for in_node in incoming {
            let entry = all_nodes.entry(in_node.pubkey_ed25519.clone());

            match entry {
                Entry::Vacant(v) => {
                    v.insert(in_node);
                }
                Entry::Occupied(mut o) => {
                    let _ = std::mem::replace(o.get_mut(), in_node);
                }
            };
        }
    }

    async fn poll_loop(mut all_nodes: SafeNodeMap, network: Network) {

        loop {

            match lokid_api::get_all_service_nodes(&network).await {
                Ok(nodes) => {
                    NodePool::update_pool(&mut all_nodes, nodes);
                },
                Err(error) => {
                    println!("{}", error);
                }
            }

            tokio::time::delay_for(std::time::Duration::from_secs(60)).await;

        }

    }

    /// Initialize the node pool with nodes from the seed and
    /// periodically poll it for updates
    pub async fn connect(&mut self) {

        let all_nodes = self.all_nodes.clone();

        tokio::task::spawn(NodePool::poll_loop(all_nodes, self.network.clone()));

    }

    fn reset_queue(&mut self) {

        let nodes = self.all_nodes.lock().unwrap().clone();

        let mut nodes: Vec<EdKey> = nodes.keys().map(|d| d.clone()).collect();

        let mut rng = thread_rng();
        nodes.shuffle(&mut rng);
        self.test_queue = nodes;
    }

    /// Get nodes that should be tested next
    pub fn get_next_nodes(&mut self, n: u32) -> Vec<ServiceNodeRecord> {
        let mut result = vec![];

        if self.test_queue.is_empty() {
            self.reset_queue();
        }

        let n = usize::min(self.test_queue.len(), n as usize);

        for _ in 0..n {
            // Can unwrap since we just checked the length
            result.push(self.test_queue.pop().unwrap());
        }

        let all_nodes = self.all_nodes.lock().unwrap();

        result
            .iter()
            .map(|edkey| all_nodes.get(edkey))
            .filter(|v| v.is_some())
            .map(|v| v.unwrap().clone())
            .collect()
    }
}
