use crate::models::Color;
use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use raqote::{DrawOptions, DrawTarget, Point, Source};

pub struct TextRenderer {
    font: Font,
}

impl TextRenderer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let font = SystemSource::new()
            .select_best_match(&[FamilyName::Monospace], &Properties::new())
            .unwrap()
            .load()
            .unwrap();
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
