use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountInfo {
    #[serde(rename = "makerCommission")]
    pub maker_commission: u32,
    #[serde(rename = "takerCommission")]
    pub taker_commission: u32,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    #[serde(rename = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(rename = "canDeposit")]
    pub can_deposit: bool,
    pub balances: Vec<Balance>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Order {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: Decimal,
    pub status: String,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    #[serde(rename = "avgPrice")]
    pub avg_price: Decimal,
}

impl Order {
    pub fn is_open(&self) -> bool {
        self.status == "NEW" || self.status == "PARTIALLY_FILLED"
    }

    pub fn is_closed(&self) -> bool {
        if self.is_open() {
            return false;
        };

        !self.executed_qty.is_zero()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    StopLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    Gtc, // Good Till Cancel
    Ioc, // Immediate or Cancel
    Fok, // Fill or Kill
}

impl From<&str> for TimeInForce {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GTC" => TimeInForce::Gtc,
            "IOC" => TimeInForce::Ioc,
            "FOK" => TimeInForce::Fok,
            _ => panic!("Invalid TimeInForce value"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub msg: String,
}

#[derive(serde::Deserialize)]
pub struct TickerPriceResponse {
    pub price: Decimal,
}

#[derive(serde::Deserialize)]
pub struct ListenKeyResponse {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}
