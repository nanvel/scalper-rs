use super::base::errors::ExchangeError;
use super::base::exchange::Exchange;
use super::binance_futures::BinanceFuturesExchange;
use crate::models::{Config, Interval};

pub struct ExchangeFactory;

impl ExchangeFactory {
    pub fn create(
        name: &str,
        symbol: String,
        interval: Interval,
        candles_limit: usize,
        config: &Config,
    ) -> Result<dyn Exchange, ExchangeError> {
        match name {
            "binance_usd_futures" => Ok(BinanceFuturesExchange::new(
                symbol,
                interval,
                candles_limit,
                config.binance_access_key.clone(),
                config.binance_secret_key.clone(),
            )),
            _ => Err(ExchangeError::UnknownExchange(name.to_string())),
        }
    }
}
