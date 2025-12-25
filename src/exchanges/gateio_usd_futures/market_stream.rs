use super::client::GateioClient;
use crate::models::{
    Candle, CandlesState, Interval, SharedCandlesState, SharedOrderBookState, SharedOrderFlowState,
    Timestamp,
};
use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub async fn start_market_stream(
    client: &GateioClient,
    contract: &String,
    _settle: &String,
    dom_limit: usize,
    shared_candles_state: SharedCandlesState,
    shared_dom_state: SharedOrderBookState,
    shared_order_flow_state: SharedOrderFlowState,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = "wss://fx-ws.gateio.ws/v4/ws/usdt";
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to candlesticks, order book, and trades
    let subscribe_candles = serde_json::json!({
        "time": get_timestamp(),
        "channel": "futures.candlesticks",
        "event": "subscribe",
        "payload": ["1m", contract],
    });

    let subscribe_orderbook = serde_json::json!({
        "time": get_timestamp(),
        "channel": "futures.order_book_update",
        "event": "subscribe",
        "payload": [contract, "100ms", dom_limit.to_string()],
    });

    let subscribe_trades = serde_json::json!({
        "time": get_timestamp(),
        "channel": "futures.trades",
        "event": "subscribe",
        "payload": [contract],
    });

    write
        .send(Message::Text(subscribe_candles.to_string().into()))
        .await?;
    write
        .send(Message::Text(subscribe_orderbook.to_string().into()))
        .await?;
    write
        .send(Message::Text(subscribe_trades.to_string().into()))
        .await?;

    let mut candles_state_1m = CandlesState::new(60, Interval::M1);
    for c in client
        .get_candles("1m", candles_state_1m.capacity())
        .await?
    {
        candles_state_1m.push(c);
    }

    let depth_snapshot = client.get_depth(dom_limit).await?;

    {
        // Gate.io futures: swap bids/asks from REST response
        let bids: Vec<(Decimal, Decimal)> = depth_snapshot
            .bids
            .iter()
            .filter_map(|a| {
                let price = Decimal::from_str(&a.p).unwrap();
                let qty = Decimal::from(a.s);
                Some((price, qty))
            })
            .collect();
        let asks: Vec<(Decimal, Decimal)> = depth_snapshot
            .asks
            .iter()
            .filter_map(|b| {
                let price = Decimal::from_str(&b.p).unwrap();
                let qty = Decimal::from(b.s);
                Some((price, qty))
            })
            .collect();

        let mut buffer = shared_dom_state.write().unwrap();
        buffer.init_snapshot(bids, asks);
    }

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(wrapper) = serde_json::from_str::<EventWrapper>(&text) {
                    if wrapper.event == "update" {
                        match wrapper.channel.as_str() {
                            "futures.order_book_update" => {
                                if let Ok(event) =
                                    serde_json::from_value::<OrderBookEvent>(wrapper.result)
                                {
                                    if event.first_update_id >= depth_snapshot.id {
                                        apply_order_book_update(&shared_dom_state, &event);
                                    }
                                }
                            }
                            "futures.trades" => {
                                if let Ok(events) =
                                    serde_json::from_value::<Vec<TradeEvent>>(wrapper.result)
                                {
                                    for event in events {
                                        if let Ok(price) = Decimal::from_str(&event.price) {
                                            let qty = Decimal::from(event.size.abs());
                                            let mut buffer =
                                                shared_order_flow_state.write().unwrap();
                                            if event.size > 0 {
                                                buffer.buy(price, qty);
                                            } else {
                                                buffer.sell(price, qty);
                                            }
                                            buffer.updated =
                                                Timestamp::from_milliseconds(event.create_time);
                                            buffer.online = true;
                                        }
                                    }
                                }
                            }
                            "futures.candlesticks" => {
                                if let Ok(events) =
                                    serde_json::from_value::<Vec<CandleEvent>>(wrapper.result)
                                {
                                    if let Some(event) = events.first() {
                                        let candle = Candle {
                                            open_time: Timestamp::from_seconds(event.t),
                                            open: Decimal::from_str(&event.o).unwrap_or_default(),
                                            high: Decimal::from_str(&event.h).unwrap_or_default(),
                                            low: Decimal::from_str(&event.l).unwrap_or_default(),
                                            close: Decimal::from_str(&event.c).unwrap_or_default(),
                                            volume: Decimal::from(event.v),
                                        };
                                        candles_state_1m.push(candle);

                                        let mut buffer = shared_candles_state.write().unwrap();
                                        if let Some(candle) =
                                            candles_state_1m.to_candle(&buffer.interval)
                                        {
                                            buffer.push(candle);
                                            buffer.updated = Timestamp::now();
                                            buffer.online = true;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
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

fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn apply_order_book_update(shared_dom_state: &SharedOrderBookState, event: &OrderBookEvent) {
    let mut buffer = shared_dom_state.write().unwrap();
    // Gate.io futures: swap 'b' and 'a' - they're reversed from expected
    for b in event.b.iter() {
        if let Ok(price) = Decimal::from_str(&b.p) {
            let qty = Decimal::from(b.s);
            buffer.update_bid(price, qty);
        }
    }
    for a in event.a.iter() {
        if let Ok(price) = Decimal::from_str(&a.p) {
            let qty = Decimal::from(a.s);
            buffer.update_ask(price, qty);
        }
    }
    buffer.updated = Timestamp::from_milliseconds(event.t);
    buffer.online = true;
}

#[derive(Deserialize)]
struct EventWrapper {
    event: String,
    channel: String,
    result: serde_json::Value,
}

#[derive(Deserialize)]
struct CandleEvent {
    t: u64,
    #[serde(rename = "o")]
    o: String,
    #[serde(rename = "h")]
    h: String,
    #[serde(rename = "l")]
    l: String,
    #[serde(rename = "c")]
    c: String,
    #[serde(rename = "v")]
    v: i64,
}

#[derive(Deserialize, Clone)]
struct OrderBookEvent {
    t: u64,
    #[serde(rename = "U")]
    first_update_id: u64,
    #[serde(rename = "b")]
    b: Vec<BookLevel>,
    #[serde(rename = "a")]
    a: Vec<BookLevel>,
}

#[derive(Deserialize, Clone)]
struct BookLevel {
    p: String,
    s: i64,
}

#[derive(Deserialize)]
struct TradeEvent {
    #[serde(rename = "create_time_ms")]
    create_time: u64,
    price: String,
    size: i64,
}
