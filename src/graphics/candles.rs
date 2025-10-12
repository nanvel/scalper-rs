use crate::data::{Candle, Color};
use raqote::{
    DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, SolidSource, Source, StrokeStyle,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::cmp;

pub struct CandlesRenderer {
    width: i32,
    height: i32,
    padding: i32,
    bg_color: Color,
}

impl CandlesRenderer {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            padding: 10,
            bg_color: Color::WHITE,
        }
    }

    pub fn render(&self, candles: &[Candle]) -> DrawTarget {
        let mut dt = DrawTarget::new(self.width, self.height);

        dt.clear(SolidSource::from_unpremultiplied_argb(
            self.bg_color.a,
            self.bg_color.r,
            self.bg_color.g,
            self.bg_color.b,
        ));

        if candles.is_empty() {
            return dt;
        }

        let mut min_price: Decimal = candles[0].low;
        let mut max_price: Decimal = candles[0].high;

        for candle in candles {
            if candle.low < min_price {
                min_price = candle.low;
            }
            if candle.high > max_price {
                max_price = candle.high;
            }
        }

        let price_range = max_price - min_price;
        let height_range = Decimal::from(self.height - 2 * self.padding);
        let candle_width = cmp::min((self.width - 2 * self.padding) / (candles.len() as i32), 10);
        let body_width = (candle_width as f32 * 0.7).max(1.0) as i32;

        let price_to_y = |price: Decimal| -> i32 {
            let relative_price = price - min_price;
            let y = height_range - (relative_price * height_range / price_range);
            (y + Decimal::from(self.padding))
                .to_i32()
                .unwrap_or(self.padding)
        };

        for (i, candle) in candles.iter().enumerate() {
            let x = self.padding + (i as i32) * candle_width;

            let open_y = price_to_y(candle.open);
            let close_y = price_to_y(candle.close);
            let high_y = price_to_y(candle.high);
            let low_y = price_to_y(candle.low);

            let color = if candle.is_bullish() {
                SolidSource::from_unpremultiplied_argb(255, 0, 170, 0) // Green
            } else {
                SolidSource::from_unpremultiplied_argb(255, 204, 0, 0) // Red
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

            if candle.is_bullish() {
                // Bullish: fill with color
                dt.fill(&path, &Source::Solid(color), &DrawOptions::new());
            } else {
                // Bearish: fill with color
                dt.fill(&path, &Source::Solid(color), &DrawOptions::new());
            }

            // Add border to body
            dt.stroke(
                &path,
                &Source::Solid(SolidSource::from_unpremultiplied_argb(
                    0xff, 0x33, 0x33, 0x33,
                )),
                &StrokeStyle {
                    width: 0.5,
                    ..Default::default()
                },
                &DrawOptions::new(),
            );
        }

        dt
    }
}
