use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::{self, OnlineStatus, StatusEntry};
use crate::node_pool::EdKey;

use crate::sn_api::SnodeStats;

#[derive(Clone, Debug)]
pub struct TimestampedStatus {
    pub status: OnlineStatus,
    pub timestamp: SystemTime,
}


// TODO: add long-term stats here too

// - swarm id (from lokid)
// - 

/// A layer above the database that keeps track of
/// when the nodes if online/offline and only makes
/// necessary writes
pub struct Cache {
    /// A map from node to its latest status
    pub status_map: HashMap<EdKey, TimestampedStatus>,
    /// Keeps the latest stats received (could probably merge this with status_map)
    pub stats_map: HashMap<EdKey, SnodeStats>,
    db: rusqlite::Connection,
}

fn earlier(t1: &SystemTime, t2: &SystemTime) -> bool {
    let secs1 = t1.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let secs2 = t2.duration_since(UNIX_EPOCH).unwrap().as_secs();

    secs1 < secs2
}

impl Cache {
    pub fn new() -> Cache {
        let db = database::create_or_open();

        // read entries and populate the cache

        // let db = database::create_or_open();
        Cache {
            status_map: HashMap::new(),
            stats_map: HashMap::new(),
            db,
        }
    }

    fn save_latest_status(&mut self, entries: &Vec<StatusEntry>) {
        for entry in entries {
            let key = &entry.edkey;

            match self.status_map.get(key) {
                Some(tstatus) => {
                    if earlier(&tstatus.timestamp, &entry.date) {
                        // new entry is later, updating
                        // This is pretty slow, but should be fine for now
                        self.status_map.insert(
                            key.clone(),
                            TimestampedStatus {
                                timestamp: entry.date,
                                status: entry.status.clone(),
                            },
                        );
                    }
                }
                None => {
                    self.status_map.insert(
                        key.clone(),
                        TimestampedStatus {
                            timestamp: entry.date,
                            status: entry.status.clone(),
                        },
                    );
                }
            };
        }
    }

    pub fn load(&mut self) {
        let rows = database::read_all_online_status_rows(&self.db);
        self.save_latest_status(&rows)
    }

    fn do_update_status(&mut self, node_edkey: &EdKey, status: OnlineStatus) {
        // TODO: we don't have to make a copy when we are only updating the value

        let v = TimestampedStatus {
            timestamp: SystemTime::now(),
            status,
        };

        self.status_map.insert(node_edkey.clone(), v.clone());

        database::record_online_status(&self.db, &node_edkey, v);
    }

    pub fn update_status(&mut self, node_edkey: &EdKey, status: OnlineStatus) {
        match self.status_map.get(node_edkey) {
            Some(v) => {
                if v.status != status {
                    println!("Status changed for {}: {:?}", node_edkey, &status);
                    self.do_update_status(&node_edkey, status);
                }
            }
            None => {
                // println!("Inserting status for the first time for: {}", &node_edkey);
                self.do_update_status(&node_edkey, status);
            }
        }
    }

    pub fn update_node_stats(&mut self, edkey: &EdKey, stats: &SnodeStats) {
        self.stats_map.insert(edkey.clone(), stats.clone());
    }
}
