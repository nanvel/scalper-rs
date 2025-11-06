use super::client::BinanceClient;
use super::market_stream::start_market_stream;
use crate::exchanges::base::exchange::Exchange;
use crate::models::{
    Interval, Message, NewOrder, Order, SharedCandlesState, SharedDomState,
    SharedOpenInterestState, SharedOrderFlowState, Symbol,
};
use std::sync::{Arc, mpsc::Sender};
use std::thread;
use std::time::Duration;
use tokio::runtime;
use tokio::sync::oneshot;
use tokio::time::sleep;

pub struct BinanceFuturesExchange {
    name: &'static str,
    symbol: String,
    interval: Interval,
    candles: SharedCandlesState,
    dom: SharedDomState,
    open_interest: SharedOpenInterestState,
    order_flow: SharedOrderFlowState,
    messages_sender: Sender<Message>,
    orders_sender: Sender<Order>,
    access_key: Option<String>,
    secret_key: Option<String>,
    client: BinanceClient,
    stop_tx: Option<oneshot::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Exchange for BinanceFuturesExchange {
    fn start(&mut self) -> Result<Symbol, dyn std::error::Error> {
        self.set_interval(self.interval.clone());

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let symbol_clone = self.symbol.clone();
        let candles_clone = self.candles.clone();
        let dom_clone = self.dom.clone();
        let order_flow_clone = self.order_flow.clone();
        let handle = thread::spawn(move || {
            let rt = runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("Failed to build tokio runtime for streams");

            rt.block_on(async move {
                tokio::select! {
                    res = start_market_stream(
                        symbol_clone,
                        500,
                        candles_clone,
                        dom_clone,
                        order_flow_clone,
                    ) => {
                        if let Err(e) = res {
                            eprintln!("Market stream error: {:?}", e);
                        }
                    }

                    _ = shutdown_rx => {
                        println!("Shutting down market stream listener");
                    }
                }

                sleep(Duration::from_millis(10)).await;
            });
        });

        self.stop_tx = Some(shutdown_tx);
        self.handle = Some(handle);
    }

    fn stop(&mut self) -> () {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn can_trade(&self) -> bool {
        self.access_key.is_some() && self.secret_key.is_some()
    }

    fn set_interval(&mut self, interval: Interval) -> () {
        if let Some(client) = &self.client {
            let interval_str = match interval {
                Interval::M1 => "1m",
                Interval::M5 => "5m",
                Interval::M15 => "15m",
                Interval::H1 => "1h",
            };

            let mut limit = 200;
            {
                let candles_buffer = self.candles.read().unwrap();
                limit = candles_buffer.capacity();
            }

            let candles = client
                .get_candles(&interval_str, limit)
                .unwrap_or_else(|err| {
                    eprintln!("Error fetching candles: {}", err);
                    return;
                });

            {
                let mut buffer = self.candles.write().unwrap();
                buffer.clear();
                for candle in candles {
                    buffer.push(candle);
                }
            }

            self.interval = interval;
        }
    }

    fn place_order(&self, new_order: NewOrder) -> () {
        let client = Arc::clone(&self.client);
        if self.can_trade() {
            thread::spawn(move || {
                let order = client.place_order(new_order).unwrap();
                self.orders_sender.send(order).unwrap();
            });
        }
    }

    fn cancel_order(&self, order: Order) -> () {
        let client = Arc::clone(&self.client);
        if self.can_trade() {
            thread::spawn(move || {
                let order = client.cancel_order(&order.id).unwrap();
                self.orders_sender.send(order).unwrap();
            });
        }
    }
}

impl BinanceFuturesExchange {
    pub fn new(
        symbol: String,
        interval: Interval,
        candles: SharedCandlesState,
        dom: SharedDomState,
        open_interest: SharedOpenInterestState,
        order_flow: SharedOrderFlowState,
        messages_sender: Sender<Message>,
        orders_sender: Sender<Order>,
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Self {
        let client = BinanceClient::new(symbol.clone(), access_key.clone(), secret_key.clone());

        Self {
            name: "Binance USD Futures",
            symbol,
            interval,
            candles,
            dom,
            open_interest,
            order_flow,
            messages_sender,
            orders_sender,
            access_key,
            secret_key,
            client,
            stop_tx: None,
            handle: None,
        }
    }
}
