use crate::exchanges::base::USER_AGENT;
use crate::models::{Candle, Symbol, Timestamp};
use hmac::{Hmac, Mac};
use reqwest::{Client, Response};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sha2::Sha512;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

const BASE_URL: &str = "https://api.gateio.ws/api/v4";

type HmacSha512 = Hmac<Sha512>;

pub struct GateioClient {
    client: Client,
    settle: String,
    contract: String,
    access_key: Option<String>,
    secret_key: Option<String>,
    runtime: Runtime,
}

#[derive(Debug)]
pub enum GateioError {
    ReqwestError(reqwest::Error),
    ParseError(String),
    ApiError { label: String, message: String },
    AuthError(String),
}

impl From<reqwest::Error> for GateioError {
    fn from(err: reqwest::Error) -> Self {
        GateioError::ReqwestError(err)
    }
}

impl std::fmt::Display for GateioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateioError::ReqwestError(e) => write!(f, "Request error: {}", e),
            GateioError::ParseError(e) => write!(f, "Parse error: {}", e),
            GateioError::ApiError { label, message } => {
                write!(f, "API error {}: {}", label, message)
            }
            GateioError::AuthError(e) => write!(f, "Auth error: {}", e),
        }
    }
}

impl std::error::Error for GateioError {}

type Result<T> = std::result::Result<T, GateioError>;

