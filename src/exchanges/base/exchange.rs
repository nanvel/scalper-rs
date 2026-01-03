use crate::models::{Interval, NewOrder, SharedState, Symbol};

pub trait Exchange: Send + Sync {
    /// Returns the exchange name that is being displayed in the window header.
    fn name(&self) -> &str;

    /// Should return Symbol and SharedState, and then keep them updated.
    /// Use channels to log errors and important events.
    fn start(
        &mut self,
        interval: Interval,
    ) -> Result<(Symbol, SharedState), Box<dyn std::error::Error>>;

    /// Gracefully stops all exchange activities and free resources.
    fn stop(&mut self) -> ();

    /// Sets `shared_candles_state.interval` and populate it with historical data.
    fn set_interval(&self, interval: Interval) -> ();

    /// Submits an order.
    /// This method should return immediately, spawn a new thread to submit the order,
    /// then communicate updates or errors using channels.
    fn place_order(&self, new_order: NewOrder) -> ();

    /// Cancels an existing order. Similar to `place_order`, this method should return immediately.
    fn cancel_order(&self, order_id: String) -> ();
}
