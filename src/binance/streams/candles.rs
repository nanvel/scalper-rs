use crate::models::{Candle, CandlesState, Timestamp};
use futures_util::stream::StreamExt;
use reqwest;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub type SharedCandlesState = Arc<RwLock<CandlesState>>;

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

pub async fn start_candles_stream(
    symbol: String,
    interval: String,
    limit: usize,
) -> SharedCandlesState {
    let candles_store = Arc::new(RwLock::new(CandlesState::new(limit)));
    let candles_store_clone = candles_store.clone();

    tokio::spawn(async move {
        if let Err(e) = run_stream(symbol, interval, limit, candles_store_clone).await {
            eprintln!("Candles stream error: {}", e)
        }
    });

    candles_store
}

async fn run_stream(
    symbol: String,
    interval: String,
    limit: usize,
    shared_candles_state: SharedCandlesState,
) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch initial candles
    let url = format!(
        "https://fapi.binance.com/fapi/v1/klines?symbol={}&interval={}&limit={}",
        symbol, interval, limit
    );
    let response = reqwest::get(&url).await?;
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
        "wss://fstream.binance.com/ws/{}@kline_{}",
        symbol.to_lowercase(),
        interval
    );
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (_write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<KlineEvent>(&text) {
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
