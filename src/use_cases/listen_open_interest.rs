use crate::models::SharedOpenInterestState;
use std::thread;
use tokio::runtime;
use tokio::time::{Duration, sleep};

pub fn listen_open_interest(open_interest_state: SharedOpenInterestState, symbol: String) {
    thread::spawn(move || {
        let rt = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for open interest");

        rt.block_on(async move {
            // https://fapi.binance.com/futures/data/openInterestHist?symbol=RIVERUSDT&period=5m
            // https://fapi.binance.com/fapi/v1/openInterest?symbol=RIVERUSDT

            loop {
                sleep(Duration::from_secs(5)).await;
            }
        });
    });
}
