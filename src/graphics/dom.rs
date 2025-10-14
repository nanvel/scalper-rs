use crate::models::{Area, Config, DomState};
use raqote::{DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, Source, StrokeStyle};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use tokio::sync::RwLockReadGuard;

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
    ) {
        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(config.background_color.into()),
            &DrawOptions::new(),
        );

        let bids: Vec<_> = dom_state.get_bids(20, tick_size);
        let asks: Vec<_> = dom_state.get_asks(20, tick_size);
        if bids.is_empty() && asks.is_empty() {
            return;
        }

        let bar_height = 10.0;
        let gap = 2.0;
        let max_value = bids
            .iter()
            .chain(asks.iter())
            .map(|(_, v)| v.to_f32().unwrap_or(0.0))
            .fold(0.0, f32::max);

        let left = self.area.left as f32 + self.padding as f32;
        let right = (self.area.left + self.area.width) as f32 - self.padding as f32;
        let max_width = right - left;

        // Draw bids (bottom up)
        let mut y = (self.area.top + self.area.height) as f32 - self.padding as f32 - bar_height;
        for (price, value) in bids {
            let width = if max_value > 0.0 {
                value.to_f32().unwrap_or(0.0) / max_value * max_width
            } else {
                0.0
            };
            dt.fill_rect(
                left,
                y,
                width,
                bar_height,
                &Source::Solid(config.bullish_color.into()),
                &DrawOptions::new(),
            );
            y -= bar_height + gap;
        }

        // Draw asks (top down)
        let mut y = self.area.top as f32 + self.padding as f32;
        for (price, value) in asks {
            let width = if max_value > 0.0 {
                value.to_f32().unwrap_or(0.0) / max_value * max_width
            } else {
                0.0
            };
            dt.fill_rect(
                left,
                y,
                width,
                bar_height,
                &Source::Solid(config.bearish_color.into()),
                &DrawOptions::new(),
            );
            y += bar_height + gap;
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
