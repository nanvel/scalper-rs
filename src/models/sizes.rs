use crate::models::Symbol;
use rust_decimal::Decimal;

pub struct Sizes {
    quotes: [Decimal; 3],
    values: Option<[Decimal; 3]>,
    selected: usize,
}

impl Sizes {
    pub fn new(quotes: [Decimal; 3]) -> Self {
        Sizes {
            quotes,
            values: None,
            selected: 0,
        }
    }

    pub fn select_size(&mut self, index: usize) {
        self.selected = index;
    }

    pub fn get_quote(&self) -> Decimal {
        self.quotes[self.selected]
    }

    pub fn get_value(&mut self, price: Decimal, symbol: &Symbol) -> Decimal {
        if let Some(v) = &self.values {
            return v[self.selected];
        }

        let mut arr = [Decimal::ZERO; 3];
        for (i, q) in self.quotes.iter().enumerate() {
            arr[i] = symbol.tune_quantity(q / price, price);
        }

        let selected_value = arr[self.selected];
        self.values = Some(arr);
        selected_value
    }
}
