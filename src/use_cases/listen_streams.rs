use crate::binance::start_market_stream;
use crate::models::{SharedCandlesState, SharedDomState, SharedOrderFlowState};
use std::thread;
use tokio::runtime;
use tokio::time::{Duration, sleep};

pub fn listen_streams(
    candles_state: SharedCandlesState,
    dom_state: SharedDomState,
    order_flow_state: SharedOrderFlowState,
    symbol: String,
    interval: String,
    candles_limit: usize,
    dom_limit: usize,
) {
    thread::spawn(move || {
        let rt = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for streams");

        rt.block_on(async {
            tokio::spawn(async move {
                if let Err(e) = start_market_stream(
                    symbol,
                    interval,
                    candles_limit,
                    dom_limit,
                    candles_state,
                    dom_state,
                    order_flow_state,
                )
                .await
                {
                    eprintln!("Market stream error: {}", e)
                }

                loop {
                    sleep(Duration::from_secs(1)).await;
                }
            });
        });
    });
}
