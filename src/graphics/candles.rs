use crate::models::{Area, CandlesState, Config, Scale};
use raqote::{
    DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, SolidSource, Source, StrokeStyle,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use std::cmp;
use std::ops::Mul;
use tokio::sync::RwLockReadGuard;

pub struct CandlesRenderer {
    area: Area,
    padding: i32,
}

impl CandlesRenderer {
    pub fn new(area: Area) -> Self {
        Self { area, padding: 10 }
    }

    pub fn render(
        &self,
        candles_state: RwLockReadGuard<CandlesState>,
        dt: &mut DrawTarget,
        config: &Config,
        scale: &mut Scale,
    ) {
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

        let price_range = max_price - min_price;
        let candle_width = cmp::min(
            (self.area.width - 2 * self.padding) / (candles.len() as i32),
            10,
        );
        let body_width = (candle_width as f32 * 0.7).max(1.0) as i32;

        let new_central_price = closest_significant(current_price, min_price, max_price);
        if scale.central_price.is_zero()
            || (scale.central_price - new_central_price).abs() > price_range / Decimal::from(4)
        {
            let price_to_points = price_range / Decimal::from(self.area.height - self.padding * 2);
            let price_to_points = closest_significant(
                price_to_points,
                price_to_points.mul(Decimal::from_f64(0.3).unwrap()),
                price_to_points.mul(Decimal::from(3)),
            );
            scale.price_to_points = price_to_points;
            scale.central_price = new_central_price;
            scale.central_point = (self.area.height / 2) + self.area.top;
        }

        let price_to_y = |price: Decimal| -> i32 {
            if price > scale.central_price {
                return scale.central_point
                    - ((price - scale.central_price) / scale.price_to_points)
                        .to_i32()
                        .unwrap_or(0);
            } else {
                return scale.central_point
                    + ((scale.central_price - price) / scale.price_to_points)
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

        let dot_color: SolidSource = if candles_state.online {
            config.online_color.into()
        } else {
            config.offline_color.into()
        };

        let mut pb = PathBuilder::new();
        let dot_radius = 5.0;
        let dot_x = self.area.left as f32 + 15.0;
        let dot_y = (self.area.top + self.area.height) as f32 - 15.0;
        pb.arc(dot_x, dot_y, dot_radius, 0.0, 2.0 * std::f32::consts::PI);
        let path = pb.finish();

        dt.fill(&path, &Source::Solid(dot_color), &DrawOptions::new());

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

        // border
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, self.area.height as f32);
        pb.line_to(self.area.width as f32, self.area.height as f32);
        pb.line_to(self.area.width as f32, 0.0);
        let path = pb.finish();

        dt.stroke(
            &path,
            &Source::Solid(config.border_color.into()),
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
