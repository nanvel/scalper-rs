use crate::data::Config;
use font_kit::font::Font;
use raqote::{DrawOptions, DrawTarget, Point, Source};
use std::fs;

pub struct StatusRenderer {
    width: i32,
    height: i32,
    offset_left: i32,
    offset_top: i32,
    padding: i32,
}

impl StatusRenderer {
    pub fn new(width: i32, height: i32, offset_left: i32, offset_top: i32) -> Self {
        Self {
            width,
            height,
            offset_left,
            offset_top,
            padding: 1,
        }
    }

    pub fn render(&self, symbol: &str, dt: &mut DrawTarget, config: &Config) {
        dt.fill_rect(
            self.offset_left as f32,
            self.offset_top as f32,
            self.width as f32,
            self.height as f32,
            &Source::Solid(config.background_color.into()),
            &DrawOptions::new(),
        );

        let font_data = fs::read("/System/Library/Fonts/SFNSMono.ttf".to_string())
            .expect("Failed to read font file");
        let font = Font::from_bytes(font_data.into(), 0).expect("Failed to load font");

        dt.draw_text(
            &font,
            ((self.height - self.padding * 2) * 72 / 96) as f32,
            symbol,
            Point::new(
                (self.offset_left + self.padding) as f32,
                (self.offset_top + self.height - self.padding) as f32,
            ),
            &Source::Solid(config.text_color.into()),
            &DrawOptions::new(),
        );
    }
}
