use super::Timestamp;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Candle {
    pub open_time: Timestamp,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

impl Candle {
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

pub struct CandlesState {
    data: Box<[Option<Candle>]>,
    head: usize,
    size: usize,
    capacity: usize,
    pub online: bool,
    pub updated: Timestamp,
}

/// Circular buffer for candles
impl CandlesState {
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

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn push(&mut self, candle: Candle) {
        if let Some(last_candle) = self.last() {
            if last_candle.open_time == candle.open_time {
                // Update existing candle
                let index = (self.head + self.capacity - 1) % self.capacity;
                self.data[index] = Some(candle);
                return;
            }
        }
        // Add new candle
        self.data[self.head] = Some(candle);
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        }
    }

    pub fn to_vec(&self) -> Vec<Candle> {
        let mut result = Vec::with_capacity(self.size);
        for i in 0..self.size {
            let index = (self.head + self.capacity - self.size + i) % self.capacity;
            if let Some(candle) = &self.data[index] {
                result.push(*candle);
            }
        }
        result
    }

    pub fn last(&self) -> Option<Candle> {
        if self.size == 0 {
            return None;
        }
        let index = (self.head + self.capacity - 1) % self.capacity;
        self.data[index]
    }

    pub fn clear(&mut self) {
        self.data = vec![None; self.capacity].into_boxed_slice();
        self.head = 0;
        self.size = 0;
        self.online = false;
        self.updated = Timestamp::now();
    }
}

#[cfg(test)]
mod tests {
    use super::{Candle, CandlesState};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn create_candle(
        open_time: u64,
        open: &str,
        high: &str,
        low: &str,
        close: &str,
        volume: &str,
    ) -> Candle {
        Candle {
            open_time: open_time.into(),
            open: Decimal::from_str(open).unwrap(),
            high: Decimal::from_str(high).unwrap(),
            low: Decimal::from_str(low).unwrap(),
            close: Decimal::from_str(close).unwrap(),
            volume: Decimal::from_str(volume).unwrap(),
        }
    }

    #[test]
    fn test_candles_buffer_push_and_to_vec() {
        let mut buffer = CandlesState::new(3);

        let candle1 = create_candle(1, "100.0", "110.0", "90.0", "105.0", "1000.0");
        let candle2 = create_candle(2, "105.0", "115.0", "95.0", "110.0", "1500.0");
        let candle3 = create_candle(3, "110.0", "120.0", "100.0", "115.0", "2000.0");
        let candle4 = create_candle(4, "115.0", "125.0", "105.0", "120.0", "2500.0");
        let candle5 = create_candle(4, "116.0", "125.0", "105.0", "120.0", "2500.0");

        buffer.push(candle1);
        buffer.push(candle2);
        buffer.push(candle3);

        let vec = buffer.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].open_time, candle1.open_time);
        assert_eq!(vec[1].open_time, candle2.open_time);
        assert_eq!(vec[2].open_time, candle3.open_time);

        buffer.push(candle4);
        let vec = buffer.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].open_time, candle2.open_time);
        assert_eq!(vec[1].open_time, candle3.open_time);
        assert_eq!(vec[2].open_time, candle4.open_time);
        assert_eq!(vec[2].open, candle4.open);

        // test update existing
        buffer.push(candle5);
        let vec = buffer.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[1].open_time, candle3.open_time);
        assert_eq!(vec[2].open_time, candle5.open_time);
        assert_eq!(vec[2].open, candle5.open);

        let last_candle = buffer.last().unwrap();
        assert_eq!(last_candle.open_time, candle5.open_time);
    }
}

pub type SharedCandlesState = Arc<RwLock<CandlesState>>;
