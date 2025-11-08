use super::Timestamp;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::OpenInterestState;
    use super::Timestamp;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_open_interest_push_and_get() {
        let mut oi_state = OpenInterestState::new();
        let ts1 = Timestamp::from_milliseconds(1625079600000); // 2021-06-30 15:00:00 UTC
        let ts2 = Timestamp::from_milliseconds(1625083200000); // 2021-06-30 16:00:00 UTC

        let oi1 = Decimal::from_str("12345.67").unwrap();
        let oi2 = Decimal::from_str("23456.78").unwrap();

        oi_state.push(&ts1, oi1);
        oi_state.push(&ts2, oi2);

        assert_eq!(oi_state.get(&ts1), Some(oi1));
        assert_eq!(oi_state.get(&ts2), Some(oi2));
        assert_eq!(
            oi_state.get(&Timestamp::from_milliseconds(1625086800000)),
            None
        ); // 2021-06-30 17:00:00 UTC
    }
}
