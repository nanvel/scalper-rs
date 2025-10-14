use super::timestamp::Timestamp;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::BTreeMap;

pub struct DomState {
    pub bids: BTreeMap<Decimal, Decimal>,
    pub asks: BTreeMap<Decimal, Decimal>,
    pub updated: Timestamp,
    pub online: bool,
}

impl DomState {
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

    pub fn get_asks(&self, n: u32, tick_size: Decimal) -> Vec<(Decimal, Decimal)> {
        let min_price = self.asks.keys().next().cloned().unwrap_or(Decimal::ZERO);
        let min_price = (min_price / tick_size).floor() * tick_size;
        let mut buckets = vec![Decimal::ZERO; n as usize];

        for (&price, &qty) in &self.asks {
            let index = ((price - min_price) / tick_size)
                .to_u32()
                .unwrap_or(u32::MAX);
            if index < n {
                buckets[index as usize] += qty;
            } else {
                break;
            }
        }

        (0..n)
            .map(|i| {
                (
                    min_price + tick_size * Decimal::from(i),
                    buckets[i as usize],
                )
            })
            .collect()
    }

    pub fn get_bids(&self, n: u32, tick_size: Decimal) -> Vec<(Decimal, Decimal)> {
        let max_price = self
            .bids
            .keys()
            .next_back()
            .cloned()
            .unwrap_or(Decimal::ZERO);
        let max_price = (max_price / tick_size).floor() * tick_size;
        let mut buckets = vec![Decimal::ZERO; n as usize];

        for (&price, &qty) in self.bids.iter().rev() {
            let index = ((max_price - price) / tick_size)
                .to_u32()
                .unwrap_or(u32::MAX);
            if index < n {
                buckets[index as usize] += qty;
            } else {
                break;
            }
        }

        (0..n)
            .map(|i| {
                (
                    max_price - tick_size * Decimal::from(i),
                    buckets[i as usize],
                )
            })
            .collect()
    }
}
