use crate::models::Timestamp;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderType {
    Limit,
    Market,
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
}

#[derive(Debug)]
pub struct NewOrder {
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub quantity: Decimal,
    pub executed_quantity: Decimal,
    pub price: Decimal,
    pub average_price: Decimal,
    pub commission: Decimal,
    pub timestamp: Timestamp,
    pub is_update: bool,
}

impl Order {
    pub fn new(
        id: String,
        order_type: OrderType,
        order_side: OrderSide,
        order_status: OrderStatus,
        quantity: Decimal,
        executed_quantity: Decimal,
        price: Decimal,
        average_price: Decimal,
        commission: Decimal,
        timestamp: Timestamp,
        is_update: bool,
    ) -> Self {
        Self {
            id,
            order_type,
            order_side,
            order_status,
            quantity,
            executed_quantity,
            price,
            average_price,
            commission,
            timestamp,
            is_update,
        }
    }
}

pub struct Orders {
    orders: Vec<Order>,
}

impl Orders {
    pub fn new() -> Self {
        Self { orders: Vec::new() }
    }

    pub fn consume(&mut self, order: Order) -> bool {
        let is_filled = order.order_status == OrderStatus::Filled;
        if let Some(pos) = self.orders.iter().position(|o| o.id == order.id) {
            if self.orders[pos].order_status == OrderStatus::Pending {
                self.orders[pos] = order;

                return is_filled;
            }
        } else {
            if !order.is_update {
                // do not insert updates and the order could be created outside the app
                self.orders.push(order);

                return is_filled;
            }
        }

        false
    }

    pub fn base_balance(&self) -> Decimal {
        let mut balance = Decimal::ZERO;
        for order in &self.orders {
            match order.order_side {
                OrderSide::Buy => balance += order.executed_quantity,
                OrderSide::Sell => balance -= order.executed_quantity,
            }
        }
        balance
    }

    pub fn commission(&self) -> Decimal {
        let mut total = Decimal::ZERO;
        for order in &self.orders {
            total += order.commission;
        }
        total
    }

    pub fn pnl(&self, bid: Option<Decimal>, ask: Option<Decimal>) -> Decimal {
        if !bid.is_some() || !ask.is_some() {
            return Decimal::ZERO;
        }

        let mut spent = Decimal::ZERO;
        let mut received = Decimal::ZERO;
        let mut base_balance = Decimal::ZERO;
        for order in &self.orders {
            match order.order_side {
                OrderSide::Buy => {
                    spent += order.average_price * order.executed_quantity;
                    base_balance += order.executed_quantity;
                }
                OrderSide::Sell => {
                    received += order.average_price * order.executed_quantity;
                    base_balance -= order.executed_quantity;
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

    pub fn open(&self) -> Vec<&Order> {
        self.orders
            .iter()
            .filter(|o| o.order_status == OrderStatus::Pending)
            .collect()
    }

    pub fn last_closed(&self) -> Option<&Order> {
        self.orders
            .iter()
            .filter(|o| {
                o.order_status == OrderStatus::Filled && o.executed_quantity > Decimal::ZERO
            })
            .max_by_key(|o| o.timestamp)
    }

    pub fn price_at_pnl(&self, pnl: Decimal) -> Option<Decimal> {
        let base_balance = self.base_balance();
        if base_balance == Decimal::ZERO {
            return None;
        }

        let mut spent = Decimal::ZERO;
        let mut received = Decimal::ZERO;
        for order in &self.orders {
            match order.order_side {
                OrderSide::Buy => {
                    spent += order.average_price * order.executed_quantity;
                }
                OrderSide::Sell => {
                    received += order.average_price * order.executed_quantity;
                }
            }
        }

        Some((pnl - received + spent) / base_balance)
    }
}
