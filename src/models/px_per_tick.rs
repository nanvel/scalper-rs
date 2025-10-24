use rust_decimal::Decimal;

pub struct PxPerTick {
    value: Decimal,
    choices: Vec<Decimal>,
}

impl PxPerTick {
    pub fn new(value: Decimal, choices: Vec<Decimal>) -> Self {
        Self { value, choices }
    }

    pub fn scale_in(&mut self) {
        if let Some(pos) = self.choices.iter().position(|&x| x == self.value) {
            if pos > 0 {
                self.value = self.choices[pos - 1];
            }
        }
    }

    pub fn scale_out(&mut self) {
        if let Some(pos) = self.choices.iter().position(|&x| x == self.value) {
            if pos + 1 < self.choices.len() {
                self.value = self.choices[pos + 1];
            }
        }
    }

    pub fn get(&self) -> Decimal {
        self.value
    }
}
