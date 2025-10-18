use crate::models::{Area, Config, DomState};
use raqote::{DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, Source, StrokeStyle};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::sync::RwLockReadGuard;

pub struct DomRenderer {
    area: Area,
    padding: i32,
}

impl DomRenderer {
    pub fn new(area: Area) -> Self {
        Self { area, padding: 10 }
    }

    pub fn render(
        &self,
        dom_state: RwLockReadGuard<DomState>,
        dt: &mut DrawTarget,
        config: &Config,
        tick_size: Decimal,
        center: Decimal,
        px_per_tick: Decimal,
    ) {
        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(config.background_color.into()),
            &DrawOptions::new(),
        );

        let left = self.area.left as f32 + self.padding as f32;
        let right = (self.area.left + self.area.width) as f32 - self.padding as f32;
        let max_width = Decimal::from((right - left) as u32);
        let central_point = self.area.height as i32 / 2;

        let mut buckets: Vec<Decimal> = vec![Decimal::ZERO; self.area.height as usize];

        for (price, quantity) in dom_state.bids.iter() {
            let price_diff = (center - *price) / tick_size;
            let px_offset = (price_diff * px_per_tick).to_i32().unwrap_or(0);
            let y = central_point + px_offset;
            if y < 0 || y >= self.area.height {
                continue;
            }

            buckets[y as usize] += *quantity;
        }

        for (price, quantity) in dom_state.asks.iter() {
            let price_diff = (center - *price) / tick_size;
            let px_offset = (price_diff * px_per_tick).to_i32().unwrap_or(0);
            let y = central_point + px_offset;
            if y < 0 || y >= self.area.height {
                continue;
            }

            buckets[y as usize] += *quantity;
        }

        let max_val = buckets.iter().cloned().max().unwrap_or(Decimal::ZERO);
        if max_val.is_zero() {
            return;
        }

        for (i, val) in buckets.iter().enumerate() {
            if val.is_zero() {
                continue;
            }

            let width = (val / max_val * max_width).to_f32().unwrap_or(0.0);
            let y = i as f32 + self.area.top as f32;

            dt.fill_rect(
                left,
                y,
                width,
                1.0,
                &Source::Solid(config.bullish_color.into()),
                &DrawOptions::new(),
            );
        }

        // border
        let mut pb = PathBuilder::new();
        pb.move_to(self.area.left as f32, self.area.height as f32);
        pb.line_to(
            (self.area.left + self.area.width) as f32,
            self.area.height as f32,
        );
        let path = pb.finish();

        dt.stroke(
            &path,
            &Source::Solid(config.border_color.into()),
            &StrokeStyle {
                width: 1.0,
                cap: LineCap::Round,
                join: LineJoin::Round,
                ..Default::default()
            },
            &DrawOptions::new(),
        );
    }
}
