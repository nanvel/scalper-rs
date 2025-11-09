use super::auth::{build_signed_query, get_timestamp};
use crate::exchanges::base::USER_AGENT;
use crate::exchanges::binance_futures::client::ApiError;
use crate::exchanges::binance_futures::errors::BinanceError;
use crate::models::{Log, LogLevel, Order, OrderSide, OrderStatus, OrderType, Timestamp};
use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use std::io::repeat;
use std::sync::mpsc::Sender;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::client;
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

pub async fn start_orders_stream(
    access_key: Option<String>,
    secret_key: Option<String>,
    symbol: &String,
    logs_sender: Sender<Log>,
    orders_sender: Sender<Order>,
) -> Result<(), Box<dyn std::error::Error>> {
    if access_key.is_none() || secret_key.is_none() {
        loop {
            sleep(Duration::from_secs(5)).await;
        }
    }

    loop {
        let listen_key =
            get_listen_key(&access_key.as_ref().unwrap(), &secret_key.as_ref().unwrap()).await?;

        let ws_url = format!("wss://fstream.binance.com/ws/{}", listen_key);
        let (ws_stream, _) = connect_async(ws_url).await?;
        let (_write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Parse into generic JSON first to handle different Binance wrappers
                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
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
                                    process_filled_order(&er, &logs_sender, &orders_sender)
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
            Timestamp::from_milliseconds(er.trade_time.unwrap()),
        ))
        .ok();
}

async fn get_listen_key(
    access_key: &String,
    secret_key: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    let timestamp = get_timestamp().to_string();
    let params = vec![("timestamp", timestamp)];
    let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let query = build_signed_query(&params_ref, secret_key);
    let url = format!("https://fapi.binance.com/fapi/v1/listenKey?{}", query);

    let http_client = Client::builder().user_agent(USER_AGENT).build()?;
    let response = http_client
        .post(&url)
        .header("X-MBX-APIKEY", access_key)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    if status.is_success() {
        let data: Value = serde_json::from_str(&text)?;
        if let Some(listen_key) = data["listenKey"].as_str() {
            Ok(listen_key.to_string())
        } else {
            Err(Box::new(BinanceError::ParseError(
                "Invalid listenKey".to_string(),
            )))
        }
    } else {
        if let Ok(api_error) = serde_json::from_str::<ApiError>(&text) {
            Err(Box::new(BinanceError::ApiError {
                code: api_error.code,
                msg: api_error.msg,
            }))
        } else {
            Err(Box::new(BinanceError::ApiError {
                code: status.as_u16() as i32,
                msg: text.to_string(),
            }))
        }
    }
}
