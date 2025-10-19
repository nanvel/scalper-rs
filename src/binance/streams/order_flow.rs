use crate::models::{OrderFlowState, SharedOrderFlowState};
use futures_util::stream::StreamExt;
use reqwest;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

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

/// Run an aggTrade websocket stream for `symbol` and update `shared_order_flow_state`.
pub async fn run_order_flow_stream(
    symbol: String,
    shared_order_flow_state: SharedOrderFlowState,
) -> Result<(), Box<dyn std::error::Error>> {
    // Listen on the WebSocket
    let ws_url = format!(
        "wss://fstream.binance.com/ws/{}@aggTrade",
        symbol.to_lowercase()
    );
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (_write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<AggTradeEvent>(&text) {
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
                }
            }
            Ok(Message::Close(_)) => {
                let mut buffer = shared_order_flow_state.write().unwrap();
                buffer.online = false;
                println!("WebSocket closed");
                break;
            }
            Err(e) => {
                let mut buffer = shared_order_flow_state.write().unwrap();
                buffer.online = false;
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
