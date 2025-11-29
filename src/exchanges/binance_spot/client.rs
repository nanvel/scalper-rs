use crate::exchanges::base::USER_AGENT;
use crate::exchanges::binance_base::auth::{build_signed_query, get_timestamp};
use crate::exchanges::binance_base::errors::{BinanceError, Result};
use crate::models::{
    Candle, NewOrder, Order, OrderSide, OrderStatus, OrderType, Symbol, Timestamp,
};
use reqwest::{Client, Response};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use tokio::runtime::Runtime;

const BASE_URL: &str = "https://api.binance.com";

pub struct BinanceClient {
    client: Client,
    symbol: String,
    access_key: Option<String>,
    secret_key: Option<String>,
    runtime: Runtime,
}

// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-endpoints#exchange-information
impl BinanceClient {
    pub fn new(symbol: String, access_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
            symbol,
            access_key,
            secret_key,
            runtime: Runtime::new().expect("Failed to create BinanceClient Tokio runtime"),
        }
    }

    pub fn has_auth(&self) -> bool {
        self.access_key.is_some() && self.secret_key.is_some()
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
        let mut url = format!("{}{}", BASE_URL, endpoint);

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
        let url = format!("{}{}?{}", BASE_URL, endpoint, query);

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
        let url = format!("{}{}?{}", BASE_URL, endpoint, query);

        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", access_key)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn put_signed<T: DeserializeOwned>(
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
        let url = format!("{}{}?{}", BASE_URL, endpoint, query);

        let response = self
            .client
            .put(&url)
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
        let url = format!("{}{}?{}", BASE_URL, endpoint, query);

        let response = self
            .client
            .delete(&url)
            .header("X-MBX-APIKEY", access_key)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn post_with_api_key<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let access_key = self
            .access_key
            .as_ref()
            .ok_or_else(|| BinanceError::AuthError("API key not set".to_string()))?;

        let url = format!("{}{}", BASE_URL, endpoint);

        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", access_key)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn put_with_api_key<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let access_key = self
            .access_key
            .as_ref()
            .ok_or_else(|| BinanceError::AuthError("API key not set".to_string()))?;

        let url = format!("{}{}", BASE_URL, endpoint);

        let response = self
            .client
            .put(&url)
            .header("X-MBX-APIKEY", access_key)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // === Public API endpoints ===

    pub async fn get_symbol(&self) -> Result<Symbol> {
        let params: Vec<(&str, &str)> = vec![("symbol", self.symbol.as_str())];
        let exchange_info: ExchangeInfo = self
            .get_public("/api/v3/exchangeInfo", Some(&params))
            .await?;
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

    pub fn get_symbol_sync(&self) -> Result<Symbol> {
        self.runtime.block_on(self.get_symbol())
    }

    pub async fn get_candles(&self, interval: &str, limit: usize) -> Result<Vec<Candle>> {
        let limit_str = limit.to_string();
        let params: Vec<(&str, &str)> = vec![
            ("symbol", self.symbol.as_str()),
            ("interval", interval),
            ("limit", limit_str.as_str()),
        ];
        let data: Vec<serde_json::Value> = self.get_public("/api/v3/klines", Some(&params)).await?;

        let candles: Vec<Candle> = data
            .iter()
            .map(|k| Candle {
                open_time: Timestamp::from_milliseconds(k[0].as_u64().unwrap()),
                open: Decimal::from_str(k[1].as_str().unwrap()).unwrap(),
                high: Decimal::from_str(k[2].as_str().unwrap()).unwrap(),
                low: Decimal::from_str(k[3].as_str().unwrap()).unwrap(),
                close: Decimal::from_str(k[4].as_str().unwrap()).unwrap(),
                volume: Decimal::from_str(k[5].as_str().unwrap()).unwrap(),
            })
            .collect();

        Ok(candles)
    }

    pub fn get_candles_sync(&self, interval: &str, limit: usize) -> Result<Vec<Candle>> {
        self.runtime.block_on(self.get_candles(interval, limit))
    }

    pub async fn get_depth(&self, limit: usize) -> Result<DepthSnapshot> {
        let limit_str = limit.to_string();
        let params: Vec<(&str, &str)> = vec![
            ("symbol", self.symbol.as_str()),
            ("limit", limit_str.as_str()),
        ];

        let result: DepthSnapshot = self.get_public("/api/v3/depth", Some(&params)).await?;

        Ok(result)
    }

    // === Private API endpoints (require authentication) ===

    pub async fn place_order(&self, order: NewOrder) -> Result<Order> {
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

        if order.order_type == OrderType::Limit {
            params.push(("price", order.price.unwrap().to_string()));
        } else if order.order_type == OrderType::Stop {
            params.push(("stopPrice", order.price.unwrap().to_string()));
        }

        if matches!(order.order_type, OrderType::Limit) {
            params.push(("timeInForce", "GTC".to_string()));
        };

        let resp: BinanceOrder = self.post_signed("/api/v3/order", params).await?;

        let order_status = match resp.status.as_str() {
            "NEW" => OrderStatus::Pending,
            "PARTIALLY_FILLED" => OrderStatus::Pending,
            _ => OrderStatus::Filled,
        };

        Ok(Order {
            id: resp.order_id.to_string(),
            order_type: order.order_type.clone(),
            order_side: order.order_side,
            order_status,
            quantity: resp.orig_qty,
            executed_quantity: resp.executed_qty,
            price: match &order.order_type {
                OrderType::Stop => resp.stop_price.unwrap_or(resp.price),
                _ => resp.price,
            },
            average_price: resp.get_avg_price(),
            commission: resp.commission(),
            timestamp: Timestamp::from_milliseconds(resp.get_timestamp()),
            is_update: false,
        })
    }

    pub fn place_order_sync(&self, order: NewOrder) -> Result<Order> {
        self.runtime.block_on(self.place_order(order))
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<Order> {
        let params = vec![
            ("symbol", self.symbol.clone()),
            ("orderId", order_id.to_string()),
        ];
        let resp: BinanceOrder = self.delete_signed("/api/v3/order", params).await?;

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
            order_type: order_type.clone(),
            order_side: order_side,
            order_status: OrderStatus::Filled,
            quantity: resp.orig_qty,
            executed_quantity: resp.executed_qty,
            price: match order_type {
                OrderType::Stop => resp.stop_price.unwrap_or(resp.price),
                _ => resp.price,
            },
            average_price: resp.get_avg_price(),
            commission: resp.commission(),
            timestamp: Timestamp::from_milliseconds(resp.get_timestamp()),
            is_update: false,
        })
    }

    pub fn cancel_order_sync(&self, order_id: &str) -> Result<Order> {
        self.runtime.block_on(self.cancel_order(order_id))
    }

    pub async fn create_listen_key(&self) -> Result<String> {
        let listen_key_resp: ListenKey = self.post_with_api_key("/api/v3/userDataStream").await?;
        Ok(listen_key_resp.listen_key)
    }

    pub async fn refresh_listen_key(&self) -> Result<()> {
        self.put_with_api_key::<ListenKey>("/api/v3/userDataStream")
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub msg: String,
}

#[derive(Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<SymbolInfo>,
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
    #[serde(rename = "stopPrice", default)]
    pub stop_price: Option<Decimal>,
    #[serde(rename = "origQty")]
    pub orig_qty: Decimal,
    #[serde(rename = "executedQty")]
    pub executed_qty: Decimal,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: Decimal,
    pub status: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "side")]
    pub order_side: String,
    #[serde(rename = "avgPrice", default)]
    pub avg_price: Option<Decimal>,
    #[serde(rename = "transactTime", default)]
    pub transact_time: Option<u64>,
    #[serde(rename = "updateTime", default)]
    pub update_time: Option<u64>,
}

impl BinanceOrder {
    pub fn get_avg_price(&self) -> Decimal {
        self.avg_price.unwrap_or_else(|| {
            if self.executed_qty > Decimal::ZERO {
                self.cumulative_quote_qty / self.executed_qty
            } else {
                Decimal::ZERO
            }
        })
    }

    pub fn get_timestamp(&self) -> u64 {
        self.update_time.or(self.transact_time).unwrap_or(0)
    }

    pub fn commission(&self) -> Decimal {
        let rate = match self.order_type.as_str() {
            "LIMIT" => Decimal::from_str("0.0002").unwrap(),
            _ => Decimal::from_str("0.0005").unwrap(),
        };
        self.executed_qty * self.get_avg_price() * rate
    }
}

#[derive(Deserialize)]
struct ListenKey {
    #[serde(rename = "listenKey")]
    listen_key: String,
}

#[derive(Deserialize)]
pub struct DepthSnapshot {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}
