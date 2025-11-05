pub use super::base::errors::ExchangeError;
pub use super::base::exchange::Exchange;
pub use crate::models::Config;

pub struct ExchangeFactory;

impl ExchangeFactory {
    pub fn create(name: &str, config: &Config) -> Result<dyn Exchange, ExchangeError> {
        match name {
            // "binance_usd_futures" => Ok(BinanceExchange::new()),
            _ => Err(ExchangeError::UnknownExchange(name.to_string())),
        }
    }
}
