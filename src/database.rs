use rusqlite::{params, Connection, NO_PARAMS};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cache::TimestampedStatus;

// Database tables:

struct StatusEntryRaw {
    edkey: String,
    date: u32,
    status: u32,
}

#[derive(Debug)]
pub struct StatusEntry {
    pub edkey: String,
    pub date: std::time::SystemTime,
    pub status: OnlineStatus,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum OnlineStatus {
    ONLINE = 0,
    OFFLINE = 1,
}

pub fn create_or_open() -> Connection {
    let db = Connection::open("snode_stats.db").expect("Could not open the DB");

    db.execute(
        "CREATE TABLE IF NOT EXISTS online_status(
        edkey TEXT,
        date INTEGER NOT NULL,
        status INTEGER NOT NULL
    )",
        NO_PARAMS,
    )
    .expect("could not create or open DB");

    db
}

pub fn record_online_status(db: &Connection, edkey: &str, tstatus: TimestampedStatus) {
    let secs_from_epoch = tstatus.timestamp
        .duration_since(UNIX_EPOCH)
        .expect("Could not get UNIX time")
        .as_secs();

    let secs_from_epoch = secs_from_epoch as u32;

    let status = tstatus.status as u32;

    if let Err(error) = db.execute(
        "INSERT INTO online_status (edkey, date, status) values (?1, ?2, ?3)",
        params![edkey, secs_from_epoch, status],
    ) {
        eprintln!("Could not insert: {}", error);
    }
}

pub fn read_all_online_status_rows(db: &Connection) -> Vec<StatusEntry> {
    let mut stmt = db
        .prepare("SELECT edkey, date, status FROM online_status")
        .expect("Could not prepare");

    let vals = stmt
        .query_map(params![], |row| {
            Ok(StatusEntryRaw {
                edkey: row.get(0)?,
                date: row.get(1)?,
                status: row.get(2)?,
            })
        })
        .expect("Could not parse row");

    let entries: Vec<_> = vals.filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();

    let entries = entries.iter().map(|e| {

        let duration = std::time::Duration::from_secs(e.date as u64);


        let status = match e.status {
            0 => OnlineStatus::ONLINE,
            _ => OnlineStatus::OFFLINE,
        };

        let date = std::time::SystemTime::checked_add(&UNIX_EPOCH, duration).expect("Adding to system time");

        StatusEntry {
            edkey: e.edkey.clone(),
            date,
            status
        }
    }).collect();

    entries
}
