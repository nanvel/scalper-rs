use super::text::TextRenderer;
use crate::models::{Area, ColorSchema, Interval, Status};
use crate::trader::Trader;
use chrono::Utc;
use f64_fixed::to_fixed_string;
use raqote::{DrawOptions, DrawTarget, Source};
use rust_decimal::prelude::ToPrimitive;

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
        dt: &mut DrawTarget,
        text_renderer: &TextRenderer,
        color_schema: &ColorSchema,
        trader: &Trader,
        status: &Status,
    ) {
        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(color_schema.background.into()),
            &DrawOptions::new(),
        );

        let status_text = match status {
            Status::Ok => "OK".to_string(),
            Status::Warning(msg) => msg.to_string(),
            Status::Critical(msg) => msg.to_string(),
        };
        let status_color = match status {
            Status::Ok => color_schema.status_ok,
            Status::Warning(_) => color_schema.status_warning,
            Status::Critical(_) => color_schema.status_critical,
        };

        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            50_f32,
            self.area.height as f32,
            &Source::Solid(status_color.into()),
            &DrawOptions::new(),
        );
        text_renderer.draw(
            dt,
            &format!("{:^6}", status_text),
            self.area.left + self.padding * 2,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );

        let now = Utc::now();

        let left_text = format!(
            "{} <{} X {}> {} {}",
            interval.slug(),
            trader.size_quote.to_string(),
            trader.get_size_multiplier().to_string(),
            now.format("%H:%M:%S UTC").to_string(),
            to_fixed_string(trader.get_lots(), 6),
        );
        text_renderer.draw(
            dt,
            &left_text,
            self.area.left + self.padding * 2 + 50,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );

        text_renderer.draw(
            dt,
            &to_fixed_string(trader.get_pnl().to_f64().unwrap(), 10),
            self.area.left + self.area.width - 200,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );
        text_renderer.draw(
            dt,
            &to_fixed_string(trader.get_commission().to_f64().unwrap(), 10),
            self.area.left + self.area.width - 100,
            self.area.top + self.area.height / 2 + self.padding * 2,
            self.area.height - self.padding * 2,
            color_schema.text_light,
        );
    }
}
