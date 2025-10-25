use crate::models::{Area, ColorSchema, DomState, Timestamp};
use raqote::{DrawOptions, DrawTarget, Source};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::sync::RwLockReadGuard;

pub struct DomRenderer {
    area: Area,
    padding: i32,
    last_updated: Option<Timestamp>,
}

impl DomRenderer {
    pub fn new(area: Area) -> Self {
        Self {
            area,
            padding: 10,
            last_updated: None,
        }
    }

    pub fn render(
        &mut self,
        dom_state: RwLockReadGuard<DomState>,
        dt: &mut DrawTarget,
        color_schema: &ColorSchema,
        tick_size: Decimal,
        center: Decimal,
        px_per_tick: Decimal,
        size_range: &mut Decimal,
        force_redraw: bool,
    ) {
        if !force_redraw {
            if let Some(last_updated) = self.last_updated {
                if last_updated == dom_state.updated {
                    return;
                }
            }
        }
        self.last_updated = Some(dom_state.updated);

        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(color_schema.background.into()),
            &DrawOptions::new(),
        );

        let left = self.area.left as f32;
        let right = (self.area.left + self.area.width) as f32 - self.padding as f32;
        let max_width = Decimal::from((right - left) as u32);
        let central_point = self.area.height / 2;

        let mut bid_buckets: Vec<Decimal> = vec![Decimal::ZERO; self.area.height as usize];
        let mut ask_buckets: Vec<Decimal> = vec![Decimal::ZERO; self.area.height as usize];

        for (price, quantity) in dom_state.bids.iter() {
            let price_diff = (center - *price) / tick_size;
            let px_offset = (price_diff * px_per_tick).to_i32().unwrap_or(0);
            let y = central_point + px_offset;
            if y < 0 || y >= self.area.height {
                continue;
            }

            bid_buckets[y as usize] += *quantity;
        }

        for (price, quantity) in dom_state.asks.iter() {
            let price_diff = (center - *price) / tick_size;
            let px_offset = (price_diff * px_per_tick).to_i32().unwrap_or(0);
            let y = central_point + px_offset;
            if y < 0 || y >= self.area.height {
                continue;
            }

            ask_buckets[y as usize] += *quantity;
        }

        let max_val = bid_buckets
            .iter()
            .cloned()
            .max()
            .unwrap_or(Decimal::ZERO)
            .max(ask_buckets.iter().cloned().max().unwrap_or(Decimal::ZERO))
            .max(*size_range);
        if max_val.is_zero() {
            return;
        }

        *size_range = max_val;

        for (i, val) in bid_buckets.iter().enumerate() {
            if val.is_zero() {
                continue;
            }

            let width = (val / max_val * max_width).to_f32().unwrap_or(0.0);
            let y = i as f32 + self.area.top as f32;

            if px_per_tick == Decimal::from(3) {
                dt.fill_rect(
                    left,
                    y - 1.0,
                    width,
                    3.0,
                    &Source::Solid(color_schema.bid_bar.into()),
                    &DrawOptions::new(),
                );
            } else if px_per_tick >= Decimal::from(5) {
                dt.fill_rect(
                    left,
                    y - (px_per_tick.to_f32().unwrap() / 2.0).floor(),
                    width,
                    px_per_tick.to_f32().unwrap() - 2.0,
                    &Source::Solid(color_schema.bid_bar.into()),
                    &DrawOptions::new(),
                );
            } else {
                dt.fill_rect(
                    left,
                    y,
                    width,
                    1.0,
                    &Source::Solid(color_schema.bid_bar.into()),
                    &DrawOptions::new(),
                );
            }
        }

        for (i, val) in ask_buckets.iter().enumerate() {
            if val.is_zero() {
                continue;
            }

            let width = (val / max_val * max_width).to_f32().unwrap_or(0.0);
            let y = i as f32 + self.area.top as f32;

            if px_per_tick == Decimal::from(3) {
                dt.fill_rect(
                    left,
                    y - 1.0,
                    width,
                    3.0,
                    &Source::Solid(color_schema.ask_bar.into()),
                    &DrawOptions::new(),
                );
            } else if px_per_tick >= Decimal::from(5) {
                dt.fill_rect(
                    left,
                    y - (px_per_tick.to_f32().unwrap() / 2.0).floor(),
                    width,
                    px_per_tick.to_f32().unwrap() - 2.0,
                    &Source::Solid(color_schema.ask_bar.into()),
                    &DrawOptions::new(),
                );
            } else {
                dt.fill_rect(
                    left,
                    y,
                    width,
                    1.0,
                    &Source::Solid(color_schema.ask_bar.into()),
                    &DrawOptions::new(),
                );
            }
        }
    }
}
