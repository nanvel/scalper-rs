use rust_decimal::Decimal;

pub struct Symbol {
    pub slug: String,
    pub tick_size: Decimal,
    pub step_size: Decimal,
    pub notional: Decimal,
}

impl Symbol {
    pub fn tune_quantity(&self, quantity: Decimal, price: Decimal) -> Decimal {
        let qty = quantity
            .round_dp_with_strategy(
                self.step_size.normalize().scale(),
                rust_decimal::RoundingStrategy::ToZero,
            )
            .max(self.step_size);
        let total = (qty * price).round_dp_with_strategy(
            self.tick_size.normalize().scale(),
            rust_decimal::RoundingStrategy::ToZero,
        );
        if total < self.notional {
            let min_qty = (self.notional / price)
                .round_dp_with_strategy(
                    self.step_size.normalize().scale(),
                    rust_decimal::RoundingStrategy::ToZero,
                )
                .max(self.step_size);
            min_qty
        } else {
            qty
        }
    }
}
