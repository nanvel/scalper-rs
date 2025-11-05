use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub enum BinanceError {
    HttpError(reqwest::Error),
    AuthError(String),
    ApiError { code: i32, msg: String },
    ParseError(String),
    WebSocketError(String),
    InvalidParameter(String),
}

impl fmt::Display for BinanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinanceError::HttpError(e) => write!(f, "HTTP error: {}", e),
            BinanceError::AuthError(msg) => write!(f, "Auth error: {}", msg),
            BinanceError::ApiError { code, msg } => write!(f, "API error {}: {}", code, msg),
            BinanceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            BinanceError::WebSocketError(msg) => write!(f, "WebSocket error: {}", msg),
            BinanceError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for BinanceError {}

impl From<reqwest::Error> for BinanceError {
    fn from(err: reqwest::Error) -> Self {
        BinanceError::HttpError(err)
    }
}

impl From<serde_json::Error> for BinanceError {
    fn from(err: serde_json::Error) -> Self {
        BinanceError::ParseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, BinanceError>;
