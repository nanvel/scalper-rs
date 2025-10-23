use super::Timestamp;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct OpenInterestState {
    data: HashMap<u64, Decimal>,
    pub online: bool,
    pub updated: Timestamp,
}

impl OpenInterestState {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            online: false,
            updated: Timestamp::now(),
        }
    }

    pub fn push(&mut self, time: &Timestamp, open_interest: Decimal) {
        self.data.insert(time.seconds() / 60 * 60, open_interest);
    }

    pub fn get(&self, time: &Timestamp) -> Option<Decimal> {
        match self.data.get(&(time.seconds() / 60 * 60)) {
            Some(oi) => Some(*oi),
            None => None,
        }
    }
}

pub type SharedOpenInterestState = Arc<RwLock<OpenInterestState>>;
