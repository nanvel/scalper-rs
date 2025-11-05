use crate::models::{Interval, NewOrder, Order, Symbol};

pub trait Exchange: Send + Sync {
    fn start(&mut self) -> Result<Symbol, dyn std::error::Error>;

    fn can_trade(&self) -> bool {
        false
    }

    fn stop(&self) -> ();

    fn set_interval(&mut self, interval: Interval) -> ();

    fn place_order(&self, new_order: NewOrder) -> ();

    fn cancel_order(&self, order: Order) -> ();
}
