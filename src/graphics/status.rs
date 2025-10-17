use crate::models::{Area, Config};
use font_kit::font::Font;
use raqote::{DrawOptions, DrawTarget, Point, Source};
use std::fs;

pub struct StatusRenderer {
    area: Area,
    padding: i32,
}

impl StatusRenderer {
    pub fn new(area: Area) -> Self {
        Self { area, padding: 1 }
    }

    pub fn render(&self, symbol: &str, dt: &mut DrawTarget, config: &Config) {
        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(config.background_color.into()),
            &DrawOptions::new(),
        );

        let font_data = fs::read("/System/Library/Fonts/SFNSMono.ttf".to_string())
            .expect("Failed to read font file");
        let font = Font::from_bytes(font_data.into(), 0).expect("Failed to load font");

        dt.draw_text(
            &font,
            ((self.area.height - self.padding * 2) * 72 / 96) as f32,
            symbol,
            Point::new(
                (self.area.left + self.padding) as f32,
                (self.area.top + self.area.height - self.padding) as f32,
            ),
            &Source::Solid(config.text_color.into()),
            &DrawOptions::new(),
        );
    }
}
