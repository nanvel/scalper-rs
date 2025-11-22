use super::text::TextRenderer;
use crate::models::{Area, CandlesState, ColorSchema, Order, OrderSide, Timestamp};
use f64_fixed::to_fixed_string;
use raqote::{DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, Source, StrokeStyle};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::sync::RwLockReadGuard;

pub struct OrdersRenderer {
    area: Area,
    padding: i32,
    last_updated: Option<Timestamp>,
}

impl OrdersRenderer {
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
        text_renderer: &TextRenderer,
        color_schema: &ColorSchema,
        tick_size: Decimal,
        center: Decimal,
        px_per_tick: Decimal,
        open_orders: Vec<&Order>,
        last_closed_order: Option<&Order>,
        sl_price: Option<Decimal>,
        force_redraw: bool,
    ) {
        if !force_redraw {
            if let Some(last_updated) = self.last_updated {
                if last_updated == candles_state.updated {
                    return;
                }
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

        let mut pb = PathBuilder::new();
        pb.rect(
            self.area.left as f32,
            self.area.top as f32,
            2.0,
            (self.area.height) as f32,
        );
        let path = pb.finish();

        dt.fill(
            &path,
            &Source::Solid(color_schema.border.into()),
            &DrawOptions::new(),
        );

        // Guard: tick_size must be non-zero to map prices to pixels
        if tick_size.is_zero() || candles_state.is_empty() {
            return;
        }

        let current_price = candles_state.last().unwrap().close;
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

        text_renderer.draw(
            dt,
            &to_fixed_string(tick_price.to_f64().unwrap(), 8),
            self.area.left + 5,
            price_to_y(tick_price) + 4,
            14,
            color_schema.text_light,
        );

        let mut pb = PathBuilder::new();
        let delta = tick_price / Decimal::from(100);
        pb.rect(
            self.area.left as f32,
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

        for i in 1..3 {
            let tp = tick_price + m * Decimal::from(i);
            text_renderer.draw(
                dt,
                &to_fixed_string(tp.to_f64().unwrap(), 8),
                self.area.left + 5,
                price_to_y(tp) + 4,
                14,
                color_schema.text_light,
            );
        }

        for i in 1..3 {
            let tp = tick_price - m * Decimal::from(i);
            text_renderer.draw(
                dt,
                &to_fixed_string(tp.to_f64().unwrap(), 8),
                self.area.left + 5,
                price_to_y(tp) + 4,
                14,
                color_schema.text_light,
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

        text_renderer.draw(
            dt,
            &to_fixed_string(current_price.to_f64().unwrap(), 8),
            self.area.left + 5,
            price_to_y(current_price) - 2,
            14,
            color_schema.text_light,
        );

        // orders
        for order in open_orders {
            let color = match order.order_side {
                OrderSide::Buy => color_schema.volume_buy,
                OrderSide::Sell => color_schema.volume_sell,
            };

            let y = price_to_y(order.price);
            let mut pb = PathBuilder::new();
            pb.move_to((self.area.left + self.area.width - 3) as f32, y as f32);
            pb.line_to(
                ((self.area.left + self.area.width) - 10) as f32,
                (y - 4) as f32,
            );
            pb.line_to(
                ((self.area.left + self.area.width) - 10) as f32,
                (y + 4) as f32,
            );
            pb.close();
            let path = pb.finish();
            let stroke_style = StrokeStyle {
                width: 1.0,
                ..Default::default()
            };
            dt.stroke(
                &path,
                &Source::Solid(color.into()),
                &stroke_style,
                &DrawOptions::new(),
            );
        }

        if let Some(order) = last_closed_order {
            // solid triangle
            let color = match order.order_side {
                OrderSide::Buy => color_schema.volume_buy,
                OrderSide::Sell => color_schema.volume_sell,
            };

            let y = price_to_y(order.average_price);
            let mut pb = PathBuilder::new();
            pb.move_to((self.area.left + self.area.width - 3) as f32, y as f32);
            pb.line_to(
                ((self.area.left + self.area.width) - 10) as f32,
                (y - 4) as f32,
            );
            pb.line_to(
                ((self.area.left + self.area.width) - 10) as f32,
                (y + 4) as f32,
            );
            pb.close();
            let path = pb.finish();
            dt.fill(&path, &Source::Solid(color.into()), &DrawOptions::new());
        }

        if let Some(sl_price) = sl_price {
            let y = price_to_y(sl_price);
            let mut pb = PathBuilder::new();
            pb.move_to(self.area.left as f32, y as f32);
            pb.line_to((self.area.left + self.area.width) as f32, y as f32);
            let path = pb.finish();

            dt.stroke(
                &path,
                &Source::Solid(color_schema.text_error.into()),
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
}