impl GateioClient {
    pub fn new(contract: String, access_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
            settle: "usdt".to_string(),
            contract,
            access_key,
            secret_key,
            runtime: Runtime::new().expect("Failed to create GateioClient Tokio runtime"),
        }
    }

    pub fn has_auth(&self) -> bool {
        self.access_key.is_some() && self.secret_key.is_some()
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn sign_request(
        method: &str,
        path: &str,
        query: &str,
        body_hash: &str,
        timestamp: u64,
        secret: &str,
    ) -> String {
        let payload = format!(
            "{}\n{}\n{}\n{}\n{}",
            method, path, query, body_hash, timestamp
        );

        let mut mac =
            HmacSha512::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&text).map_err(|e| {
                GateioError::ParseError(format!("Failed to parse response: {}. Body: {}", e, text))
            })
        } else {
            if let Ok(api_error) = serde_json::from_str::<ApiError>(&text) {
                Err(GateioError::ApiError {
                    label: api_error.label,
                    message: api_error.message,
                })
            } else {
                Err(GateioError::ApiError {
                    label: status.as_u16().to_string(),
                    message: text,
                })
            }
        }
    }

    async fn get_public<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<T> {
        let mut url = format!("{}{}", BASE_URL, endpoint);

        if let Some(params) = params {
            let query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            url.push_str(&format!("?{}", query));
        }

        let response = self
            .client
            .get(&url)
            .header("Gate-Size-Decimal", "1")
            .send()
            .await?;
        self.handle_response(response).await
    }

    async fn get_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Option<Vec<(&str, String)>>,
    ) -> Result<T> {
        let access_key = self
            .access_key
            .as_ref()
            .ok_or_else(|| GateioError::AuthError("API key not set".to_string()))?;
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or_else(|| GateioError::AuthError("Secret key not set".to_string()))?;

        let query = if let Some(params) = params {
            params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&")
        } else {
            String::new()
        };

        let timestamp = Self::get_timestamp();
        let signature = Self::sign_request("GET", endpoint, &query, "", timestamp, secret_key);

        let mut url = format!("{}{}", BASE_URL, endpoint);
        if !query.is_empty() {
            url.push_str(&format!("?{}", query));
        }

        let response = self
            .client
            .get(&url)
            .header("KEY", access_key)
            .header("Timestamp", timestamp.to_string())
            .header("SIGN", signature)
            .header("X-Gate-Size-Decimal", "1")
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn post_signed<T: DeserializeOwned>(&self, endpoint: &str, body: String) -> Result<T> {
        let access_key = self
            .access_key
            .as_ref()
            .ok_or_else(|| GateioError::AuthError("API key not set".to_string()))?;
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or_else(|| GateioError::AuthError("Secret key not set".to_string()))?;

        let timestamp = Self::get_timestamp();
        let body_hash = {
            use sha2::{Digest, Sha512};
            let mut hasher = Sha512::new();
            hasher.update(body.as_bytes());
            hex::encode(hasher.finalize())
        };

        let signature = Self::sign_request("POST", endpoint, "", &body_hash, timestamp, secret_key);

        let url = format!("{}{}", BASE_URL, endpoint);

        let response = self
            .client
            .post(&url)
            .header("KEY", access_key)
            .header("Timestamp", timestamp.to_string())
            .header("SIGN", signature)
            .header("Content-Type", "application/json")
            .header("X-Gate-Size-Decimal", "1")
            .body(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn delete_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Vec<(&str, String)>,
    ) -> Result<T> {
        let access_key = self
            .access_key
            .as_ref()
            .ok_or_else(|| GateioError::AuthError("API key not set".to_string()))?;
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or_else(|| GateioError::AuthError("Secret key not set".to_string()))?;

        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let timestamp = Self::get_timestamp();
        let signature = Self::sign_request("DELETE", endpoint, &query, "", timestamp, secret_key);

        let url = format!("{}{}?{}", BASE_URL, endpoint, query);

        let response = self
            .client
            .delete(&url)
            .header("KEY", access_key)
            .header("Timestamp", timestamp.to_string())
            .header("SIGN", signature)
            .header("X-Gate-Size-Decimal", "1")
            .send()
            .await?;

        self.handle_response(response).await
    }

    // === Public API endpoints ===

    pub async fn get_symbol(&self) -> Result<Symbol> {
        let endpoint = format!("/futures/{}/contracts/{}", self.settle, self.contract);
        let contract_info: ContractInfo = self.get_public(&endpoint, None).await?;

        let tick_size =
            Decimal::from_str(&contract_info.order_price_round).unwrap_or(Decimal::new(1, 1));
        let step_size = Decimal::from_str(&contract_info.quanto_multiplier).unwrap_or(Decimal::ONE);
        let min_notional = Decimal::from(contract_info.order_size_min);

        Ok(Symbol {
            slug: self.contract.clone(),
            tick_size,
            step_size,
            min_notional,
        })
    }

    pub fn get_symbol_sync(&self) -> Result<Symbol> {
        self.runtime.block_on(self.get_symbol())
    }

    pub async fn get_candles(&self, interval: &str, limit: usize) -> Result<Vec<Candle>> {
        let endpoint = format!("/futures/{}/candlesticks", self.settle);
        let limit_str = limit.to_string();
        let params: Vec<(&str, &str)> = vec![
            ("contract", self.contract.as_str()),
            ("interval", interval),
            ("limit", limit_str.as_str()),
        ];

        let data: Vec<CandleData> = self.get_public(&endpoint, Some(&params)).await?;

        let candles: Vec<Candle> = data
            .iter()
            .map(|k| Candle {
                open_time: Timestamp::from_seconds(k.t),
                open: Decimal::from_str(&k.o).unwrap_or_default(),
                high: Decimal::from_str(&k.h).unwrap_or_default(),
                low: Decimal::from_str(&k.l).unwrap_or_default(),
                close: Decimal::from_str(&k.c).unwrap_or_default(),
                volume: Decimal::from(k.v),
            })
            .collect();

        Ok(candles)
    }

    pub fn get_candles_sync(&self, interval: &str, limit: usize) -> Result<Vec<Candle>> {
        self.runtime.block_on(self.get_candles(interval, limit))
    }

    pub async fn get_depth(&self, limit: usize) -> Result<DepthSnapshot> {
        let endpoint = format!("/futures/{}/order_book", self.settle);
        let limit_str = limit.to_string();
        let params: Vec<(&str, &str)> = vec![
            ("contract", self.contract.as_str()),
            ("limit", limit_str.as_str()),
            ("interval", "0"),
            ("with_id", "true"),
        ];

        let result: DepthSnapshot = self.get_public(&endpoint, Some(&params)).await?;

        Ok(result)
    }

    pub async fn get_contract_stats(&self, limit: usize) -> Result<Vec<ContractStats>> {
        let endpoint = format!("/futures/{}/contract_stats", self.settle);
        let limit_str = limit.to_string();
        let params: Vec<(&str, &str)> = vec![
            ("contract", self.contract.as_str()),
            ("interval", "1m"),
            ("limit", limit_str.as_str()),
        ];

        self.get_public(&endpoint, Some(&params)).await
    }

    pub fn get_contract_stats_sync(&self, limit: usize) -> Result<Vec<ContractStats>> {
        self.runtime.block_on(self.get_contract_stats(limit))
    }

    // === Private API endpoints (require authentication) ===
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub label: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct ContractInfo {
    order_price_round: String,
    quanto_multiplier: String,
    order_size_min: i64,
}

#[derive(Deserialize)]
struct CandleData {
    t: u64,
    #[serde(rename = "o")]
    o: String,
    #[serde(rename = "h")]
    h: String,
    #[serde(rename = "l")]
    l: String,
    #[serde(rename = "c")]
    c: String,
    #[serde(rename = "v")]
    v: i64,
}

#[derive(Deserialize, Debug)]
pub struct DepthSnapshot {
    pub id: u64,
    pub bids: Vec<DepthLevel>,
    pub asks: Vec<DepthLevel>,
}

#[derive(Deserialize, Debug)]
pub struct DepthLevel {
    pub p: String, // price
    pub s: i64,    // size
}

#[derive(Deserialize, Debug)]
pub struct ContractStats {
    pub time: u64,
    #[serde(rename = "open_interest")]
    pub open_interest: u64,
}
