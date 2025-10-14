use crate::data::DomState;
use futures_util::stream::StreamExt;
use reqwest;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub type DomStore = Arc<RwLock<DomState>>;

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

pub async fn start_dom_stream(symbol: String, limit: usize) -> DomStore {
    let dom_store = Arc::new(RwLock::new(DomState::new()));
    let dom_store_clone = dom_store.clone();

    tokio::spawn(async move {
        if let Err(e) = run_dom_stream(symbol, limit, dom_store_clone).await {
            eprintln!("DOM stream error: {}", e)
        }
    });

    dom_store
}

async fn run_dom_stream(
    symbol: String,
    limit: usize,
    dom_store: DomStore,
) -> Result<(), Box<dyn std::error::Error>> {
    // Listen on the WebSocket
    let ws_url = format!(
        "wss://fstream.binance.com/ws/{}@depth@100ms",
        symbol.to_lowercase()
    );
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (_write, mut read) = ws_stream.split();

    // Fetch initial snapshot
    let url = format!(
        "https://fapi.binance.com/fapi/v1/depth?symbol={}&limit={}",
        symbol, limit
    );
    let response = reqwest::get(&url).await?;
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
        let mut buffer = dom_store.write().await;
        buffer.init_snapshot(bids, asks);
        buffer.online = true;
    }

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<DepthUpdateEvent>(&text) {
                    let mut buffer = dom_store.write().await;
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
                }
            }
            Ok(Message::Close(_)) => {
                let mut buffer = dom_store.write().await;
                buffer.online = false;
                println!("WebSocket closed");
                break;
            }
            Err(e) => {
                let mut buffer = dom_store.write().await;
                buffer.online = false;
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
