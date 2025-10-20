use crate::binance::start_market_stream;
use crate::models::{SharedCandlesState, SharedDomState, SharedOrderFlowState};
use std::thread;
use tokio::runtime;
use tokio::sync::oneshot;
use tokio::time::{Duration, sleep};

pub fn listen_streams(
    candles_state: SharedCandlesState,
    dom_state: SharedDomState,
    order_flow_state: SharedOrderFlowState,
    symbol: String,
    interval: String,
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
            tokio::select! {
                res = start_market_stream(
                    symbol,
                    interval,
                    candles_limit,
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
