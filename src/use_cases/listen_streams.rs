use crate::binance::streams::{run_candles_stream, run_dom_stream};
use crate::models::{SharedCandlesState, SharedDomState};
use tokio::time::{Duration, sleep};

pub async fn listen_streams(
    candles_state: SharedCandlesState,
    dom_state: SharedDomState,
    symbol: String,
    interval: String,
    candles_limit: usize,
    dom_limit: usize,
) {
    let symbol_clone = symbol.clone();
    tokio::spawn(async move {
        if let Err(e) =
            run_candles_stream(symbol_clone, interval, candles_limit, candles_state).await
        {
            eprintln!("Candles stream error: {}", e)
        }
    });

    tokio::spawn(async move {
        if let Err(e) = run_dom_stream(symbol, dom_limit, dom_state).await {
            eprintln!("DOM stream error: {}", e)
        }
    });

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
