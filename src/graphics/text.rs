use crate::models::Color;
use font_kit::font::Font;
use raqote::{DrawOptions, DrawTarget, Point, Source};
use std::fs;

pub struct TextRenderer {
    font: Font,
}

impl TextRenderer {
    pub fn new(font_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let font_data = fs::read(font_path.to_string())?;
        let font = Font::from_bytes(font_data.into(), 0)?;
        Ok(Self { font })
    }

    pub fn draw(
        &self,
        dt: &mut DrawTarget,
        text: &str,
        left: i32,
        top: i32,
        height: i32,
        color: Color,
    ) {
        dt.draw_text(
            &self.font,
            (height * 72 / 96) as f32,
            text,
            Point::new(left as f32, top as f32),
            &Source::Solid(color.into()),
            &DrawOptions::new(),
        );
    }
}
