use futures_util::stream::StreamExt;
use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutionReport {
    #[serde(rename = "e")]
    pub event_type: Option<String>,
    #[serde(rename = "E")]
    pub event_time: Option<u64>,
    #[serde(rename = "s")]
    pub symbol: Option<String>,
    #[serde(rename = "S")]
    pub side: Option<String>,
    #[serde(rename = "o")]
    pub order_type: Option<String>,
    #[serde(rename = "q")]
    pub orig_qty: Option<String>,
    #[serde(rename = "p")]
    pub price: Option<String>,
    #[serde(rename = "x")]
    pub current_exec_type: Option<String>,
    #[serde(rename = "X")]
    pub current_order_status: Option<String>,
    #[serde(rename = "i")]
    pub order_id: Option<u64>,
    #[serde(rename = "l")]
    pub last_executed_qty: Option<String>,
    #[serde(rename = "z")]
    pub accumulated_executed_qty: Option<String>,
    #[serde(rename = "L")]
    pub last_executed_price: Option<String>,
    #[serde(rename = "n")]
    pub commission: Option<String>,
    #[serde(rename = "T")]
    pub trade_time: Option<u64>,
}

/// Start listening for account/order updates on the Binance futures user data stream.
///
/// - `listen_key`: the Binance user data listenKey (created via REST).
/// - `symbol`: symbol to filter updates for (case-insensitive).
/// - `tx`: sender to forward matched `ExecutionReport` events.
///
/// The function runs until the WebSocket closes or an error occurs.
pub async fn start_account_stream(
    listen_key: String,
    symbol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = format!("wss://fstream.binance.com/ws/{}", listen_key);
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (_write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse into generic JSON first to handle different Binance wrappers
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Some user stream events carry the order under "o" (e.g. ORDER_TRADE_UPDATE),
                    // others may be direct executionReport-like objects. Try both.
                    let candidate = if value.get("o").is_some() {
                        value.get("o").cloned().unwrap_or_else(|| value.clone())
                    } else {
                        value
                    };

                    if let Ok(er) = serde_json::from_value::<ExecutionReport>(candidate) {
                        if let Some(sym) = &er.symbol {
                            if sym.eq_ignore_ascii_case(&symbol) {
                                if matches!(
                                    er.event_type.as_deref(),
                                    Some("NEW") | Some("PARTIAL_FILL")
                                ) {
                                    dbg!(er);
                                } else {
                                    dbg!(er);
                                }
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_frame)) => {
                break;
            }
            Err(e) => {
                eprintln!("account stream error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
