use super::interval::Interval;
use rust_decimal::{Decimal, prelude::FromStr};

pub struct Config {
    pub dom_width: i32,
    pub order_flow_width: i32,
    pub status_height: i32,
    pub row_height: i32,
    pub border_width: i32,

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
            px_per_tick_initial: Decimal::from_str("1").unwrap(),
            candle_interval_initial: Interval::M5,
        }
    }
}
