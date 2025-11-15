use super::client::BinanceClient;
use crate::models::{Log, LogLevel, Order, OrderSide, OrderStatus, OrderType, Timestamp};
use futures_util::stream::StreamExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::time::Duration;
use tokio::time::sleep;
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

pub async fn start_orders_stream(
    client: &BinanceClient,
    symbol: &String,
    logs_sender: Sender<Log>,
    orders_sender: Sender<Order>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !client.has_auth() {
        loop {
            sleep(Duration::from_secs(5)).await;
        }
    }

    loop {
        match client.create_listen_key().await {
            Ok(listen_key) => {
                let ws_url = format!("wss://fstream.binance.com/ws/{}", listen_key);
                let (ws_stream, _) = connect_async(ws_url).await?;
                let (_write, mut read) = ws_stream.split();

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                let event = value.get("e").and_then(|v| v.as_str());
                                if event != Some("ORDER_TRADE_UPDATE") {
                                    continue;
                                }

                                let candidate =
                                    value.get("o").cloned().unwrap_or_else(|| value.clone());

                                if let Ok(er) = serde_json::from_value::<ExecutionReport>(candidate)
                                {
                                    if let Some(sym) = &er.symbol {
                                        if sym.eq_ignore_ascii_case(&symbol) {
                                            process_filled_order(&er, &logs_sender, &orders_sender)
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Message::Close(_frame)) => {
                            sleep(Duration::from_secs(1)).await;
                            break;
                        }
                        Err(e) => {
                            eprintln!("account stream error: {}", e);
                            sleep(Duration::from_secs(5)).await;
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(er) => {
                logs_sender
                    .send(Log::new(LogLevel::Info, format!("{:?}", er), Some(10)))
                    .ok();

                loop {
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
}

fn process_filled_order(
    er: &ExecutionReport,
    logs_sender: &Sender<Log>,
    orders_sender: &Sender<Order>,
) {
    // logs_sender
    //     .send(Log::new(
    //         LogLevel::Info,
    //         format!("Filled {:?}", er.order_id),
    //         Some(10),
    //     ))
    //     .ok();

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
        Some(s) if s.eq("PARTIALLY_FILLED") => OrderStatus::Pending,
        _ => OrderStatus::Filled,
    };

    let rate = match order_type {
        OrderType::Limit => Decimal::from_str("0.0002").unwrap(),
        _ => Decimal::from_str("0.0005").unwrap(),
    };
    let commission = er.accumulated_executed_qty.unwrap() * er.avg_price.unwrap() * rate;

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
            commission,
            Timestamp::from_milliseconds(er.trade_time.unwrap()),
            true,
        ))
        .ok();
}
