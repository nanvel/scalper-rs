use super::client::BinanceClient;
use crate::exchanges::binance_base::depth_sync::{DomSync, Market, parse_depth_update};
use crate::models::{
    Candle, CandlesState, Interval, SharedCandlesState, SharedOrderBookState, SharedOrderFlowState,
    Timestamp,
};
use futures_util::stream::StreamExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Minimum time between REST depth snapshot requests while resyncing.
const SNAPSHOT_RETRY_INTERVAL: Duration = Duration::from_secs(1);

pub async fn start_market_stream(
    client: &BinanceClient,
    symbol: &String,
    dom_limit: usize,
    shared_candles_state: SharedCandlesState,
    shared_dom_state: SharedOrderBookState,
    shared_order_flow_state: SharedOrderFlowState,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = format!(
        "wss://stream.binance.com:9443/stream?streams={}@kline_1m/{}@depth@100ms/{}@aggTrade",
        symbol.to_lowercase(),
        symbol.to_lowercase(),
        symbol.to_lowercase(),
    );
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (_write, mut read) = ws_stream.split();

    let mut candles_state_1m = CandlesState::new(60, Interval::M1);
    for c in client
        .get_candles("1m", candles_state_1m.capacity())
        .await?
    {
        candles_state_1m.push(c);
    }

    // Binance's diff-depth procedure: buffer the stream, take a snapshot that
    // bridges it, then check every event's sequence ids and resync on a gap.
    let mut dom_sync = DomSync::new(Market::Spot);
    let mut last_snapshot_request: Option<Instant> = None;

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Some(data) = extract_inner(&text) {
                    if data.get("e").and_then(|e| e.as_str()) == Some("depthUpdate") {
                        let Ok(update) = parse_depth_update(&data.to_string()) else {
                            continue;
                        };
                        {
                            let mut buffer = shared_dom_state.write().unwrap();
                            dom_sync.on_update(update, &mut buffer);
                            if let Some(event_time) = data.get("E").and_then(|t| t.as_u64()) {
                                buffer.updated = Timestamp::from_milliseconds(event_time);
                            }
                        }
                        let throttled = last_snapshot_request
                            .is_some_and(|at| at.elapsed() < SNAPSHOT_RETRY_INTERVAL);
                        if dom_sync.needs_snapshot() && !throttled {
                            last_snapshot_request = Some(Instant::now());
                            match client.get_depth(dom_limit).await {
                                Ok(snapshot)
                                    if !dom_sync.snapshot_is_too_old(snapshot.last_update_id) =>
                                {
                                    let mut buffer = shared_dom_state.write().unwrap();
                                    dom_sync.on_snapshot(
                                        snapshot.last_update_id,
                                        &snapshot.bids,
                                        &snapshot.asks,
                                        &mut buffer,
                                    );
                                }
                                // Older than the buffered events: fetch again on a later event.
                                Ok(_) => {}
                                Err(e) => eprintln!("Depth snapshot error: {}", e),
                            }
                        }
                    } else if let Ok(event) = serde_json::from_value::<AggTradeEvent>(data.clone())
                    {
                        if let (Ok(price), Ok(qty)) = (
                            Decimal::from_str(&event.price),
                            Decimal::from_str(&event.quantity),
                        ) {
                            let mut buffer = shared_order_flow_state.write().unwrap();
                            // If maker == true, buyer is the market maker -> trade was seller-initiated (sell)
                            if event.maker {
                                buffer.sell(price, qty);
                            } else {
                                buffer.buy(price, qty);
                            }
                            buffer.updated = Timestamp::from_milliseconds(event.event_time);
                            buffer.online = true;
                        }
                    } else if let Ok(event) = serde_json::from_value::<KlineEvent>(data.clone()) {
                        let candle = Candle {
                            open_time: Timestamp::from_milliseconds(event.kline.start_time),
                            open: Decimal::from_str(&event.kline.open).unwrap(),
                            high: Decimal::from_str(&event.kline.high).unwrap(),
                            low: Decimal::from_str(&event.kline.low).unwrap(),
                            close: Decimal::from_str(&event.kline.close).unwrap(),
                            volume: Decimal::from_str(&event.kline.volume).unwrap(),
                        };
                        candles_state_1m.push(candle);

                        let mut buffer = shared_candles_state.write().unwrap();
                        if let Some(candle) = candles_state_1m.to_candle(&buffer.interval) {
                            buffer.push(candle);
                            buffer.updated = Timestamp::from_milliseconds(event.event_time);
                            buffer.online = true;
                        };
                    }
                }
            }
            Ok(Message::Close(_)) => {
                let mut buffer = shared_candles_state.write().unwrap();
                buffer.online = false;
                println!("WebSocket closed");
                break;
            }
            Err(e) => {
                let mut buffer = shared_candles_state.write().unwrap();
                buffer.online = false;
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn extract_inner(text: &str) -> Option<serde_json::Value> {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(v) => {
            if v.get("stream").is_some() {
                v.get("data").cloned()
            } else {
                Some(v)
            }
        }
        Err(_) => None,
    }
}

#[derive(Deserialize)]
struct KlineEvent {
    #[serde(rename = "k")]
    kline: KlineData,
    #[serde(rename = "E")]
    event_time: u64,
}

#[derive(Deserialize)]
struct KlineData {
    #[serde(rename = "t")]
    start_time: u64,
    #[serde(rename = "o")]
    open: String,
    #[serde(rename = "h")]
    high: String,
    #[serde(rename = "l")]
    low: String,
    #[serde(rename = "c")]
    close: String,
    #[serde(rename = "v")]
    volume: String,
}

#[derive(Deserialize)]
struct AggTradeEvent {
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "m")]
    maker: bool,
    #[serde(rename = "E")]
    event_time: u64,
}
