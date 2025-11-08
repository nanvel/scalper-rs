use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub slug: String,
    pub tick_size: Decimal,
    pub step_size: Decimal,
    pub min_notional: Decimal,
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
        if total < self.min_notional {
            let min_qty = (self.min_notional / price)
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
