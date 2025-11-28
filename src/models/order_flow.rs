use super::timestamp::Timestamp;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub struct OrderFlowState {
    pub buys: BTreeMap<Decimal, Decimal>,
    pub sells: BTreeMap<Decimal, Decimal>,
    pub updated: Timestamp,
    pub online: bool,
}

impl OrderFlowState {
    pub fn new() -> Self {
        Self {
            buys: BTreeMap::new(),
            sells: BTreeMap::new(),
            updated: Timestamp::now(),
            online: false,
        }
    }

    pub fn buy(&mut self, price: Decimal, quantity: Decimal) {
        self.buys.insert(
            price,
            self.buys.get(&price).unwrap_or(&Decimal::ZERO) + quantity,
        );
    }

    pub fn sell(&mut self, price: Decimal, quantity: Decimal) {
        self.sells.insert(
            price,
            self.sells.get(&price).unwrap_or(&Decimal::ZERO) + quantity,
        );
    }

    pub fn get_balance(&self) -> Decimal {
        let total_buy: Decimal = self.buys.values().cloned().sum();
        let total_sell: Decimal = self.sells.values().cloned().sum();
        if (total_buy + total_sell) == Decimal::ZERO {
            return Decimal::from(50) / Decimal::from(100);
        }
        total_buy / (total_buy + total_sell)
    }
}

pub type SharedOrderFlowState = Arc<RwLock<OrderFlowState>>;
