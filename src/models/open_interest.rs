use super::{CandlesState, Timestamp};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OpenInterest {
    pub time: Timestamp,
    pub value: Decimal,
}

pub struct OpenInterestState {
    data: Box<[Option<OpenInterest>]>,
    head: usize,
    size: usize,
    capacity: usize,
    pub online: bool,
    pub updated: Timestamp,
}

/// Circular buffer for open interest
impl OpenInterestState {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![None; capacity].into_boxed_slice(),
            head: 0,
            size: 0,
            capacity,
            online: false,
            updated: Timestamp::now(),
        }
    }

    pub fn push(&mut self, open_interest: OpenInterest) {
        if let Some(last_candle) = self.last() {
            if last_candle.time == open_interest.time {
                let index = (self.head + self.capacity - 1) % self.capacity;
                self.data[index] = Some(open_interest);
                return;
            }
        }
        // Add new candle
        self.data[self.head] = Some(open_interest);
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        }
    }

    pub fn to_vec(&self) -> Vec<OpenInterest> {
        let mut result = Vec::with_capacity(self.size);
        for i in 0..self.size {
            let index = (self.head + self.capacity - self.size + i) % self.capacity;
            if let Some(oi) = &self.data[index] {
                result.push(*oi);
            }
        }
        result
    }

    pub fn last(&self) -> Option<OpenInterest> {
        if self.size == 0 {
            return None;
        }
        let index = (self.head + self.capacity - 1) % self.capacity;
        self.data[index]
    }
}

pub type SharedOpenInterestState = Arc<RwLock<OpenInterestState>>;
