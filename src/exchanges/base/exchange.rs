use crate::models::{Interval, NewOrder, SharedState, Symbol};

pub trait Exchange: Send + Sync {
    fn name(&self) -> &str;

    fn start(
        &mut self,
        interval: Interval,
    ) -> Result<(Symbol, SharedState), Box<dyn std::error::Error>>;

    fn stop(&mut self) -> ();

    fn set_interval(&self, interval: Interval) -> ();

    fn place_order(&self, new_order: NewOrder) -> ();

    fn cancel_order(&self, order_id: String) -> ();
}
