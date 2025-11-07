use super::timestamp::Timestamp;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub struct OrderBookState {
    pub bids: BTreeMap<Decimal, Decimal>,
    pub asks: BTreeMap<Decimal, Decimal>,
    pub updated: Timestamp,
    pub online: bool,
}

impl OrderBookState {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            updated: Timestamp::now(),
            online: false,
        }
    }

    pub fn init_snapshot(&mut self, bids: Vec<(Decimal, Decimal)>, asks: Vec<(Decimal, Decimal)>) {
        self.bids = bids.into_iter().collect();
        self.asks = asks.into_iter().collect();
    }

    pub fn update_bid(&mut self, price: Decimal, quantity: Decimal) {
        if quantity.is_zero() {
            self.bids.remove(&price);
        } else {
            self.bids.insert(price, quantity);
        }
    }

    pub fn update_ask(&mut self, price: Decimal, quantity: Decimal) {
        if quantity.is_zero() {
            self.asks.remove(&price);
        } else {
            self.asks.insert(price, quantity);
        }
    }

    pub fn bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }

    pub fn ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.updated = Timestamp::now();
        self.online = false;
    }
}

pub type SharedOrderBookState = Arc<RwLock<OrderBookState>>;
