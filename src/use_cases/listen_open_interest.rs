use crate::models::{SharedOpenInterestState, Timestamp};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::os::macos::raw::stat;
use std::str::FromStr;
use std::thread;
use tokio::runtime;
use tokio::time::{Duration, sleep};

#[derive(Debug, Deserialize)]
struct HistEntry {
    #[serde(rename = "sumOpenInterest")]
    open_interest: String,
    #[serde(rename = "timestamp")]
    timestamp: u64,
}

#[derive(Debug, Deserialize)]
struct CurrentEntry {
    #[serde(rename = "openInterest")]
    open_interest: String,
    #[serde(rename = "time")]
    timestamp: u64,
}

pub fn listen_open_interest(open_interest_state: SharedOpenInterestState, symbol: String) {
    thread::spawn(move || {
        let rt = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for open interest");

        rt.block_on(async move {
            let client = Client::new();

            // Load historical snapshot using serde-deserialized struct
            let snapshot_url = format!(
                "https://fapi.binance.com/futures/data/openInterestHist?symbol={}&period=5m&limit=500",
                symbol
            );

            match client.get(&snapshot_url).send().await {
                Ok(resp) => match resp.json::<Vec<HistEntry>>().await {
                    Ok(entries) => {
                        // push entries oldest -> newest
                        let mut state = open_interest_state.write().unwrap();
                        for oi in entries.iter() {
                            state.push(&Timestamp::from(oi.timestamp / 1000), Decimal::from_str(oi.open_interest.as_str()).unwrap());
                        }
                    }
                    Err(e) => eprintln!("Failed to parse openInterestHist JSON: {}", e),
                },
                Err(e) => eprintln!("Failed to fetch openInterestHist {}: {}", snapshot_url, e),
            }

            // Then poll every 5 seconds for current open interest
            loop {
                let url = format!("https://fapi.binance.com/fapi/v1/openInterest?symbol={}", symbol);
                match client.get(&url).send().await {
                    Ok(resp) => match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Ok(oi) = serde_json::from_value::<CurrentEntry>(json.clone()) {
                                let mut state = open_interest_state.write().unwrap();
                                let ts = Timestamp::from(oi.timestamp / 1000);
                                state.push(&ts, Decimal::from_str(oi.open_interest.as_str()).unwrap());
                                state.online = true;
                                state.updated = ts;
                            } else {
                                eprintln!("Could not parse openInterest JSON: {}", json);
                            }
                        }
                        Err(e) => eprintln!("Failed to parse openInterest response JSON: {}", e),
                    },
                    Err(e) => {
                        eprintln!("Request error fetching openInterest: {}", e);
                        let mut state = open_interest_state.write().unwrap();
                        state.online = false;
                    }
                }

                sleep(Duration::from_secs(5)).await;
            }
        });
    });
}
