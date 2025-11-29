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

pub async fn start_orders_stream(
    client: &BinanceClient,
    symbol: &String,
    logs_sender: &Sender<Log>,
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
                let ws_url = format!("wss://stream.binance.com:9443/ws/{}", listen_key);
                let (ws_stream, _) = connect_async(ws_url).await?;
                let (_write, mut read) = ws_stream.split();

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                let event = value.get("e").and_then(|v| v.as_str());
                                if event != Some("executionReport") {
                                    continue;
                                }

                                if let Ok(er) = serde_json::from_value::<ExecutionReport>(value) {
                                    if let Some(sym) = &er.symbol {
                                        if sym.eq_ignore_ascii_case(&symbol) {
                                            process_filled_order(&er, &orders_sender)
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
                            logs_sender.send(Log::new(
                                LogLevel::Warning("STREAM".to_string(), None),
                                format!("{:?}", e),
                            ))?;
                            sleep(Duration::from_secs(5)).await;
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(er) => {
                logs_sender.send(Log::new(
                    LogLevel::Error("AUTH".to_string()),
                    format!("{:?}", er),
                ))?;

                loop {
                    sleep(Duration::from_mins(5)).await;
                }
            }
        }
    }
}

fn process_filled_order(er: &ExecutionReport, orders_sender: &Sender<Order>) {
    if let Some(t) = &er.order_type {
        if let Some(s) = &er.current_order_status {
            if t.eq("STOP_LOSS") && s.eq("EXPIRED") {
                return;
            }
        }
    }

    let order_side = match &er.side {
        Some(s) if s.eq("BUY") => OrderSide::Buy,
        Some(s) if s.eq("SELL") => OrderSide::Sell,
        _ => panic!("Invalid order side"),
    };

    let order_type = match &er.order_type {
        Some(t) if t.eq("MARKET") => OrderType::Market,
        Some(t) if t.eq("LIMIT") => OrderType::Limit,
        Some(t) if t.eq("STOP_LOSS") => OrderType::Stop,
        _ => panic!("Invalid order type"),
    };

    let order_status = match &er.current_order_status {
        Some(s) if s.eq("NEW") => OrderStatus::Pending,
        Some(s) if s.eq("PARTIALLY_FILLED") => OrderStatus::Pending,
        _ => OrderStatus::Filled,
    };

    let price = match order_status {
        OrderStatus::Filled => er.avg_price.unwrap_or(Decimal::ZERO),
        OrderStatus::Pending => match order_type {
            OrderType::Stop => er.stop_price.unwrap_or(Decimal::ZERO),
            OrderType::Limit => er.price.unwrap_or(Decimal::ZERO),
            OrderType::Market => Decimal::ZERO,
        },
    };

    let rate = match order_type {
        OrderType::Limit => Decimal::from_str("0.001").unwrap(),
        _ => Decimal::from_str("0.001").unwrap(),
    };
    let commission = er.accumulated_executed_qty.unwrap_or(Decimal::ZERO)
        * er.avg_price.unwrap_or(Decimal::ZERO)
        * rate;

    orders_sender
        .send(Order::new(
            er.order_id.unwrap_or_default().to_string(),
            order_type,
            order_side,
            order_status,
            er.orig_qty.unwrap_or(Decimal::ZERO),
            er.accumulated_executed_qty.unwrap_or(Decimal::ZERO),
            price,
            er.avg_price.unwrap_or(Decimal::ZERO),
            commission,
            Timestamp::from_milliseconds(er.trade_time.unwrap_or_default()),
            true,
        ))
        .ok();
}

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
    #[serde(rename = "n")]
    pub commission: Option<Decimal>,
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "Z")]
    pub avg_price: Option<Decimal>,
    #[serde(rename = "P")]
    pub stop_price: Option<Decimal>,
}
