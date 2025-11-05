use crate::binance::start_market_stream;
use crate::models::{Candle, Interval, SharedCandlesState, SharedDomState, SharedOrderFlowState};
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::thread;
use tokio::runtime;
use tokio::sync::oneshot;
use tokio::time::{Duration, sleep};

pub fn listen_streams(
    candles_state: SharedCandlesState,
    dom_state: SharedDomState,
    order_flow_state: SharedOrderFlowState,
    symbol: String,
    interval: Interval,
    candles_limit: usize,
    dom_limit: usize,
) -> (thread::JoinHandle<()>, oneshot::Sender<()>) {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let handle = thread::spawn(move || {
        let rt = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for streams");

        rt.block_on(async move {
            let http_client = Client::builder()
                .user_agent("scalper-rs/0.1")
                .build()
                .unwrap();

            // Fetch initial candles
            let candles_url = format!(
                "https://fapi.binance.com/fapi/v1/klines?symbol={}&interval={}&limit={}",
                symbol,
                interval.slug(),
                candles_limit
            );

            let response = http_client.get(&candles_url).send().await.unwrap();
            let data: Vec<serde_json::Value> = response.json().await.unwrap();

            let initial_candles: Vec<Candle> = data
                .iter()
                .map(|k| Candle {
                    open_time: (k[0].as_u64().unwrap() / 1000).into(),
                    open: Decimal::from_str(k[1].as_str().unwrap()).unwrap_or(Decimal::ZERO),
                    high: Decimal::from_str(k[2].as_str().unwrap()).unwrap_or(Decimal::ZERO),
                    low: Decimal::from_str(k[3].as_str().unwrap()).unwrap_or(Decimal::ZERO),
                    close: Decimal::from_str(k[4].as_str().unwrap()).unwrap_or(Decimal::ZERO),
                    volume: Decimal::from_str(k[5].as_str().unwrap()).unwrap_or(Decimal::ZERO),
                })
                .collect();

            {
                let mut buffer = candles_state.write().unwrap();
                for candle in initial_candles {
                    buffer.push(candle);
                }
            }

            tokio::select! {
                res = start_market_stream(
                    symbol,
                    dom_limit,
                    candles_state,
                    dom_state,
                    order_flow_state,
                ) => {
                    if let Err(e) = res {
                        eprintln!("Market stream error: {:?}", e);
                    }
                }

                _ = shutdown_rx => {
                    println!("Shutting down market stream listener");
                }
            }

            sleep(Duration::from_millis(10)).await;
        });
    });

    (handle, shutdown_tx)
}
