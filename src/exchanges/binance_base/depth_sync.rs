//! Keeps the DOM in step with Binance's diff-depth stream.
//!
//! Binance documents how a local book must be built: buffer the stream, fetch
//! a REST snapshot, drop events already inside it, apply the first one that
//! bridges it, then check that every later event continues the sequence
//! (spot: `U == previous u + 1`; USD-M futures: `pu == previous u`). A missed
//! event means the book is wrong until a new snapshot is taken.
//!
//! The sequencing is done by `crypto_orderbook::BookSync`, a sans-IO state
//! machine; this module mirrors its book into the shared [`OrderBookState`]
//! and clears the DOM (and marks it offline) whenever the book cannot be
//! trusted, instead of leaving stale levels on screen.

use crate::models::OrderBookState;
use crypto_orderbook::{
    BookSync, DepthSnapshot, DepthUpdate, OrderBook, Outcome, PriceLevel, SnapshotOutcome,
    SyncState,
};
use rust_decimal::Decimal;
use std::str::FromStr;

pub use crypto_orderbook::{Market, parse_depth_update};

pub struct DomSync {
    sync: BookSync,
    book: OrderBook,
}

impl DomSync {
    pub fn new(market: Market) -> Self {
        Self {
            sync: BookSync::new(market),
            book: OrderBook::new(),
        }
    }

    /// True when a REST snapshot should be fetched and passed to [`Self::on_snapshot`].
    pub fn needs_snapshot(&self) -> bool {
        self.sync.needs_snapshot()
    }

    /// True when a snapshot with this `lastUpdateId` predates the buffered events.
    pub fn snapshot_is_too_old(&self, last_update_id: u64) -> bool {
        self.sync.snapshot_is_too_old(last_update_id)
    }

    pub fn is_live(&self) -> bool {
        self.sync.is_live()
    }

    /// Feed one diff-depth event and mirror the result into `dom`.
    pub fn on_update(&mut self, update: DepthUpdate, dom: &mut OrderBookState) -> Outcome {
        // Only an event that can be applied needs its levels kept for the DOM.
        let diff = (self.sync.state() != SyncState::Buffering)
            .then(|| (update.bids.clone(), update.asks.clone()));
        let outcome = self.sync.on_event(&mut self.book, update);
        match outcome {
            Outcome::Applied => match diff {
                Some((bids, asks)) => {
                    for l in bids {
                        dom.update_bid(l.price, l.quantity);
                    }
                    for l in asks {
                        dom.update_ask(l.price, l.quantity);
                    }
                }
                None => self.mirror(dom),
            },
            Outcome::Resync(_) => self.mirror(dom),
            Outcome::Buffered | Outcome::Skipped => {}
        }
        dom.online = self.is_live();
        outcome
    }

    /// Apply a REST snapshot, replay the buffered events onto it and mirror the book into `dom`.
    pub fn on_snapshot(
        &mut self,
        last_update_id: u64,
        bids: &[[String; 2]],
        asks: &[[String; 2]],
        dom: &mut OrderBookState,
    ) -> SnapshotOutcome {
        let snapshot = DepthSnapshot {
            last_update_id,
            bids: parse_levels(bids),
            asks: parse_levels(asks),
        };
        let out = self.sync.on_snapshot(&mut self.book, snapshot);
        self.mirror(dom);
        dom.online = self.is_live();
        out
    }

    fn mirror(&self, dom: &mut OrderBookState) {
        dom.init_snapshot(
            self.book
                .top_bids(usize::MAX)
                .into_iter()
                .map(|l| (l.price, l.quantity))
                .collect(),
            self.book
                .top_asks(usize::MAX)
                .into_iter()
                .map(|l| (l.price, l.quantity))
                .collect(),
        );
    }
}

