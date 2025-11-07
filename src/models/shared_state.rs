use super::candles::SharedCandlesState;
use super::dom::SharedDomState;
use super::open_interest::SharedOpenInterestState;
use super::order_flow::SharedOrderFlowState;

pub struct SharedState {
    pub candles: SharedCandlesState,
    pub dom: SharedDomState,
    pub open_interest: SharedOpenInterestState,
    pub order_flow: SharedOrderFlowState,
}
