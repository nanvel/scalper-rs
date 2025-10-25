use crate::binance::BinanceClient;
use crate::binance::types::{Order, OrderRequest, OrderSide, OrderType};
use crate::models::Symbol;
use rust_decimal::Decimal;

pub struct Trader {
    orders: Vec<Order>,
}

impl Trader {
    pub fn new() -> Self {
        Self { orders: Vec::new() }
    }

    pub async fn buy(&mut self, client: &BinanceClient, symbol: &Symbol, quantity: Decimal) {
        let order = client
            .place_order(OrderRequest {
                symbol: symbol.slug.clone(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                quantity: Some(quantity.to_string()),
                price: None,
                time_in_force: None,
            })
            .await
            .unwrap();

        self.orders.push(order);
    }

    pub async fn sell(&mut self, client: &BinanceClient, symbol: &Symbol, quantity: Decimal) {
        let order = client
            .place_order(OrderRequest {
                symbol: symbol.slug.clone(),
                side: OrderSide::Sell,
                order_type: OrderType::Market,
                quantity: Some(quantity.to_string()),
                price: None,
                time_in_force: None,
            })
            .await
            .unwrap();

        self.orders.push(order);
    }

    pub async fn flat(&mut self, client: &BinanceClient, symbol: &Symbol) {
        let mut total_quantity = Decimal::ZERO;
        for order in &self.orders {
            let executed_qty = order.executed_qty;
            match order.side {
                OrderSide::Buy => total_quantity += executed_qty,
                OrderSide::Sell => total_quantity -= executed_qty,
            }
        }

        if total_quantity > Decimal::ZERO {
            // Need to sell to flat
            self.sell(client, symbol, total_quantity).await;
        } else if total_quantity < Decimal::ZERO {
            // Need to buy to flat
            self.buy(client, symbol, -total_quantity).await;
        }
    }

    pub fn pnl(&self) -> Decimal {
        let mut pnl = Decimal::ZERO;
        for order in &self.orders {
            let executed_qty = order.executed_qty;
            let price = order.avg_price;
            match order.side {
                OrderSide::Buy => pnl -= price * executed_qty,
                OrderSide::Sell => pnl += price * executed_qty,
            }
        }
        pnl
    }
}
