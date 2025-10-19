use rust_decimal::Decimal;
use std::time::{Duration, Instant};

pub struct PxPerTick {
    value: Decimal,
    choices: Vec<Decimal>,
    change_ts: Instant,
}

impl PxPerTick {
    pub fn new(value: Decimal, choices: Vec<Decimal>) -> Self {
        Self {
            value,
            choices,
            change_ts: Instant::now(),
        }
    }

    pub fn scale_in(&mut self) {
        if Instant::now() - self.change_ts > Duration::from_millis(100) {
            if let Some(pos) = self.choices.iter().position(|&x| x == self.value) {
                if pos > 0 {
                    self.value = self.choices[pos - 1];
                }
            }
            self.change_ts = Instant::now();
        }
    }

    pub fn scale_out(&mut self) {
        if Instant::now() - self.change_ts > Duration::from_millis(100) {
            if let Some(pos) = self.choices.iter().position(|&x| x == self.value) {
                if pos + 1 < self.choices.len() {
                    self.value = self.choices[pos + 1];
                }
            }
            self.change_ts = Instant::now();
        }
    }

    pub fn get(&self) -> Decimal {
        self.value
    }
}
