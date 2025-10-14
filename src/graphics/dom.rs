use crate::data::{Config, DomState};
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Source};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use tokio::sync::RwLockReadGuard;

pub struct DomRenderer {
    width: i32,
    height: i32,
    offset_left: i32,
    offset_top: i32,
    padding: i32,
}

impl DomRenderer {
    pub fn new(width: i32, height: i32, offset_left: i32, offset_top: i32) -> Self {
        Self {
            width,
            height,
            offset_left,
            offset_top,
            padding: 10,
        }
    }

    pub fn render(
        &self,
        dom_state: RwLockReadGuard<DomState>,
        dt: &mut DrawTarget,
        config: &Config,
    ) {
        dt.fill_rect(
            self.offset_left as f32,
            self.offset_top as f32,
            self.width as f32,
            self.height as f32,
            &Source::Solid(config.background_color.into()),
            &DrawOptions::new(),
        );

        let bids: Vec<_> = dom_state.bids.iter().collect();
        let asks: Vec<_> = dom_state.asks.iter().collect();
        if bids.is_empty() && asks.is_empty() {
            return;
        }

        // Find min and max price for scaling
        let min_price = bids
            .first()
            .map(|(p, _)| **p)
            .into_iter()
            .chain(asks.first().map(|(p, _)| **p))
            .min();
        let max_price = bids
            .last()
            .map(|(p, _)| **p)
            .into_iter()
            .chain(asks.last().map(|(p, _)| **p))
            .max();
        let (min_price, max_price) = match (min_price, max_price) {
            (Some(min), Some(max)) => (min, max),
            (Some(min), None) => (min, min),
            (None, Some(max)) => (max, max),
            (None, None) => return,
        };
        let price_range = max_price - min_price;
        let height_range = Decimal::from(self.height - 2 * self.padding);

        // Find max quantity for scaling bar length
        let max_qty = bids
            .iter()
            .chain(asks.iter())
            .map(|(_, qty)| *qty)
            .max_by(|a, b| a.cmp(b))
            .cloned()
            .unwrap_or(Decimal::ONE);
        let max_qty_f = max_qty.to_f32().unwrap_or(1.0);
        let bar_max_width = (self.width - self.offset_left - 2 * self.padding) as f32;

        // Draw bids
        for (price, qty) in bids {
            let price_f = (*price - min_price).to_f32().unwrap_or(0.0);
            let price_norm = if price_range.is_zero() {
                0.0
            } else {
                price_f / price_range.to_f32().unwrap_or(1.0)
            };
            let y = self.height + self.offset_top
                - self.padding
                - (price_norm * height_range.to_f32().unwrap_or(1.0)) as i32;
            let bar_len = ((*qty).to_f32().unwrap_or(0.0) / max_qty_f * bar_max_width) as i32;
            let mut pb = PathBuilder::new();
            pb.rect(
                (self.padding + self.offset_left) as f32,
                y as f32 - 4.0,
                bar_len as f32,
                8.0,
            );
            dt.fill(
                &pb.finish(),
                &Source::Solid(SolidSource::from(config.bullish_color)),
                &DrawOptions::new(),
            );
        }

        // Draw asks
        for (price, qty) in asks {
            let price_f = (*price - min_price).to_f32().unwrap_or(0.0);
            let price_norm = if price_range.is_zero() {
                0.0
            } else {
                price_f / price_range.to_f32().unwrap_or(1.0)
            };
            let y = self.height + self.offset_top
                - self.padding
                - (price_norm * height_range.to_f32().unwrap_or(1.0)) as i32;
            let bar_len = ((*qty).to_f32().unwrap_or(0.0) / max_qty_f * bar_max_width) as i32;
            let mut pb = PathBuilder::new();
            pb.rect(
                (self.width + self.offset_left) as f32 - self.padding as f32 - bar_len as f32,
                y as f32 - 4.0,
                bar_len as f32,
                8.0,
            );
            dt.fill(
                &pb.finish(),
                &Source::Solid(SolidSource::from(config.bearish_color)),
                &DrawOptions::new(),
            );
        }
    }
}
