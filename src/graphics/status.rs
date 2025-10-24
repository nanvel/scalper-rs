use crate::models::{Area, Config, Interval};
use font_kit::font::Font;
use raqote::{DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, Point, Source, StrokeStyle};
use std::fs;

pub struct StatusRenderer {
    area: Area,
    padding: i32,
}

impl StatusRenderer {
    pub fn new(area: Area) -> Self {
        Self { area, padding: 2 }
    }

    pub fn render(&self, interval: Interval, dt: &mut DrawTarget, config: &Config) {
        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(config.status_background_color.into()),
            &DrawOptions::new(),
        );

        let font_data = fs::read("/System/Library/Fonts/SFNSMono.ttf".to_string())
            .expect("Failed to read font file");
        let font = Font::from_bytes(font_data.into(), 0).expect("Failed to load font");
        let text_height = self.area.height - self.padding * 2;

        dt.draw_text(
            &font,
            (text_height * 72 / 96) as f32,
            interval.slug(),
            Point::new(
                (self.area.left + self.padding * 2) as f32,
                (self.area.top + self.area.height / 2 + self.padding * 2) as f32,
            ),
            &Source::Solid(config.text_color.into()),
            &DrawOptions::new(),
        );
    }
}
