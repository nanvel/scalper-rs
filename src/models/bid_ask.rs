use rust_decimal::Decimal;

pub struct BidAsk {
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
}

impl Default for BidAsk {
    fn default() -> Self {
        BidAsk {
            bid: None,
            ask: None,
        }
    }
}

impl BidAsk {
    pub fn update(&mut self, bid: Option<Decimal>, ask: Option<Decimal>) {
        if bid.is_some() {
            self.bid = bid;
        }
        if ask.is_some() {
            self.ask = ask;
        }
    }

    pub fn is_some(&self) -> bool {
        self.bid.is_some() && self.ask.is_some()
    }
}
