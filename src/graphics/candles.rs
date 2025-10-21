use crate::models::{Area, CandlesState, Color, Config, Timestamp};
use font_kit::font::Font;
use raqote::{
    DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, Point, SolidSource, Source,
    StrokeStyle,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use std::fs;
use std::sync::RwLockReadGuard;

pub struct CandlesRenderer {
    area: Area,
    padding: i32,
    last_updated: Option<Timestamp>,
}

impl CandlesRenderer {
    pub fn new(area: Area) -> Self {
        Self {
            area,
            padding: 10,
            last_updated: None,
        }
    }

    pub fn render(
        &mut self,
        candles_state: RwLockReadGuard<CandlesState>,
        dt: &mut DrawTarget,
        config: &Config,
        tick_size: Decimal,
        center: Decimal,
        px_per_tick: Decimal,
    ) {
        if let Some(last_updated) = self.last_updated {
            if last_updated == candles_state.updated {
                return;
            }
        }
        self.last_updated = Some(candles_state.updated);

        dt.fill_rect(
            self.area.left as f32,
            self.area.top as f32,
            self.area.width as f32,
            self.area.height as f32,
            &Source::Solid(config.background_color.into()),
            &DrawOptions::new(),
        );

        let candles = candles_state.to_vec();
        if candles.is_empty() {
            return;
        }

        let current_price = candles.last().unwrap().close;
        let mut min_price: Decimal = candles[0].low;
        let mut max_price: Decimal = candles[0].high;
        for candle in &candles {
            if candle.low < min_price {
                min_price = candle.low;
            }
            if candle.high > max_price {
                max_price = candle.high;
            }
        }

        let candle_width = 15;
        let body_width = 11;
        let central_point = self.area.height / 2;

        let price_to_y = |price: Decimal| -> i32 {
            if price > center {
                return central_point
                    - ((price - center) / tick_size * px_per_tick)
                        .to_i32()
                        .unwrap_or(0);
            } else {
                return central_point
                    + ((center - price) / tick_size * px_per_tick)
                        .to_i32()
                        .unwrap_or(0);
            }
        };

        for (i, candle) in candles.iter().rev().enumerate() {
            let x = self.area.width + self.area.left - self.padding - (i as i32) * candle_width;

            let open_y = price_to_y(candle.open);
            let close_y = price_to_y(candle.close);
            let high_y = price_to_y(candle.high);
            let low_y = price_to_y(candle.low);

            let color: SolidSource = if candle.is_bullish() {
                config.bullish_color.into()
            } else {
                config.bearish_color.into()
            };

            let mut pb = PathBuilder::new();
            pb.move_to(x as f32, high_y as f32);
            pb.line_to(x as f32, low_y as f32);
            let path = pb.finish();

            dt.stroke(
                &path,
                &Source::Solid(color),
                &StrokeStyle {
                    width: 1.,
                    cap: LineCap::Round,
                    join: LineJoin::Round,
                    ..Default::default()
                },
                &DrawOptions::new(),
            );

            // Draw body (open-close rectangle)
            let body_top = open_y.min(close_y);
            let body_bottom = open_y.max(close_y);
            let body_height = (body_bottom - body_top).max(1);

            let mut pb = PathBuilder::new();
            pb.rect(
                (x - body_width / 2) as f32,
                body_top as f32,
                body_width as f32,
                body_height as f32,
            );
            let path = pb.finish();

            dt.fill(&path, &Source::Solid(color), &DrawOptions::new());
        }

        // scale
        let visible_ticks = (self.area.height as f64) / px_per_tick.to_f64().unwrap();
        let mut m = 1;
        while visible_ticks / m.to_f64().unwrap() > 10.0 {
            m *= 10;
        }
        if visible_ticks / m.to_f64().unwrap() > 5. {
            m /= 2;
        }
        let m = Decimal::from(m) * tick_size;
        let tick_price = (center / m).floor() * m;

        let font_data = fs::read("/System/Library/Fonts/SFNSMono.ttf".to_string())
            .expect("Failed to read font file");
        let font = Font::from_bytes(font_data.into(), 0).expect("Failed to load font");

        dt.draw_text(
            &font,
            (14 * 72 / 96) as f32,
            &tick_price.to_string(),
            Point::new((self.area.width - 50) as f32, price_to_y(tick_price) as f32),
            &Source::Solid(config.text_color.into()),
            &DrawOptions::new(),
        );

        let mut pb = PathBuilder::new();
        let delta = tick_price / Decimal::from(100);
        pb.rect(
            self.area.width as f32 - 3.0,
            price_to_y(tick_price + delta) as f32,
            2.0,
            (price_to_y(tick_price) - price_to_y(tick_price + delta)) as f32,
        );
        let path = pb.finish();

        dt.fill(
            &path,
            &Source::Solid(Color::BLUE.into()),
            &DrawOptions::new(),
        );

        for i in 0..5 {
            let tp = tick_price + m * Decimal::from(i);
            dt.draw_text(
                &font,
                (14 * 72 / 96) as f32,
                &tp.to_string(),
                Point::new((self.area.width - 50) as f32, price_to_y(tp) as f32),
                &Source::Solid(config.text_color.into()),
                &DrawOptions::new(),
            );
        }

        for i in 0..5 {
            let tp = tick_price - m * Decimal::from(i);
            dt.draw_text(
                &font,
                (14 * 72 / 96) as f32,
                &tp.to_string(),
                Point::new((self.area.width - 50) as f32, price_to_y(tp) as f32),
                &Source::Solid(config.text_color.into()),
                &DrawOptions::new(),
            );
        }

        // current price line
        let mut pb = PathBuilder::new();
        pb.move_to(self.area.left as f32, price_to_y(current_price) as f32);
        pb.line_to(
            (self.area.left + self.area.width) as f32,
            price_to_y(current_price) as f32,
        );
        let path = pb.finish();

        dt.stroke(
            &path,
            &Source::Solid(config.current_price_color.into()),
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

fn closest_significant(price: Decimal, lower: Decimal, upper: Decimal) -> Decimal {
    let diff = upper - lower;
    if diff <= Decimal::ZERO {
        return price;
    }
    let diff_f = diff.to_f64().unwrap_or(0.0);
    let magnitude = 10f64.powf(diff_f.log10().floor());
    let steps = [1.0, 2.0, 5.0];
    let mut best = price;
    let mut min_dist = f64::MAX;
    for step in steps {
        let candidate =
            (price.to_f64().unwrap_or(0.0) / (magnitude * step)).round() * magnitude * step;
        if candidate >= lower.to_f64().unwrap_or(0.0) && candidate <= upper.to_f64().unwrap_or(0.0)
        {
            let dist = (candidate - price.to_f64().unwrap_or(0.0)).abs();
            if dist < min_dist {
                min_dist = dist;
                best = Decimal::from_f64(candidate).unwrap();
            }
        }
    }
    best
}
