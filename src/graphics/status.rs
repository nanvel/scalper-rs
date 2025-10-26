use super::text::TextRenderer;
use crate::models::{Area, ColorSchema, Interval};
use chrono::Utc;
use raqote::{DrawOptions, DrawTarget, Source};
use rust_decimal::Decimal;

pub struct StatusRenderer {
    area: Area,
    padding: i32,
}

impl StatusRenderer {
    pub fn new(area: Area) -> Self {
        Self { area, padding: 2 }
    }

    pub fn render(
        &self,
        interval: Interval,
        size: Decimal,
        dt: &mut DrawTarget,
        text_renderer: &TextRenderer,
        color_schema: &ColorSchema,
        pnl: Decimal,
        balance: Decimal,
    ) {
        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(color_schema.background.into()),
            &DrawOptions::new(),
        );

        text_renderer.draw(
            dt,
            interval.slug(),
            self.area.left + self.padding * 2,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );

        text_renderer.draw(
            dt,
            &(size.to_string() + "$"),
            self.area.left + self.padding * 2 + 25,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );

        text_renderer.draw(
            dt,
            &pnl.to_string(),
            self.area.left + self.area.width - 100,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );
        text_renderer.draw(
            dt,
            &balance.to_string(),
            self.area.left + self.area.width - 200,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );

        let now = Utc::now();
        text_renderer.draw(
            dt,
            &now.format("%H:%M:%S").to_string(),
            self.area.left + self.padding * 2 + 80,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );
    }
}
