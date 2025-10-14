use crate::models::Symbol;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize)]
struct ExchangeInfo {
    symbols: Vec<SymbolInfo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolInfo {
    symbol: String,
    filters: Vec<Filter>,
}

#[derive(Deserialize)]
#[serde(tag = "filterType")]
enum Filter {
    #[serde(rename = "PRICE_FILTER")]
    PriceFilter {
        #[serde(rename = "tickSize")]
        tick_size: String,
    },
    #[serde(other)]
    Other,
}

pub async fn load_symbol(symbol: &str) -> Result<Symbol, Box<dyn std::error::Error>> {
    let url = "https://fapi.binance.com/fapi/v1/exchangeInfo";
    let response: ExchangeInfo = reqwest::get(url).await?.json().await?;

    for sym in response.symbols {
        if sym.symbol.eq_ignore_ascii_case(symbol) {
            for filter in sym.filters {
                if let Filter::PriceFilter { tick_size } = filter {
                    return Ok(Symbol {
                        tick_size: tick_size.parse()?,
                        slug: sym.symbol,
                    });
                }
            }
        }
    }

    Err("Symbol not found".into())
}
