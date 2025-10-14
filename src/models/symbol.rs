use rust_decimal::Decimal;

pub struct Symbol {
    pub slug: String,
    pub tick_size: Decimal,
}
