use super::auth::{build_signed_query, get_timestamp};
use super::errors::{BinanceError, Result};
use super::types::{
    AccountInfo, ApiError, ExchangeInfo, Filter, Order, OrderRequest, TickerPriceResponse,
};
use crate::models::Symbol;
use reqwest::{Client, Response};
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

const BINANCE_API_BASE: &str = "https://fapi.binance.com";

pub struct BinanceClient {
    client: Client,
    access_key: Option<String>,
    secret_key: Option<String>,
    base_url: String,
}

impl BinanceClient {
    pub fn new(access_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            access_key,
            secret_key,
            base_url: BINANCE_API_BASE.to_string(),
        }
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&text).map_err(|e| {
                BinanceError::ParseError(format!("Failed to parse response: {}. Body: {}", e, text))
            })
        } else {
            if let Ok(api_error) = serde_json::from_str::<ApiError>(&text) {
                Err(BinanceError::ApiError {
                    code: api_error.code,
                    msg: api_error.msg,
                })
            } else {
                Err(BinanceError::ApiError {
                    code: status.as_u16() as i32,
                    msg: text,
                })
            }
        }
    }

    async fn get_public<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<T> {
        let mut url = format!("{}{}", self.base_url, endpoint);

        if let Some(params) = params {
            let query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            url.push_str(&format!("?{}", query));
        }

        let response = self.client.get(&url).send().await?;
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
            .ok_or_else(|| BinanceError::AuthError("API key not set".to_string()))?;
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or_else(|| BinanceError::AuthError("Secret key not set".to_string()))?;

        let timestamp = get_timestamp().to_string();
        let mut all_params = params.unwrap_or_default();
        all_params.push(("timestamp", timestamp));

        let params_ref: Vec<(&str, &str)> =
            all_params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let query = build_signed_query(&params_ref, secret_key);
        let url = format!("{}{}?{}", self.base_url, endpoint, query);

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", access_key)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn post_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Vec<(&str, String)>,
    ) -> Result<T> {
        let access_key = self
            .access_key
            .as_ref()
            .ok_or_else(|| BinanceError::AuthError("API key not set".to_string()))?;
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or_else(|| BinanceError::AuthError("Secret key not set".to_string()))?;

        let timestamp = get_timestamp().to_string();
        let mut all_params = params;
        all_params.push(("timestamp", timestamp));

        let params_ref: Vec<(&str, &str)> =
            all_params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let query = build_signed_query(&params_ref, secret_key);
        let url = format!("{}{}?{}", self.base_url, endpoint, query);

        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", access_key)
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
            .ok_or_else(|| BinanceError::AuthError("API key not set".to_string()))?;
        let secret_key = self
            .secret_key
            .as_ref()
            .ok_or_else(|| BinanceError::AuthError("Secret key not set".to_string()))?;

        let timestamp = get_timestamp().to_string();
        let mut all_params = params;
        all_params.push(("timestamp", timestamp));

        let params_ref: Vec<(&str, &str)> =
            all_params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let query = build_signed_query(&params_ref, secret_key);
        let url = format!("{}{}?{}", self.base_url, endpoint, query);

        let response = self
            .client
            .delete(&url)
            .header("X-MBX-APIKEY", access_key)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // === Public API endpoints ===

    pub async fn ping(&self) -> Result<HashMap<String, String>> {
        self.get_public("/fapi/v1/ping", None).await
    }

    pub async fn get_ticker_price(&self, symbol: &str) -> Result<Decimal> {
        let response: TickerPriceResponse = self
            .get_public(&format!("/fapi/v2/ticker/price?symbol={}", symbol), None)
            .await?;
        Ok(response.price)
    }

    pub async fn get_symbol(&self, symbol: &str) -> Result<Symbol> {
        let exchange_info: ExchangeInfo = self.get_public("/fapi/v1/exchangeInfo", None).await?;
        for sym in exchange_info.symbols {
            if sym.symbol.eq_ignore_ascii_case(symbol) {
                let mut t_s = Decimal::ZERO;
                let mut s_s = Decimal::ZERO;
                let mut n = Decimal::ZERO;
                for filter in sym.filters {
                    match filter {
                        Filter::PriceFilter { tick_size } => {
                            t_s = tick_size.parse().unwrap();
                        }
                        Filter::MinNotional { notional } => {
                            n = notional.parse().unwrap();
                        }
                        Filter::LotSize { step_size } => {
                            s_s = step_size.parse().unwrap();
                        }
                        _ => {}
                    }
                }
                return Ok(Symbol {
                    tick_size: t_s,
                    step_size: s_s,
                    notional: n,
                    slug: sym.symbol,
                });
            }
        }
        Err(BinanceError::ParseError("Symbol not found".to_string()))
    }

    // === Private API endpoints (require authentication) ===

    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        self.get_signed("/fapi/v3/account", None).await
    }

    pub async fn place_order(&self, order: OrderRequest) -> Result<Order> {
        let mut params = vec![
            ("symbol", order.symbol.clone()),
            ("side", format!("{:?}", order.side).to_uppercase()),
            ("type", format!("{:?}", order.order_type).to_uppercase()),
        ];

        if let Some(qty) = order.quantity {
            params.push(("quantity", qty));
        }

        if let Some(price) = order.price {
            params.push(("price", price));
        }

        if let Some(tif) = order.time_in_force {
            params.push(("timeInForce", format!("{:?}", tif).to_uppercase()));
        }

        if let Some(nort) = order.new_order_resp_type {
            params.push(("newOrderRespType", nort));
        }

        self.post_signed("/fapi/v1/order", params).await
    }

    pub async fn cancel_order(&self, symbol: &str, order_id: u64) -> Result<Order> {
        let params = vec![
            ("symbol", symbol.to_string()),
            ("orderId", order_id.to_string()),
        ];
        self.delete_signed("/fapi/v1/order", params).await
    }

    pub async fn cancel_all_orders(&self, symbol: &str) -> Result<Vec<Order>> {
        let params = vec![("symbol", symbol.to_string())];
        self.delete_signed("/fapi/v1/allOpenOrders", params).await
    }
}

impl Default for BinanceClient {
    fn default() -> Self {
        Self::new(None, None)
    }
}
