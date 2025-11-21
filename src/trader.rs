use crate::exchanges::Exchange;
use crate::models::{Interval, Lot, NewOrder, Order, OrderSide, OrderType, Orders, Symbol};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

pub struct Trader<'a> {
    exchange: &'a Box<dyn Exchange>,
    symbol: Symbol,
    orders: Orders,
    lot: Lot,
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
}

impl<'a> Trader<'a> {
    pub fn new(exchange: &'a Box<dyn Exchange>, symbol: Symbol, orders: Orders, lot: Lot) -> Self {
        Trader {
            exchange,
            symbol,
            orders,
            lot,
            bid: None,
            ask: None,
        }
    }

    pub fn place_market_buy(&mut self) {
        if let Some(ask) = self.ask {
            let size = self.lot.get_value(ask, &self.symbol);
            self.exchange.place_order(NewOrder {
                order_type: OrderType::Market,
                order_side: OrderSide::Buy,
                quantity: size,
                price: None,
            });
        }
    }

    pub fn place_market_sell(&mut self) {
        if let Some(bid) = self.bid {
            let size = self.lot.get_value(bid, &self.symbol);
            self.exchange.place_order(NewOrder {
                order_type: OrderType::Market,
                order_side: OrderSide::Sell,
                quantity: size,
                price: None,
            });
        }
    }

    pub fn place_limit(&mut self, price: Decimal) {
        if let Some(bid) = self.bid {
            let size = self.lot.get_value(price, &self.symbol);
            self.exchange.place_order(NewOrder {
                order_type: OrderType::Limit,
                order_side: if price < bid {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                },
                quantity: size,
                price: Some(price),
            });
        }
    }

    pub fn place_stop(&mut self, price: Decimal) {
        if let Some(bid) = self.bid {
            let size = self.lot.get_value(price, &self.symbol);
            self.exchange.place_order(NewOrder {
                order_type: OrderType::Stop,
                order_side: if price < bid {
                    OrderSide::Sell
                } else {
                    OrderSide::Buy
                },
                quantity: size,
                price: Some(price),
            });
        }
    }

    pub fn flat(&self) {
        let balance = self.orders.base_balance();
        if balance != Decimal::ZERO {
            if balance > Decimal::ZERO {
                self.exchange.place_order(NewOrder {
                    order_type: OrderType::Market,
                    order_side: OrderSide::Sell,
                    quantity: balance,
                    price: None,
                });
            } else {
                self.exchange.place_order(NewOrder {
                    order_type: OrderType::Market,
                    order_side: OrderSide::Buy,
                    quantity: -balance,
                    price: None,
                });
            }
        }
    }

    pub fn reverse(&self) {
        let balance = self.orders.base_balance();
        if balance != Decimal::ZERO {
            if balance > Decimal::ZERO {
                self.exchange.place_order(NewOrder {
                    order_type: OrderType::Market,
                    order_side: OrderSide::Sell,
                    quantity: balance * Decimal::from(2),
                    price: None,
                });
            } else {
                self.exchange.place_order(NewOrder {
                    order_type: OrderType::Market,
                    order_side: OrderSide::Buy,
                    quantity: -balance * Decimal::from(2),
                    price: None,
                });
            }
        }
    }

    pub fn cancel_all(&self) {
        for o in self.orders.open() {
            self.exchange.cancel_order(o.id.clone());
        }
    }

    pub fn consume_order(&mut self, order: Order) {
        self.orders.consume(order);
    }

    pub fn get_pnl(&self) -> Decimal {
        self.orders.pnl(self.bid, self.ask)
    }

    pub fn get_commission(&self) -> Decimal {
        self.orders.commission()
    }

    pub fn get_open_orders(&self) -> Vec<&Order> {
        self.orders.open()
    }

    pub fn get_last_closed_order(&self) -> Option<&Order> {
        self.orders.last_closed()
    }

    pub fn get_size(&self) -> Decimal {
        self.lot.get_size()
    }

    pub fn select_size(&mut self, index: usize) {
        self.lot.select_size(index);
    }

    pub fn get_lots(&self) -> f64 {
        if let Some(size_base) = self.lot.get_size_base() {
            (self.orders.base_balance() / size_base)
                .round()
                .to_f64()
                .unwrap()
        } else {
            0_f64
        }
    }

    pub fn get_multiplier(&self) -> usize {
        self.lot.get_multiplier()
    }

    pub fn update_bid_ask(&mut self, bid: Option<Decimal>, ask: Option<Decimal>) {
        if bid.is_some() {
            self.bid = bid;
        }
        if ask.is_some() {
            self.ask = ask;
        }
    }
}
