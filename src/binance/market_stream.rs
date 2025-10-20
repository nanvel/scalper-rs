use crate::models::{Candle, Interval, SharedCandlesState, SharedDomState, SharedOrderFlowState};
use futures_util::stream::StreamExt;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio_tungstenite::tungstenite::http;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug, Deserialize)]
struct KlineEvent {
    #[serde(rename = "k")]
    kline: KlineData,
    #[serde(rename = "E")]
    event_time: u64,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct DepthSnapshot {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
struct DepthUpdateEvent {
    #[serde(rename = "u")]
    update_id: u64,
    #[serde(rename = "b")]
    bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    asks: Vec<[String; 2]>,
    #[serde(rename = "E")]
    event_time: u64,
}

#[derive(Debug, Deserialize)]
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

pub async fn start_market_stream(
    symbol: String,
    interval: Interval,
    candles_limit: usize,
    dom_limit: usize,
    shared_candles_state: SharedCandlesState,
    shared_dom_state: SharedDomState,
    shared_order_flow_state: SharedOrderFlowState,
) -> Result<(), Box<dyn std::error::Error>> {
    let http_client = Client::builder().user_agent("scalper-rs/0.1").build()?;

    // Fetch initial candles
    let candles_url = format!(
        "https://fapi.binance.com/fapi/v1/klines?symbol={}&interval={}&limit={}",
        symbol,
        interval.slug(),
        candles_limit
    );

    let response = http_client.get(&candles_url).send().await?;
    let data: Vec<serde_json::Value> = response.json().await?;

    let initial_candles: Vec<Candle> = data
        .iter()
        .map(|k| Candle {
            open_time: k[0].as_u64().unwrap().into(),
            open: Decimal::from_str(k[1].as_str().unwrap()).unwrap_or(Decimal::ZERO),
            high: Decimal::from_str(k[2].as_str().unwrap()).unwrap_or(Decimal::ZERO),
            low: Decimal::from_str(k[3].as_str().unwrap()).unwrap_or(Decimal::ZERO),
            close: Decimal::from_str(k[4].as_str().unwrap()).unwrap_or(Decimal::ZERO),
            volume: Decimal::from_str(k[5].as_str().unwrap()).unwrap_or(Decimal::ZERO),
        })
        .collect();

    {
        let mut buffer = shared_candles_state.write().unwrap();
        for candle in initial_candles {
            buffer.push(candle);
        }
    }

    // Listen on the WebSocket
    let ws_url = format!(
        "wss://fstream.binance.com/stream?streams={}@kline_{}/{}@depth@100ms/{}@aggTrade",
        symbol.to_lowercase(),
        interval.slug(),
        symbol.to_lowercase(),
        symbol.to_lowercase(),
    );
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (_write, mut read) = ws_stream.split();

    // Fetch initial order book snapshot
    let dom_url = format!(
        "https://fapi.binance.com/fapi/v1/depth?symbol={}&limit={}",
        symbol, dom_limit
    );
    let response = http_client.get(&dom_url).send().await?;
    let snapshot: DepthSnapshot = response.json().await?;

    let bids: Vec<(Decimal, Decimal)> = snapshot
        .bids
        .iter()
        .filter_map(|b| {
            let price = Decimal::from_str(&b[0]).ok()?;
            let qty = Decimal::from_str(&b[1]).ok()?;
            Some((price, qty))
        })
        .collect();
    let asks: Vec<(Decimal, Decimal)> = snapshot
        .asks
        .iter()
        .filter_map(|a| {
            let price = Decimal::from_str(&a[0]).ok()?;
            let qty = Decimal::from_str(&a[1]).ok()?;
            Some((price, qty))
        })
        .collect();

    {
        let mut buffer = shared_dom_state.write().unwrap();
        buffer.init_snapshot(bids, asks);
    }

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Some(data) = extract_inner(&text) {
                    if let Ok(event) = serde_json::from_value::<DepthUpdateEvent>(data.clone()) {
                        let mut buffer = shared_dom_state.write().unwrap();
                        if event.update_id <= snapshot.last_update_id {
                            continue;
                        }
                        for b in event.bids.iter() {
                            if let (Ok(price), Ok(qty)) =
                                (Decimal::from_str(&b[0]), Decimal::from_str(&b[1]))
                            {
                                buffer.update_bid(price, qty);
                            }
                        }
                        for a in event.asks.iter() {
                            if let (Ok(price), Ok(qty)) =
                                (Decimal::from_str(&a[0]), Decimal::from_str(&a[1]))
                            {
                                buffer.update_ask(price, qty);
                            }
                        }
                        buffer.updated = event.event_time.into();
                        buffer.online = true;
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
                            buffer.updated = event.event_time.into();
                            buffer.online = true;
                        }
                    } else if let Ok(event) = serde_json::from_value::<KlineEvent>(data.clone()) {
                        let candle = Candle {
                            open_time: event.kline.start_time.into(),
                            open: Decimal::from_str(&event.kline.open).unwrap_or(Decimal::ZERO),
                            high: Decimal::from_str(&event.kline.high).unwrap_or(Decimal::ZERO),
                            low: Decimal::from_str(&event.kline.low).unwrap_or(Decimal::ZERO),
                            close: Decimal::from_str(&event.kline.close).unwrap_or(Decimal::ZERO),
                            volume: Decimal::from_str(&event.kline.volume).unwrap_or(Decimal::ZERO),
                        };

                        let mut buffer = shared_candles_state.write().unwrap();
                        buffer.push(candle);
                        buffer.updated = event.event_time.into();
                        buffer.online = true;
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
