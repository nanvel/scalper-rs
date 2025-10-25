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
                new_order_resp_type: Some("RESULT".into()),
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
                new_order_resp_type: Some("RESULT".to_string()),
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
            self.sell(client, symbol, total_quantity).await;
        } else if total_quantity < Decimal::ZERO {
            self.buy(client, symbol, -total_quantity).await;
        }
    }

    pub fn base_balance(&self) -> Decimal {
        let mut base_balance = Decimal::ZERO;
        for order in &self.orders {
            let executed_qty = order.executed_qty;
            match order.side {
                OrderSide::Buy => {
                    base_balance += executed_qty;
                }
                OrderSide::Sell => {
                    base_balance -= executed_qty;
                }
            }
        }
        base_balance
    }

    pub fn pnl(&self, bid: Option<Decimal>, ask: Option<Decimal>) -> Decimal {
        if !bid.is_some() && !ask.is_some() {
            return Decimal::ZERO;
        }

        let mut spent = Decimal::ZERO;
        let mut received = Decimal::ZERO;
        let mut base_balance = Decimal::ZERO;
        for order in &self.orders {
            let executed_qty = order.executed_qty;
            let price = order.avg_price;
            match order.side {
                OrderSide::Buy => {
                    spent += price * executed_qty;
                    base_balance += executed_qty;
                }
                OrderSide::Sell => {
                    received += price * executed_qty;
                    base_balance -= executed_qty;
                }
            }
        }
        if base_balance > Decimal::ZERO {
            received += bid.unwrap() * base_balance;
        } else if base_balance < Decimal::ZERO {
            spent += ask.unwrap() * -base_balance;
        }

        received - spent
    }
}
