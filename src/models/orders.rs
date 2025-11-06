use crate::models::Timestamp;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
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

#[derive(Debug)]
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

    pub fn on_order(&mut self, order: Order) {
        if let Some(pos) = self.orders.iter().position(|o| o.id == order.id) {
            if self.orders[pos].order_status == OrderStatus::Pending {
                self.orders[pos] = order;
            }
        } else {
            self.orders.push(order);
        }
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
        if !bid.is_some() && !ask.is_some() {
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

    pub fn all(&self) -> Vec<&Order> {
        self.orders
            .iter()
            .filter(|o| {
                o.order_status == OrderStatus::Pending || o.executed_quantity > Decimal::from(0)
            })
            .collect()
    }
}
