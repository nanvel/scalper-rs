use crate::models::{Log, LogLevel, Order, OrderSide, OrderStatus, OrderType, Timestamp};
use futures_util::stream::StreamExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::mpsc::Sender;
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
    pub orig_qty: Option<Decimal>,
    #[serde(rename = "p")]
    pub price: Option<Decimal>,
    #[serde(rename = "x")]
    pub current_exec_type: Option<String>,
    #[serde(rename = "X")]
    pub current_order_status: Option<String>,
    #[serde(rename = "i")]
    pub order_id: Option<u64>,
    #[serde(rename = "l")]
    pub last_executed_qty: Option<String>,
    #[serde(rename = "z")]
    pub accumulated_executed_qty: Option<Decimal>,
    #[serde(rename = "L")]
    pub last_executed_price: Option<String>,
    #[serde(rename = "T")]
    pub trade_time: Option<u64>,
    #[serde(rename = "ap")]
    pub avg_price: Option<Decimal>,
}

impl ExecutionReport {
    pub fn commission(&self) -> Decimal {
        self.avg_price.unwrap()
            * self.accumulated_executed_qty.as_ref().unwrap()
            * Decimal::new(2, 3)
    }
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
    alerts_sender: Sender<Log>,
    orders_sender: Sender<Order>,
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
                                match &er.current_order_status {
                                    Some(s) if s.eq("FILLED") => {
                                        alerts_sender
                                            .send(Log::new(
                                                LogLevel::Info,
                                                format!("Filled {:?}", er.order_id),
                                                Some(10),
                                            ))
                                            .ok();
                                        let order_side = match &er.side {
                                            Some(s) if s.eq("BUY") => OrderSide::Buy,
                                            Some(s) if s.eq("SELL") => OrderSide::Sell,
                                            _ => panic!("Invalid order side"),
                                        };
                                        let order_type = match &er.order_type {
                                            Some(t) if t.eq("MARKET") => OrderType::Market,
                                            Some(t) if t.eq("LIMIT") => OrderType::Limit,
                                            Some(t) if t.eq("STOP_MARKET") => OrderType::Stop,
                                            _ => panic!("Invalid order type"),
                                        };
                                        let order_status = match &er.current_order_status {
                                            Some(s) if s.eq("NEW") => OrderStatus::Pending,
                                            Some(s) if s.eq("PARTIALLY_FILLED") => {
                                                OrderStatus::Pending
                                            }
                                            _ => OrderStatus::Filled,
                                        };
                                        orders_sender
                                            .send(Order::new(
                                                er.order_id.unwrap_or_default().to_string(),
                                                order_type,
                                                order_side,
                                                order_status,
                                                er.orig_qty.unwrap(),
                                                er.accumulated_executed_qty.unwrap(),
                                                er.price.unwrap(),
                                                er.avg_price.unwrap(),
                                                er.commission(),
                                                Timestamp::from(er.trade_time.unwrap() / 1000),
                                            ))
                                            .ok();
                                    }
                                    _ => {}
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
