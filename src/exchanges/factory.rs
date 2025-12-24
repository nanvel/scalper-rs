use super::base::errors::ExchangeError;
use super::base::exchange::Exchange;
use super::binance_spot::BinanceSpotExchange;
use super::binance_usd_futures::BinanceUSDFuturesExchange;
use super::gateio_usd_futures::GateioUSDFuturesExchange;
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
            "binance_usd_futures" => Ok(Box::new(BinanceUSDFuturesExchange::new(
                symbol,
                candles_limit,
                orders_sender,
                logs_sender,
                config.binance_access_key.clone(),
                config.binance_secret_key.clone(),
            ))),
            "binance_spot" => Ok(Box::new(BinanceSpotExchange::new(
                symbol,
                candles_limit,
                orders_sender,
                logs_sender,
                config.binance_access_key.clone(),
                config.binance_secret_key.clone(),
            ))),
            "gateio_usd_futures" => Ok(Box::new(GateioUSDFuturesExchange::new(
                symbol,
                candles_limit,
                orders_sender,
                logs_sender,
                config.gateio_access_key.clone(),
                config.gateio_secret_key.clone(),
            ))),
            _ => Err(ExchangeError::UnknownExchange(name.to_string())),
        }
    }
}
