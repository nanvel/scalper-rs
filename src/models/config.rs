use super::interval::Interval;
use rust_decimal::{Decimal, prelude::FromStr};

pub struct Config {
    pub dom_width: i32,
    pub order_flow_width: i32,
    pub status_height: i32,
    pub row_height: i32,
    pub border_width: i32,

    pub px_per_tick_choices: Vec<Decimal>,
    pub px_per_tick_initial: Decimal,

    pub candle_interval_initial: Interval,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dom_width: 100,
            order_flow_width: 100,
            status_height: 20,
            row_height: 10,
            border_width: 1,
            px_per_tick_choices: vec![
                Decimal::from_str("0.01").unwrap(),
                Decimal::from_str("0.02").unwrap(),
                Decimal::from_str("0.05").unwrap(),
                Decimal::from_str("0.1").unwrap(),
                Decimal::from_str("0.2").unwrap(),
                Decimal::from_str("0.5").unwrap(),
                Decimal::from_str("1").unwrap(),
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("5").unwrap(),
                Decimal::from_str("7").unwrap(),
                Decimal::from_str("9").unwrap(),
                Decimal::from_str("11").unwrap(),
                Decimal::from_str("13").unwrap(),
                Decimal::from_str("15").unwrap(),
                Decimal::from_str("17").unwrap(),
                Decimal::from_str("19").unwrap(),
                Decimal::from_str("21").unwrap(),
            ],
            px_per_tick_initial: Decimal::from_str("1").unwrap(),
            candle_interval_initial: Interval::M5,
        }
    }
}
