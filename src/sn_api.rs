use crate::lokid_api::ServiceNodeRecord;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct SnodeStats {
    pub height: u32,
    pub version: String,
    pub reset_time: u64,
    pub total_stored: u32,
    pub connections_in: u32,
    pub previous_period_store_requests: u32, // stores in the last hour
    pub previous_period_retrieve_requests: u32, // retrieves in the last hour
}

pub type StatsResult = Result<SnodeStats, String>;

pub async fn get_stats(client: &reqwest::Client, sn: &ServiceNodeRecord) -> StatsResult {
    let url = format!("https://{}:{}/get_stats/v1", sn.public_ip, sn.storage_port);

    let res = client.get(&url).send().await.map_err(|err| {
        format!(
            "could not send get_stats to {}:{} ({})",
            sn.public_ip, sn.storage_port, err
        )
    })?;

    let status = res.status();

    let success = status.is_success();

    if !success {
        eprintln!("request failed: {}", &status);
        Err(format!("😵 get_stats request failed: {}", &status))
    } else {
        let res_body = res.text().await.map_err(|err| {
            let msg = format!("Could not get response body: {} for node {}", err, &sn);
            eprintln!("{}", &msg);
            msg
        })?;
        let stats: SnodeStats = serde_json::from_str(&res_body).expect("invalid json");
        Ok(stats)
    }
}
