use crate::models::Message;
use crate::models::{
    Interval, NewOrder, Order, SharedCandlesState, SharedDomState, SharedOpenInterestState,
    SharedOrderFlowState, Symbol,
};
use std::sync::mpsc::Sender;

pub trait Exchange: Send + Sync {
    fn start(
        &mut self,
        symbol: &str,
        candles: SharedCandlesState,
        dom: SharedDomState,
        open_interest: SharedOpenInterestState,
        order_flow: SharedOrderFlowState,
        messages_sender: Sender<Message>,
        orders_sender: Sender<Order>,
    ) -> Result<Symbol, dyn std::error::Error>;

    fn stop() -> ();

    fn set_interval(&mut self, interval: Interval) -> ();

    fn submit_order(&self, new_order: NewOrder) -> ();

    fn cancel_order(&self, order: Order) -> ();
}
