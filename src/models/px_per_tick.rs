use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;

const PX_PER_TICK_CHOICES: [&str; 17] = [
    "0.01", "0.02", "0.05", "0.1", "0.2", "0.5", "1", "3", "5", "7", "9", "11", "13", "15", "17",
    "19", "21",
];

pub struct PxPerTick(Decimal);

impl Default for PxPerTick {
    fn default() -> Self {
        Self(Decimal::from_str("1").unwrap())
    }
}

impl PxPerTick {
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    pub fn scale_in(&mut self) {
        if let Some(pos) = PX_PER_TICK_CHOICES
            .iter()
            .position(|&x| Decimal::from_str(x).unwrap() == self.0)
        {
            if pos > 0 {
                self.0 = Decimal::from_str(PX_PER_TICK_CHOICES[pos - 1]).unwrap();
            }
        }
    }

    pub fn scale_out(&mut self) {
        if let Some(pos) = PX_PER_TICK_CHOICES
            .iter()
            .position(|&x| Decimal::from_str(x).unwrap() == self.0)
        {
            if pos + 1 < PX_PER_TICK_CHOICES.len() {
                self.0 = Decimal::from_str(PX_PER_TICK_CHOICES[pos + 1]).unwrap();
            }
        }
    }

    pub fn get(&self) -> Decimal {
        self.0
    }
}
