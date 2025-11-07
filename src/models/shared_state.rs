use super::candles::SharedCandlesState;
use super::open_interest::SharedOpenInterestState;
use super::order_book::SharedOrderBookState;
use super::order_flow::SharedOrderFlowState;

pub struct SharedState {
    pub candles: SharedCandlesState,
    pub order_book: SharedOrderBookState,
    pub open_interest: SharedOpenInterestState,
    pub order_flow: SharedOrderFlowState,
}
