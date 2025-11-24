use super::base::errors::ExchangeError;
use super::base::exchange::Exchange;
use super::binance_usdt_futures::BinanceFuturesExchange;
use crate::models::Config;
use crate::models::{Log, Order};
use std::sync::mpsc::Sender;

pub struct ExchangeFactory;

impl ExchangeFactory {
    pub fn create(
        name: &str,
        symbol: String,
        candles_limit: usize,
        config: &Config,
        logs_sender: Sender<Log>,
        orders_sender: Sender<Order>,
    ) -> Result<Box<dyn Exchange>, ExchangeError> {
        match name {
            "binance_usd_futures" => Ok(Box::new(BinanceFuturesExchange::new(
                symbol,
                candles_limit,
                orders_sender,
                logs_sender,
                config.binance_access_key.clone(),
                config.binance_secret_key.clone(),
            ))),
            _ => Err(ExchangeError::UnknownExchange(name.to_string())),
        }
    }
}
