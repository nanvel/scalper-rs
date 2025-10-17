use rust_decimal::Decimal;

pub struct Scale {
    pub central_price: Decimal,
    pub central_point: i32,
    pub price_to_points: Decimal,
    pub dom_tick: Decimal,
}

impl Default for Scale {
    fn default() -> Self {
        Self {
            central_price: Decimal::ZERO,
            central_point: 0,
            price_to_points: Decimal::ONE,
            dom_tick: Decimal::ZERO,
        }
    }
}
