use super::client::BinanceClient;
use super::market_stream::start_market_stream;
use crate::exchanges::base::exchange::Exchange;
use crate::models::{
    CandlesState, Interval, Log, NewOrder, OpenInterestState, Order, OrderBookState,
    OrderFlowState, SharedCandlesState, SharedOpenInterestState, SharedOrderBookState,
    SharedOrderFlowState, SharedState, Symbol,
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
    messages_sender: Option<Sender<Log>>,
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
        let shared_candles_state = Arc::new(RwLock::new(CandlesState::new(self.candles_limit)));
        let shared_dom_state = Arc::new(RwLock::new(OrderBookState::new()));
        let shared_order_flow_state = Arc::new(RwLock::new(OrderFlowState::new()));
        let shared_open_interest_state = Arc::new(RwLock::new(OpenInterestState::new()));

        let (messages_sender, messages_receiver) = mpsc::channel();
        let (orders_sender, orders_receiver) = mpsc::channel();
        self.messages_sender = Some(messages_sender);
        self.orders_sender = Some(orders_sender);
        self.shared_candles_state = Some(shared_candles_state.clone());

        let symbol = self.client.get_symbol()?;

        self.set_interval(self.interval.clone());

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let symbol_clone = self.symbol.clone();
        let candles_clone = shared_candles_state.clone();
        let dom_clone = shared_dom_state.clone();
        let order_flow_clone = shared_order_flow_state.clone();
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

        Ok((
            symbol,
            SharedState {
                candles: shared_candles_state,
                order_book: shared_dom_state,
                open_interest: shared_open_interest_state,
                order_flow: shared_order_flow_state,
            },
            orders_receiver,
            messages_receiver,
        ))
    }

    fn stop(&mut self) -> () {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn set_interval(&mut self, interval: Interval) -> () {
        if let Some(client) = &self.client {
            let interval_str = match interval {
                Interval::M1 => "1m",
                Interval::M5 => "5m",
                Interval::M15 => "15m",
                Interval::H1 => "1h",
            };

            let candles = client
                .get_candles(&interval_str, self.candles_limit)
                .unwrap_or_else(|err| {
                    eprintln!("Error fetching candles: {}", err);
                    return;
                });

            if let Some(shared_candles_state) = self.shared_candles_state.as_ref() {
                let mut buffer = shared_candles_state.write().unwrap();
                buffer.clear();
                for candle in candles {
                    buffer.push(candle);
                }
            }

            self.interval = interval;
        }
    }

    fn place_order(&self, new_order: NewOrder) -> () {
        if let Some(orders_sender) = &self.orders_sender {
            let client = self.client.clone();
            let sender_clone = orders_sender.clone();
            thread::spawn(move || {
                let order = client.place_order(new_order).unwrap();
                sender_clone.send(order).unwrap();
            });
        }
    }

    fn cancel_order(&self, order: Order) -> () {
        if let Some(orders_sender) = &self.orders_sender {
            let client = self.client.clone();
            let order_id = order.id.clone();
            let sender_clone = orders_sender.clone();
            thread::spawn(move || {
                let order = client.cancel_order(&order_id).unwrap();
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
            messages_sender: None,
            orders_sender: None,
            shared_candles_state: None,
            client,
            stop_tx: None,
            handle: None,
        }
    }
}
