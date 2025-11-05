mod account_stream;
mod auth;
mod client;
mod errors;
pub mod types;

pub use crate::exchanges::binance_futures::market_stream::start_market_stream;
pub use account_stream::start_account_stream;
pub use client::BinanceClient;
