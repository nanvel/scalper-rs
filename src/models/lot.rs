use crate::models::Symbol;
use rust_decimal::Decimal;

pub struct Lot {
    size: Decimal,
    size_base: Option<Decimal>,
    multipliers: [usize; 4],
    selected: usize,
}

impl Lot {
    pub fn new(size: Decimal, multipliers: [usize; 4]) -> Self {
        Lot {
            size,
            size_base: None,
            multipliers,
            selected: 0,
        }
    }

    pub fn select_size(&mut self, index: usize) {
        self.selected = index;
    }

    pub fn get_size(&self) -> Decimal {
        self.size
    }

    pub fn get_size_base(&self) -> Option<Decimal> {
        self.size_base
    }

    pub fn get_multiplier(&self) -> usize {
        self.multipliers[self.selected]
    }

    pub fn get_quote(&self) -> Decimal {
        self.size * Decimal::from(self.multipliers[self.selected])
    }

    pub fn get_value(&mut self, price: Decimal, symbol: &Symbol) -> Decimal {
        if let Some(size_base) = &self.size_base {
            return size_base * Decimal::from(self.multipliers[self.selected]);
        }

        let size = symbol.tune_quantity(self.size / price, price);
        self.size_base = Some(size);

        size * Decimal::from(self.multipliers[self.selected])
    }
}
