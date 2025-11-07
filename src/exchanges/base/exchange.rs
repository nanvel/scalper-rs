use crate::models::{Interval, Log, NewOrder, Order, SharedState, Symbol};
use std::sync::mpsc::Receiver;

pub trait Exchange: Send + Sync {
    fn start(
        &mut self,
    ) -> Result<(Symbol, SharedState, Receiver<Order>, Receiver<Log>), Box<dyn std::error::Error>>;

    fn stop(&self) -> ();

    fn set_interval(&mut self, interval: Interval) -> ();

    fn place_order(&self, new_order: NewOrder) -> ();

    fn cancel_order(&self, order: Order) -> ();
}
