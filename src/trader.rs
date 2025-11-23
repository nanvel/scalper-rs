use crate::models::{NewOrder, Order, OrderSide, OrderType, Orders, Symbol};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

pub struct Trader {
    symbol: Symbol,
    orders: Orders,
    size_multiplier_options: [usize; 4],
    size_multiplier_index: usize,
    pub size_quote: Decimal,
    size_base: Option<Decimal>,
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
    sl_pnl: Option<Decimal>,
}

impl Trader {
    pub fn new(
        symbol: Symbol,
        orders: Orders,
        size_multiplier_options: [usize; 4],
        size_quote: Decimal,
        sl_pnl: Option<Decimal>,
    ) -> Self {
        Trader {
            symbol,
            orders,
            size_multiplier_options,
            size_multiplier_index: 0,
            size_quote,
            size_base: None,
            bid: None,
            ask: None,
            sl_pnl,
        }
    }

    fn get_single_size(&mut self) -> Option<Decimal> {
        if let Some(size_base) = self.size_base {
            return Some(size_base);
        }

        if let Some(bid) = self.bid {
            let single_size = self.symbol.tune_quantity(self.size_quote / bid, bid);
            self.size_base = Some(single_size);
            Some(single_size)
        } else {
            None
        }
    }

    fn get_work_size(&mut self) -> Option<Decimal> {
        if let Some(size) = self.get_single_size() {
            Some(size * Decimal::from(self.get_size_multiplier()))
        } else {
            None
        }
    }

    pub fn market_buy(&mut self) -> Option<NewOrder> {
        if let Some(size) = self.get_work_size() {
            Some(NewOrder {
                order_type: OrderType::Market,
                order_side: OrderSide::Buy,
                quantity: size,
                price: None,
            })
        } else {
            None
        }
    }

    pub fn market_sell(&mut self) -> Option<NewOrder> {
        if let Some(size) = self.get_work_size() {
            Some(NewOrder {
                order_type: OrderType::Market,
                order_side: OrderSide::Sell,
                quantity: size,
                price: None,
            })
        } else {
            None
        }
    }

    pub fn limit(&mut self, price: Decimal) -> Option<NewOrder> {
        if let Some(bid) = self.bid {
            if let Some(size) = self.get_work_size() {
                Some(NewOrder {
                    order_type: OrderType::Limit,
                    order_side: if price < bid {
                        OrderSide::Buy
                    } else {
                        OrderSide::Sell
                    },
                    quantity: size,
                    price: Some(price),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn stop(&mut self, price: Decimal) -> Option<NewOrder> {
        if let Some(bid) = self.bid {
            if let Some(size) = self.get_work_size() {
                Some(NewOrder {
                    order_type: OrderType::Stop,
                    order_side: if price < bid {
                        OrderSide::Sell
                    } else {
                        OrderSide::Buy
                    },
                    quantity: size,
                    price: Some(price),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn flat(&self) -> Option<NewOrder> {
        let balance = self.orders.base_balance();
        if balance != Decimal::ZERO {
            if balance > Decimal::ZERO {
                Some(NewOrder {
                    order_type: OrderType::Market,
                    order_side: OrderSide::Sell,
                    quantity: balance,
                    price: None,
                })
            } else {
                Some(NewOrder {
                    order_type: OrderType::Market,
                    order_side: OrderSide::Buy,
                    quantity: -balance,
                    price: None,
                })
            }
        } else {
            None
        }
    }

    pub fn reverse(&self) -> Option<NewOrder> {
        let balance = self.orders.base_balance();
        if balance != Decimal::ZERO {
            if balance > Decimal::ZERO {
                Some(NewOrder {
                    order_type: OrderType::Market,
                    order_side: OrderSide::Sell,
                    quantity: balance * Decimal::from(2),
                    price: None,
                })
            } else {
                Some(NewOrder {
                    order_type: OrderType::Market,
                    order_side: OrderSide::Buy,
                    quantity: -balance * Decimal::from(2),
                    price: None,
                })
            }
        } else {
            None
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

    pub fn set_size_multiplier_index(&mut self, index: usize) {
        self.size_multiplier_index = index;
    }

    pub fn get_lots(&self) -> f64 {
        if let Some(single_size) = self.size_base {
            (self.orders.base_balance() / single_size)
                .round()
                .to_f64()
                .unwrap()
        } else {
            0_f64
        }
    }

    pub fn get_size_multiplier(&self) -> usize {
        self.size_multiplier_options[self.size_multiplier_index]
    }

    pub fn set_bid_ask(&mut self, bid: Option<Decimal>, ask: Option<Decimal>) {
        if bid.is_some() {
            self.bid = bid;
        }
        if ask.is_some() {
            self.ask = ask;
        }
    }

    pub fn get_sl_price(&self) -> Option<Decimal> {
        let balance = self.orders.base_balance();
        if balance == Decimal::ZERO {
            return None;
        }
        if let Some(sl_pnl) = self.sl_pnl {
            if let Some(entry_price) = self.orders.entry_price() {
                let size = balance.abs();
                if balance > Decimal::ZERO {
                    return Some(entry_price - (sl_pnl / size));
                } else if balance < Decimal::ZERO {
                    return Some(entry_price + (sl_pnl / size));
                }
            }
        }
        None
    }
}
