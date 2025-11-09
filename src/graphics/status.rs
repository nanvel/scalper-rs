use super::text::TextRenderer;
use crate::models::{Area, ColorSchema, Interval, Orders};
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
        orders: &Orders,
        bid: &Option<Decimal>,
        ask: &Option<Decimal>,
    ) {
        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(color_schema.background.into()),
            &DrawOptions::new(),
        );

        let now = Utc::now();
        let left_text = format!(
            "{} <{}> {} {}L {}S",
            interval.slug(),
            size.to_string(),
            now.format("%H:%M:%S").to_string(),
            orders.open_limit().to_string(),
            orders.open_stop().to_string(),
        );
        text_renderer.draw(
            dt,
            &left_text,
            self.area.left + self.padding * 2,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );

        let pnl = orders.pnl(*bid, *ask);
        let balance = orders.base_balance();
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
    }
}
