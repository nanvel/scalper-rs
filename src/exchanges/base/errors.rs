use std::fmt::Display;

#[derive(Debug)]
pub enum ExchangeError {
    UnknownExchange(String),
}

impl Display for ExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExchangeError::UnknownExchange(name) => write!(f, "Unknown exchange: {}", name),
        }
    }
}

impl std::error::Error for ExchangeError {}
