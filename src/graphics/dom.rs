use crate::models::{Area, Config, DomState};
use raqote::{DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, Source, StrokeStyle};
use rust_decimal::Decimal;
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
        let max_width = right - left;
        let central_point = self.area.height as i32 / 2;

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