fn parse_levels(levels: &[[String; 2]]) -> Vec<PriceLevel> {
    levels
        .iter()
        .filter_map(|[p, q]| {
            Some(PriceLevel {
                price: Decimal::from_str(p).ok()?,
                quantity: Decimal::from_str(q).ok()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> String {
        v.to_string()
    }

    fn spot(first: u64, last: u64, bids: &str) -> DepthUpdate {
        parse_depth_update(&format!(
            r#"{{"e":"depthUpdate","E":1,"s":"BTCUSDT","U":{first},"u":{last},"b":{bids},"a":[]}}"#
        ))
        .unwrap()
    }

    fn futures(first: u64, last: u64, prev: u64, bids: &str) -> DepthUpdate {
        parse_depth_update(&format!(
            r#"{{"e":"depthUpdate","E":1,"s":"BTCUSDT","U":{first},"u":{last},"pu":{prev},"b":{bids},"a":[]}}"#
        ))
        .unwrap()
    }

    fn bid(dom: &OrderBookState, price: &str) -> Option<Decimal> {
        dom.bids.get(&Decimal::from_str(price).unwrap()).copied()
    }

    /// Snapshot at 100 with a bid of 10 @ 100.0, connected by event 99..=101.
    fn live_spot(dom: &mut OrderBookState) -> DomSync {
        let mut sync = DomSync::new(Market::Spot);
        sync.on_update(spot(99, 101, r#"[["100.5","2"]]"#), dom);
        assert!(sync.needs_snapshot());
        let out = sync.on_snapshot(100, &[[s("100.0"), s("10")]], &[[s("101.0"), s("1")]], dom);
        assert_eq!((out.applied, out.resync), (1, None));
        assert!(sync.is_live() && dom.online);
        sync
    }

    #[test]
    fn events_before_the_snapshot_are_replayed_onto_it() {
        let mut dom = OrderBookState::new();
        let _ = live_spot(&mut dom);
        assert_eq!(bid(&dom, "100.5"), Some(Decimal::from(2)));
        assert_eq!(bid(&dom, "100.0"), Some(Decimal::from(10)));
    }

    #[test]
    fn contiguous_events_update_the_dom() {
        let mut dom = OrderBookState::new();
        let mut sync = live_spot(&mut dom);
        let out = sync.on_update(spot(102, 103, r#"[["100.0","0"]]"#), &mut dom);
        assert_eq!(out, Outcome::Applied);
        assert_eq!(bid(&dom, "100.0"), None);
        assert_eq!(bid(&dom, "100.5"), Some(Decimal::from(2)));
    }

    #[test]
    fn a_dropped_spot_event_clears_the_dom_instead_of_leaving_stale_levels() {
        let mut dom = OrderBookState::new();
        let mut sync = live_spot(&mut dom);
        // Event 102..=103 is lost (it removed the 100.0 bid). The old code
        // applied 104..=105 anyway and kept showing 10 @ 100.0 forever.
        let out = sync.on_update(spot(104, 105, r#"[["99.0","3"]]"#), &mut dom);
        assert!(matches!(out, Outcome::Resync(_)));
        assert!(dom.bids.is_empty() && dom.asks.is_empty());
        assert!(!dom.online);
        assert!(sync.needs_snapshot());

        // The next event is buffered and a fresh snapshot brings the DOM back.
        sync.on_update(spot(106, 107, "[]"), &mut dom);
        sync.on_snapshot(105, &[[s("99.0"), s("3")]], &[], &mut dom);
        assert!(sync.is_live() && dom.online);
        assert_eq!(bid(&dom, "99.0"), Some(Decimal::from(3)));
        assert_eq!(bid(&dom, "100.0"), None);
    }

    #[test]
    fn a_snapshot_that_does_not_bridge_the_stream_is_rejected() {
        let mut dom = OrderBookState::new();
        let mut sync = DomSync::new(Market::Spot);
        sync.on_update(spot(200, 201, "[]"), &mut dom);
        assert!(sync.snapshot_is_too_old(150));
        assert!(!sync.snapshot_is_too_old(199));
    }

    #[test]
    fn futures_follow_pu_and_resync_on_a_break() {
        let mut dom = OrderBookState::new();
        let mut sync = DomSync::new(Market::UsdM);
        sync.on_update(futures(95, 101, 94, r#"[["50.0","1"]]"#), &mut dom);
        sync.on_snapshot(100, &[[s("49.0"), s("4")]], &[], &mut dom);
        assert!(sync.is_live());
        assert_eq!(bid(&dom, "50.0"), Some(Decimal::from(1)));

        let ok = sync.on_update(futures(102, 104, 101, r#"[["50.0","5"]]"#), &mut dom);
        assert_eq!(ok, Outcome::Applied);
        assert_eq!(bid(&dom, "50.0"), Some(Decimal::from(5)));

        // pu 110 does not point back to u 104: an event was missed.
        let gap = sync.on_update(futures(111, 112, 110, "[]"), &mut dom);
        assert!(matches!(gap, Outcome::Resync(_)));
        assert!(dom.bids.is_empty() && !dom.online);
    }
}
