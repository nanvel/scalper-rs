use super::client::BinanceClient;
use super::market_stream::start_market_stream;
use super::open_interest_stream::start_open_interest_stream;
use super::orders_stream::start_orders_stream;
use crate::exchanges::base::exchange::Exchange;
use crate::models::{
    CandlesState, Interval, Log, NewOrder, OpenInterestState, Order, OrderBookState,
    OrderFlowState, SharedCandlesState, SharedState, Symbol,
};
use std::sync::{Arc, RwLock, mpsc, mpsc::Receiver, mpsc::Sender};
use std::thread;
use std::time::Duration;
use tokio::runtime;
use tokio::sync::oneshot;
use tokio::time::sleep;

pub struct BinanceFuturesExchange {
    name: &'static str,
    symbol: String,
    interval: Interval,
    candles_limit: usize,
    access_key: Option<String>,
    secret_key: Option<String>,
    logs_sender: Option<Sender<Log>>,
    orders_sender: Option<Sender<Order>>,
    shared_candles_state: Option<SharedCandlesState>,
    client: Arc<BinanceClient>,
    stop_tx: Option<oneshot::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Exchange for BinanceFuturesExchange {
    fn start(
        &mut self,
    ) -> Result<(Symbol, SharedState, Receiver<Order>, Receiver<Log>), Box<dyn std::error::Error>>
    {
        let shared_candles_state = Arc::new(RwLock::new(CandlesState::new(
            self.candles_limit,
            self.interval.clone(),
        )));
        let shared_order_book_state = Arc::new(RwLock::new(OrderBookState::new()));
        let shared_order_flow_state = Arc::new(RwLock::new(OrderFlowState::new()));
        let shared_open_interest_state = Arc::new(RwLock::new(OpenInterestState::new()));

        let (logs_sender, logs_receiver) = mpsc::channel();
        let (orders_sender, orders_receiver) = mpsc::channel();
        self.logs_sender = Some(logs_sender);
        self.orders_sender = Some(orders_sender);
        self.shared_candles_state = Some(shared_candles_state.clone());

        let symbol = self.client.get_symbol_sync()?;

        self.set_interval(self.interval.clone());

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let symbol_clone = self.symbol.clone();
        let candles_clone = shared_candles_state.clone();
        let order_book_clone = shared_order_book_state.clone();
        let order_flow_clone = shared_order_flow_state.clone();
        let open_interest_clone = shared_open_interest_state.clone();

        let logs_sender_clone = self.logs_sender.clone().unwrap();
        let orders_sender_clone = self.orders_sender.clone().unwrap();

        let client_clone = self.client.clone();

        let keep_listen_key_alive = async |client: &BinanceClient| {
            loop {
                sleep(Duration::from_mins(30)).await;
                if !client.has_auth() {
                    let _ = client.refresh_listen_key().await;
                }
            }
        };

        let handle = thread::spawn(move || {
            let rt = runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("Failed to build tokio runtime for streams");

            rt.block_on(async move {
                tokio::select! {
                    res = start_market_stream(
                        &client_clone,
                        &symbol_clone,
                        500,
                        candles_clone,
                        order_book_clone,
                        order_flow_clone,
                    ) => {
                        if let Err(e) = res {
                            eprintln!("Market stream error: {:?}", e);
                        }
                    }

                    res = start_open_interest_stream(
                        &client_clone,
                        open_interest_clone,
                    ) => {
                        if let Err(e) = res {
                            eprintln!("Open interest stream error: {:?}", e);
                        }
                    }

                    res = start_orders_stream(
                        &client_clone,
                        &symbol_clone,
                        logs_sender_clone,
                        orders_sender_clone,
                    ) => {
                        if let Err(e) = res {
                            eprintln!("Orders stream error: {:?}", e);
                        }
                    }

                    _ = keep_listen_key_alive(&client_clone) => {}

                    _ = shutdown_rx => {
                        println!("Shutting down market stream listener");
                    }
                }

                sleep(Duration::from_millis(10)).await;
            });
        });

        self.stop_tx = Some(shutdown_tx);
        self.handle = Some(handle);

        Ok((
            symbol,
            SharedState {
                candles: shared_candles_state,
                order_book: shared_order_book_state,
                open_interest: shared_open_interest_state,
                order_flow: shared_order_flow_state,
            },
            orders_receiver,
            logs_receiver,
        ))
    }

    fn stop(&mut self) -> () {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        self.stop_tx = None;
        self.handle = None;
    }

    fn set_interval(&mut self, interval: Interval) -> () {
        let interval_str = match interval {
            Interval::M1 => "1m",
            Interval::M5 => "5m",
            Interval::M15 => "15m",
            Interval::H1 => "1h",
        };

        let candles = self
            .client
            .get_candles_sync(&interval_str, self.candles_limit)
            .unwrap();

        if let Some(shared_candles_state) = self.shared_candles_state.as_ref() {
            let mut buffer = shared_candles_state.write().unwrap();
            buffer.clear(interval.clone());
            for candle in candles {
                buffer.push(candle);
            }
        }

        self.interval = interval;
    }

    fn place_order(&self, new_order: NewOrder) -> () {
        if let Some(orders_sender) = &self.orders_sender {
            let client = self.client.clone();
            let sender_clone = orders_sender.clone();
            thread::spawn(move || {
                let order = client.place_order_sync(new_order).unwrap();
                sender_clone.send(order).unwrap();
            });
        }
    }

    fn cancel_order(&self, order_id: String) -> () {
        if let Some(orders_sender) = &self.orders_sender {
            let client = self.client.clone();
            let sender_clone = orders_sender.clone();
            thread::spawn(move || {
                let order = client.cancel_order_sync(&order_id).unwrap();
                sender_clone.send(order).unwrap();
            });
        }
    }
}

impl BinanceFuturesExchange {
    pub fn new(
        symbol: String,
        interval: Interval,
        candles_limit: usize,
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Self {
        let client = Arc::new(BinanceClient::new(
            symbol.clone(),
            access_key.clone(),
            secret_key.clone(),
        ));

        Self {
            name: "Binance USD Futures",
            symbol,
            interval,
            candles_limit,
            access_key,
            secret_key,
            logs_sender: None,
            orders_sender: None,
            shared_candles_state: None,
            client,
            stop_tx: None,
            handle: None,
        }
    }
}
