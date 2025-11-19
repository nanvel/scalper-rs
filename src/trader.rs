use crate::exchanges::Exchange;
use crate::models::{BidAsk, Log, Lot, NewOrder, Order, OrderSide, OrderType, Orders, Symbol};
use rust_decimal::Decimal;
use std::sync::mpsc::Sender;

pub struct Trader<'a> {
    exchange: &'a dyn Exchange,
    symbol: Symbol,
    orders: Orders,
    lot: Lot,
    bid_ask: &'a BidAsk,
    logs_sender: Sender<Log>,
}

impl<'a> Trader<'a> {
    pub fn new(
        exchange: &'a dyn Exchange,
        symbol: Symbol,
        orders: Orders,
        lot: Lot,
        bid_ask: &'a BidAsk,
        logs_sender: Sender<Log>,
    ) -> Self {
        Trader {
            exchange,
            symbol,
            orders,
            lot,
            bid_ask,
            logs_sender,
        }
    }

    pub fn buy_market(&mut self) {
        if let Some(ask) = self.bid_ask.ask {
            let size = self.lot.get_value(ask, &self.symbol);
            self.exchange.place_order(NewOrder {
                order_type: OrderType::Market,
                order_side: OrderSide::Buy,
                quantity: size,
                price: None,
            });
        }
    }

    pub fn sell_market(&mut self) {
        if let Some(bid) = self.bid_ask.bid {
            let size = self.lot.get_value(bid, &self.symbol);
            self.exchange.place_order(NewOrder {
                order_type: OrderType::Market,
                order_side: OrderSide::Sell,
                quantity: size,
                price: None,
            });
        }
    }

    pub fn buy_limit(&mut self, price: Decimal) {
        let size = self.lot.get_value(price, &self.symbol);
        self.exchange.place_order(NewOrder {
            order_type: OrderType::Limit,
            order_side: OrderSide::Buy,
            quantity: size,
            price: Some(price),
        });
    }

    pub fn sell_limit(&mut self, price: Decimal) {
        let size = self.lot.get_value(price, &self.symbol);
        self.exchange.place_order(NewOrder {
            order_type: OrderType::Limit,
            order_side: OrderSide::Sell,
            quantity: size,
            price: Some(price),
        });
    }

    pub fn buy_stop(&mut self, price: Decimal) {
        let size = self.lot.get_value(price, &self.symbol);
        self.exchange.place_order(NewOrder {
            order_type: OrderType::Stop,
            order_side: OrderSide::Buy,
            quantity: size,
            price: Some(price),
        });
    }

    pub fn sell_stop(&mut self, price: Decimal) {
        let size = self.lot.get_value(price, &self.symbol);
        self.exchange.place_order(NewOrder {
            order_type: OrderType::Stop,
            order_side: OrderSide::Sell,
            quantity: size,
            price: Some(price),
        });
    }

    pub fn consume_order(&mut self, order: Order) {
        self.orders.consume(order);
    }
}
