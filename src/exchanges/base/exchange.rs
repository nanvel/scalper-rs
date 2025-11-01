use crate::models::{
    SharedCandlesState, SharedDomState, SharedOpenInterestState, SharedOrderFlowState, Symbol,
};

pub trait Exchange: Send + Sync {
    fn listen(
        &mut self,
        symbol: Symbol,
        candles: SharedCandlesState,
        dom: SharedDomState,
        open_interest: SharedOpenInterestState,
        order_flow: SharedOrderFlowState,
    ) -> ();

    fn stop(&mut self) -> ();

    fn submit_order(&self, symbol: &Symbol, quantity: f64, price: f64, side: String) -> ();

    fn cancel_order(&self, symbol: &Symbol, order_id: &str) -> ();
}
