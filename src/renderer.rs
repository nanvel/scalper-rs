use crate::models::Layout;
use crate::models::SharedState;
use crate::trader::Trader;
use raqote::DrawTarget;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromStr, ToPrimitive};

const PX_PER_TICK_CHOICES: [&str; 17] = [
    "0.01", "0.02", "0.05", "0.1", "0.2", "0.5", "1", "3", "5", "7", "9", "11", "13", "15", "17",
    "19", "21",
];

pub struct Renderer {
    pub width: usize,
    pub height: usize,
    book_entry_range: Decimal,
    center_px: usize,
    center_price: Decimal,
    px_per_tick: Decimal,
    tick_size: Decimal,
}

impl Renderer {
    pub fn new(width: usize, height: usize, tick_size: Decimal) -> Self {
        Self {
            width,
            height,
            book_entry_range: Decimal::ZERO,
            center_px: height / 2,
            center_price: Decimal::ZERO,
            px_per_tick: Decimal::from(1),
            tick_size,
        }
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    pub fn scale_in(&mut self) {
        if let Some(pos) = PX_PER_TICK_CHOICES
            .iter()
            .position(|&x| Decimal::from_str(x).unwrap() == self.px_per_tick)
        {
            if pos > 0 {
                self.px_per_tick = Decimal::from_str(PX_PER_TICK_CHOICES[pos - 1]).unwrap();
            }
        }
    }

    pub fn scale_out(&mut self) {
        if let Some(pos) = PX_PER_TICK_CHOICES
            .iter()
            .position(|&x| Decimal::from_str(x).unwrap() == self.px_per_tick)
        {
            if pos + 1 < PX_PER_TICK_CHOICES.len() {
                self.px_per_tick = Decimal::from_str(PX_PER_TICK_CHOICES[pos + 1]).unwrap();
            }
        }
    }

    pub fn render(
        &mut self,
        dt: &mut DrawTarget,
        shared_state: &SharedState,
        trader: &Trader,
        status: String,
        locked: bool,
        force_redraw: bool,
    ) {
        let price;
        if let Some(last_candle) = shared_state.candles.read().unwrap().last() {
            price = last_candle.close;
        } else {
            return;
        }
        if self.center_price == Decimal::ZERO {
            self.center_price = price;
        }
        if !locked {
            self.adjust_center(price);
        }

        let layout = Layout::new(self.width as i32, self.height as i32);

        let price_to_y = |price: Decimal| -> i32 {
            (self.center_px as i32)
                + ((self.center_price - price) / self.tick_size * self.px_per_tick)
                    .to_i32()
                    .unwrap_or(0)
        };

        // todo: check if should redraw candles
        self.draw_candles();
    }

    fn draw_candles(&mut self) {}

    fn adjust_center(&mut self, price: Decimal) {
        if (price - self.center_price).abs() / self.tick_size * self.px_per_tick
            >= Decimal::from(self.height / 4)
        {
            self.center_price = price;
            self.center_px = self.height / 2;
        }
    }
}
