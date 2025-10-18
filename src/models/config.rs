use super::color::Color;
use rust_decimal::{Decimal, prelude::FromStr};

pub struct Config {
    pub online_color: Color,
    pub offline_color: Color,
    pub bullish_color: Color,
    pub bearish_color: Color,
    pub background_color: Color,
    pub text_color: Color,
    pub border_color: Color,
    pub current_price_color: Color,

    pub dom_width: i32,
    pub status_height: i32,
    pub row_height: i32,
    pub border_width: i32,

    pub scales: Vec<Decimal>,
    pub initial_scale: Decimal,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            online_color: Color::GREEN,
            offline_color: Color::RED,
            bullish_color: Color::GREEN,
            bearish_color: Color::RED,
            background_color: Color::WHITE,
            text_color: Color::BLACK,
            border_color: Color::BLACK,
            current_price_color: Color::GRAY,
            dom_width: 100,
            status_height: 20,
            row_height: 10,
            border_width: 1,
            scales: vec![
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
            initial_scale: Decimal::from_str("1").unwrap(),
        }
    }
}
