use super::base::errors::ExchangeError;
use super::base::exchange::Exchange;
use super::binance_futures::BinanceFuturesExchange;
use crate::models::{
    Config, Interval, Message, Order, SharedCandlesState, SharedDomState, SharedOpenInterestState,
    SharedOrderFlowState,
};
use std::sync::mpsc::Sender;

pub struct ExchangeFactory;

impl ExchangeFactory {
    pub fn create(
        name: &str,
        symbol: String,
        interval: Interval,
        candles: SharedCandlesState,
        dom: SharedDomState,
        open_interest: SharedOpenInterestState,
        order_flow: SharedOrderFlowState,
        messages_sender: Sender<Message>,
        orders_sender: Sender<Order>,
        config: &Config,
    ) -> Result<dyn Exchange, ExchangeError> {
        match name {
            "binance_usd_futures" => Ok(BinanceFuturesExchange::new(
                symbol,
                interval,
                candles,
                dom,
                open_interest,
                order_flow,
                messages_sender,
                orders_sender,
                config.binance_access_key.clone(),
                config.binance_secret_key.clone(),
            )),
            _ => Err(ExchangeError::UnknownExchange(name.to_string())),
        }
    }
}
