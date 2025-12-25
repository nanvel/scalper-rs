use super::client::GateioClient;
use crate::models::{SharedOpenInterestState, Timestamp};
use rust_decimal::Decimal;
use tokio::time::{Duration, sleep};

pub async fn start_open_interest_stream(
    client: &GateioClient,
    _contract: &String,
    _settle: &String,
    shared_open_interest_state: SharedOpenInterestState,
) -> Result<(), Box<dyn std::error::Error>> {
    let get_oi = async |limit| match client.get_contract_stats(limit).await {
        Ok(contract_stats_list) => {
            let mut buffer = shared_open_interest_state.write().unwrap();
            let mut last_time = 0;

            for contract_stats in &contract_stats_list {
                last_time = contract_stats.time;
                let value = Decimal::from(contract_stats.open_interest);
                let timestamp = Timestamp::from_seconds(contract_stats.time);

                buffer.push(&timestamp, value);
            }

            if last_time > 0 {
                buffer.updated = Timestamp::from_seconds(last_time);
                buffer.online = true;
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch open interest: {:?}", e);
            let mut buffer = shared_open_interest_state.write().unwrap();
            buffer.online = false;
        }
    };

    get_oi(1000).await;

    loop {
        sleep(Duration::from_secs(20)).await;

        get_oi(5).await;
    }
}
