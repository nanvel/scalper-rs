use super::client::BinanceClient;
use crate::models::SharedOpenInterestState;
use tokio::time::{Duration, sleep};

pub async fn start_open_interest_stream(
    client: &BinanceClient,
    open_interest_state: SharedOpenInterestState,
) -> Result<(), Box<dyn std::error::Error>> {
    let hist = client.get_open_interest_hist().await?;

    {
        let mut state = open_interest_state.write().unwrap();
        for (ts, oi) in hist.iter() {
            state.push(ts, *oi);
        }
    }

    // Then poll every 5 seconds for current open interest
    loop {
        let (ts, oi) = client.get_open_interest().await?;

        {
            let mut state = open_interest_state.write().unwrap();
            state.push(&ts, oi);
            state.online = true;
            state.updated = ts;
        }

        sleep(Duration::from_secs(5)).await;
    }
}
