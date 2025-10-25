mod auth;
mod client;
mod errors;
mod market_stream;
pub mod types;

pub use client::BinanceClient;
pub use market_stream::start_market_stream;
