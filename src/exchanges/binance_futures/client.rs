use super::auth::{build_signed_query, get_timestamp};
use super::errors::{BinanceError, Result};
use crate::models::{NewOrder, Order, OrderSide, OrderStatus, OrderType, Symbol, Timestamp};
use reqwest::blocking::{Client, Response};
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub struct BinanceClient {
    client: Client,
    symbol: String,
    access_key: Option<String>,
    secret_key: Option<String>,
    base_url: &'static str,
}

impl BinanceClient {
    pub fn new(symbol: String, access_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            symbol,
            access_key,
            secret_key,
            base_url: "https://fapi.binance.com",
        }
    }

    fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        let text = response.text()?;

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

    fn get_public<T: DeserializeOwned>(
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

        let response = self.client.get(&url).send()?;
        self.handle_response(response)?
    }

    fn get_signed<T: DeserializeOwned>(
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
            .send()?;

        self.handle_response(response)?
    }

    fn post_signed<T: DeserializeOwned>(
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
            .send()?;

        self.handle_response(response)
    }

    fn delete_signed<T: DeserializeOwned>(
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
            .send()?;

        self.handle_response(response)
    }

    // === Public API endpoints ===

    pub fn get_symbol(&self) -> Result<Symbol> {
        let exchange_info: ExchangeInfo = self.get_public("/fapi/v1/exchangeInfo", None)?;
        for sym in exchange_info.symbols {
            if sym.symbol.eq_ignore_ascii_case(&self.symbol) {
                let mut t_s = Decimal::ZERO;
                let mut s_s = Decimal::ZERO;
                let mut m_n = Decimal::ZERO;
                for filter in sym.filters {
                    match filter {
                        Filter::PriceFilter { tick_size } => {
                            t_s = tick_size.parse().unwrap();
                        }
                        Filter::MinNotional { notional } => {
                            m_n = notional.parse().unwrap();
                        }
                        Filter::LotSize { step_size } => {
                            s_s = step_size.parse().unwrap();
                        }
                        _ => {}
                    }
                }
                return Ok(Symbol {
                    slug: self.symbol.clone(),
                    tick_size: t_s,
                    step_size: s_s,
                    min_notional: m_n,
                });
            }
        }
        Err(BinanceError::ParseError("Symbol not found".to_string()))
    }

    // === Private API endpoints (require authentication) ===

    pub fn place_order(&self, order: NewOrder) -> Result<Order> {
        let order_side = match order.order_side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };
        let order_type = match order.order_type {
            OrderType::Market => "MARKET",
            OrderType::Limit => "LIMIT",
            OrderType::Stop => "STOP_MARKET",
        };

        let mut params = vec![
            ("symbol", self.symbol.clone()),
            ("side", order_side.to_string()),
            ("type", order_type.to_string()),
            ("quantity", order.quantity.to_string()),
            ("newOrderRespType", "RESULT".to_string()),
        ];

        if let Some(price) = order.price {
            params.push(("price", price.to_string()));
        };

        let resp: BinanceOrder = self.post_signed("/fapi/v1/order", params)?;

        let order_status = match resp.status.as_str() {
            "NEW" => OrderStatus::Pending,
            "PARTIALLY_FILLED" => OrderStatus::Pending,
            _ => OrderStatus::Filled,
        };

        Ok(Order {
            id: resp.order_id.to_string(),
            order_type: order.order_type,
            order_side: order.order_side,
            order_status,
            quantity: resp.orig_qty,
            executed_quantity: resp.executed_qty,
            price: resp.price,
            average_price: resp.avg_price,
            commission: resp.commission(),
            timestamp: Timestamp(resp.update_time / 1000),
        })
    }

    pub fn cancel_order(&self, order_id: &str) -> Result<Order> {
        let params = vec![
            ("symbol", order_id.to_string()),
            ("orderId", order_id.to_string()),
        ];
        let resp: BinanceOrder = self.delete_signed("/fapi/v1/order", params)?;

        let order_type = match resp.order_type.as_str() {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit,
            "STOP_MARKET" => OrderType::Stop,
            _ => return Err(BinanceError::ParseError("Unknown order type".to_string())),
        };

        let order_side = match resp.order_side.as_str() {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => return Err(BinanceError::ParseError("Unknown order side".to_string())),
        };

        Ok(Order {
            id: resp.order_id.to_string(),
            order_type: order_type,
            order_side: order_side,
            order_status: OrderStatus::Filled,
            quantity: resp.orig_qty,
            executed_quantity: resp.executed_qty,
            price: resp.price,
            average_price: resp.avg_price,
            commission: resp.commission(),
            timestamp: Timestamp(resp.update_time / 1000),
        })
    }

    pub fn get_listen_key(&self) -> Result<String> {
        let response: ListenKeyResponse = self.post_signed("/fapi/v1/listenKey", vec![])?;
        Ok(response.listen_key)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub msg: String,
}

#[derive(Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<crate::binance::types::SymbolInfo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    pub symbol: String,
    pub filters: Vec<Filter>,
}

#[derive(Deserialize)]
#[serde(tag = "filterType")]
pub enum Filter {
    #[serde(rename = "PRICE_FILTER")]
    PriceFilter {
        #[serde(rename = "tickSize")]
        tick_size: String,
    },
    #[serde(rename = "MIN_NOTIONAL")]
    MinNotional {
        #[serde(rename = "notional")]
        notional: String,
    },
    #[serde(rename = "LOT_SIZE")]
    LotSize {
        #[serde(rename = "stepSize")]
        step_size: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
pub struct BinanceOrder {
    #[serde(rename = "orderId")]
    pub order_id: u64,
    pub price: Decimal,
    #[serde(rename = "origQty")]
    pub orig_qty: Decimal,
    #[serde(rename = "executedQty")]
    pub executed_qty: Decimal,
    pub status: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "side")]
    pub order_side: String,
    #[serde(rename = "avgPrice")]
    pub avg_price: Decimal,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
}

impl BinanceOrder {
    pub fn commission(&self) -> Decimal {
        let rate = match self.order_type.as_str() {
            "LIMIT" => Decimal::from_str("0.0002").unwrap(),
            _ => Decimal::from_str("0.0005").unwrap(),
        };
        self.executed_qty * self.price * rate
    }
}

#[derive(serde::Deserialize)]
pub struct ListenKeyResponse {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}
