use crate::exchanges::base::exchange::Exchange;
use crate::models::{
    SharedCandlesState, SharedDomState, SharedOpenInterestState, SharedOrderFlowState, Symbol,
};

struct BinanceFuturesExchange {
    name: String,
    api_url: String,
    websocket_url: String,
}

impl Exchange for BinanceFuturesExchange {
    fn listen(
        &mut self,
        symbol: Symbol,
        candles: SharedCandlesState,
        dom: SharedDomState,
        open_interest: SharedOpenInterestState,
        order_flow: SharedOrderFlowState,
    ) -> () {
        // Implementation for listening to market data
    }

    fn stop(&mut self) -> () {
        // Implementation for stopping the exchange connection
    }

    fn submit_order(
        &self,
        symbol: &crate::models::Symbol,
        quantity: f64,
        price: f64,
        side: String,
    ) -> () {
        // Implementation for submitting an order
    }

    fn cancel_order(&self, symbol: &crate::models::Symbol, order_id: &str) -> () {
        // Implementation for canceling an order
    }
}
