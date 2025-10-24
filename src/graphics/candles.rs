use crate::models::{Area, CandlesState, ColorSchema, OpenInterestState, Timestamp};
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
        open_interest_state: RwLockReadGuard<OpenInterestState>,
        dt: &mut DrawTarget,
        color_schema: &ColorSchema,
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
            &Source::Solid(color_schema.background.into()),
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
                color_schema.bullish_candle.into()
            } else {
                color_schema.bearish_candle.into()
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

        // Draw volume bars in the reserved bottom area
        let volume_height = (self.area.height / 6).min(60);
        let mut max_volume = Decimal::ZERO;
        let mut max_oi = Decimal::ZERO;
        let mut min_oi = Decimal::MAX;
        for c in &candles {
            if c.volume > max_volume {
                max_volume = c.volume;
            }
            let oi = open_interest_state
                .get(&c.open_time)
                .unwrap_or(Decimal::ZERO);
            if oi > max_oi {
                max_oi = oi;
            }
            if oi < min_oi && oi > Decimal::ZERO {
                min_oi = oi;
            }
        }
        let oi_diff = max_oi - min_oi;

        dt.fill_rect(
            self.area.left as f32,
            (self.area.height - volume_height - 5) as f32,
            self.area.width as f32,
            (volume_height + 5) as f32,
            &Source::Solid(color_schema.background.into()),
            &DrawOptions::new(),
        );

        dt.fill_rect(
            self.area.left as f32,
            (self.area.height - volume_height - 5) as f32,
            self.area.width as f32,
            1.,
            &Source::Solid(color_schema.border.into()),
            &DrawOptions::new(),
        );

        let vh_dec = Decimal::from_i32(volume_height).unwrap_or(Decimal::from(40));
        if max_volume > Decimal::ZERO {
            for (i, candle) in candles.iter().rev().enumerate() {
                let x = self.area.width + self.area.left - self.padding - (i as i32) * candle_width;

                // Compute bar height proportional to volume
                let bar_height = ((candle.volume / max_volume) * vh_dec)
                    .to_i32()
                    .unwrap_or(0)
                    .max(1);

                let oi_height = if max_oi > Decimal::ZERO {
                    (((open_interest_state
                        .get(&candle.open_time)
                        .unwrap_or(Decimal::ZERO)
                        - min_oi)
                        / oi_diff)
                        * vh_dec)
                        .to_i32()
                        .unwrap_or(0)
                } else {
                    0
                };

                let bar_top = (self.area.top + self.area.height) - bar_height;
                let bar_left = x - (body_width / 2);

                let vol_color: SolidSource = if candle.is_bullish() {
                    color_schema.bullish_candle.into()
                } else {
                    color_schema.bearish_candle.into()
                };

                let mut pb = PathBuilder::new();
                pb.rect(bar_left as f32, bar_top as f32, 3., bar_height as f32);
                let path = pb.finish();
                dt.fill(&path, &Source::Solid(vol_color), &DrawOptions::new());

                let oi_top = (self.area.top + self.area.height) - oi_height;
                let mut pb = PathBuilder::new();
                pb.rect((bar_left + 6) as f32, oi_top as f32, 3., oi_height as f32);
                let path = pb.finish();
                dt.fill(
                    &path,
                    &Source::Solid(color_schema.open_interest.into()),
                    &DrawOptions::new(),
                );
            }
        }

        if max_oi > Decimal::ZERO {
            let oi_height = (max_oi / Decimal::from(100) / oi_diff * vh_dec)
                .to_i32()
                .unwrap_or(0);
            let oi_top = (self.area.top + self.area.height) - oi_height;
            let mut pb = PathBuilder::new();
            pb.rect(
                (self.area.width - 3) as f32,
                oi_top as f32,
                2.,
                oi_height as f32,
            );
            let path = pb.finish();
            dt.fill(
                &path,
                &Source::Solid(color_schema.scale_bar.into()),
                &DrawOptions::new(),
            );
        }

        // scale
        let visible_ticks = (self.area.height as f64) / px_per_tick.to_f64().unwrap();
        let mut m = 1;
        while visible_ticks / m.to_f64().unwrap() > 10.0 {
            m *= 10;
        }
        if visible_ticks / m.to_f64().unwrap() > 4. {
            m *= 2;
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
            &Source::Solid(color_schema.text_light.into()),
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
            &Source::Solid(color_schema.scale_bar.into()),
            &DrawOptions::new(),
        );

        for i in 0..3 {
            let tp = tick_price + m * Decimal::from(i);
            dt.draw_text(
                &font,
                (14 * 72 / 96) as f32,
                &tp.to_string(),
                Point::new((self.area.width - 50) as f32, price_to_y(tp) as f32),
                &Source::Solid(color_schema.text_light.into()),
                &DrawOptions::new(),
            );
        }

        for i in 0..3 {
            let tp = tick_price - m * Decimal::from(i);
            dt.draw_text(
                &font,
                (14 * 72 / 96) as f32,
                &tp.to_string(),
                Point::new((self.area.width - 50) as f32, price_to_y(tp) as f32),
                &Source::Solid(color_schema.text_light.into()),
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
            &Source::Solid(color_schema.crosshair.into()),
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
