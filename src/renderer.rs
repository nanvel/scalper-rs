use crate::models::{
    CandlesState, ColorSchema, Layout, OpenInterestState, OrderBookState, OrderFlowState,
    SharedState, Status, Timestamp,
};
use crate::trader::Trader;
use raqote::{
    DrawOptions, DrawTarget, LineCap, LineJoin, PathBuilder, SolidSource, Source, StrokeStyle,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromStr, ToPrimitive};

const PX_PER_TICK_CHOICES: [&str; 17] = [
    "0.01", "0.02", "0.05", "0.1", "0.2", "0.5", "1", "3", "5", "7", "9", "11", "13", "15", "17",
    "19", "21",
];

pub struct Renderer {
    dt: DrawTarget,
    layout: Layout,
    book_entry_range: Decimal,
    center_px: usize,
    center_price: Decimal,
    px_per_tick: Decimal,
    tick_size: Decimal,
    color_schema: ColorSchema,
    candles_updated: Timestamp,
    order_book_updated: Timestamp,
    order_flow_updated: Timestamp,
    force_redraw: bool,
}

impl Renderer {
    pub fn new(width: usize, height: usize, tick_size: Decimal, color_schema: ColorSchema) -> Self {
        Self {
            dt: DrawTarget::new(width as i32, height as i32),
            layout: Layout::new(width as i32, height as i32),
            book_entry_range: Decimal::ZERO,
            center_px: height / 2,
            center_price: Decimal::ZERO,
            px_per_tick: Decimal::from(1),
            tick_size,
            color_schema,
            candles_updated: Timestamp::from(0),
            order_book_updated: Timestamp::from(0),
            order_flow_updated: Timestamp::from(0),
            force_redraw: true,
        }
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        let width = width as i32;
        let height = height as i32;
        if self.layout.width != width || self.layout.height != height {
            self.layout = Layout::new(width, height);
            self.force_redraw = true;
            self.dt = DrawTarget::new(width, height);
            self.center_px = (height / 2) as usize;
            self.center_price = Decimal::ZERO;
        }
    }

    pub fn scale_in(&mut self) {
        if let Some(pos) = PX_PER_TICK_CHOICES
            .iter()
            .position(|&x| Decimal::from_str(x).unwrap() == self.px_per_tick)
        {
            if pos > 0 {
                self.px_per_tick = Decimal::from_str(PX_PER_TICK_CHOICES[pos - 1]).unwrap();
                self.book_entry_range = Decimal::ZERO;
            }
        }
    }

    pub fn scale_out(&mut self) {
        if let Some(pos) = PX_PER_TICK_CHOICES
            .iter()
            .position(|&x| Decimal::from_str(x).unwrap() == self.px_per_tick)
        {
            if pos + 1 < PX_PER_TICK_CHOICES.len() {
                self.px_per_tick = Decimal::from_str(PX_PER_TICK_CHOICES[pos + 1]).unwrap();
                self.book_entry_range = Decimal::ZERO;
            }
        }
    }

    pub fn price_to_px(&self, price: Decimal) -> i32 {
        (self.center_px as i32)
            + ((self.center_price - price) / self.tick_size * self.px_per_tick)
                .to_i32()
                .unwrap_or(0)
    }

    pub fn px_to_price(&self, px: i32) -> Decimal {
        self.center_price
            - Decimal::from(px - (self.center_px as i32)) / self.px_per_tick * self.tick_size
    }

    pub fn render(
        &mut self,
        shared_state: &SharedState,
        trader: &Trader,
        status: Status,
        locked: bool,
        force_redraw: bool,
    ) {
        if force_redraw {
            self.force_redraw = true;
        }

        let price;
        if let Some(last_candle) = shared_state.candles.read().unwrap().last() {
            price = last_candle.close;
        } else {
            return;
        }
        if self.center_price == Decimal::ZERO {
            self.center_price = price;
        }
        if !locked {
            self.adjust_center(price);
        }

        let candles_updated = shared_state.candles.read().unwrap().updated;
        let order_book_updated = shared_state.order_book.read().unwrap().updated;
        let order_flow_updated = shared_state.order_flow.read().unwrap().updated;

        if self.candles_updated != candles_updated || self.force_redraw {
            self.draw_candles(
                &shared_state.candles.read().unwrap(),
                &shared_state.open_interest.read().unwrap(),
            );
            self.candles_updated = candles_updated;
        }

        if self.order_book_updated != order_book_updated || self.force_redraw {
            self.draw_order_book(&shared_state.order_book.read().unwrap());
            self.order_book_updated = order_book_updated;
        }

        if self.order_flow_updated != order_flow_updated || self.force_redraw {
            self.draw_order_flow(&shared_state.order_flow.read().unwrap());
            self.order_flow_updated = order_flow_updated;
        }

        self.force_redraw = false;
    }

    fn draw_order_flow(&mut self, order_flow_state: &OrderFlowState) {
        let area = self.layout.order_flow_area;

        self.dt.fill_rect(
            area.left as f32,
            area.top as f32,
            area.width as f32,
            area.height as f32,
            &Source::Solid(self.color_schema.background.into()),
            &DrawOptions::new(),
        );

        let left = area.left as f32;
        let right = (area.left + area.width) as f32 - 10_f32;
        let max_width = Decimal::from((right - left) as u32);
        let central_point = area.height / 2;

        let mut buy_buckets: Vec<Decimal> = vec![Decimal::ZERO; area.height as usize];
        let mut sell_buckets: Vec<Decimal> = vec![Decimal::ZERO; area.height as usize];

        for (price, quantity) in order_flow_state.buys.iter() {
            let price_diff = (self.center_price - *price) / self.tick_size;
            let px_offset = (price_diff * self.px_per_tick).to_i32().unwrap_or(0);
            let y = central_point + px_offset;
            if y < 0 || y >= area.height {
                continue;
            }
            buy_buckets[y as usize] += *quantity;
        }

        for (price, quantity) in order_flow_state.sells.iter() {
            let price_diff = (self.center_price - *price) / self.tick_size;
            let px_offset = (price_diff * self.px_per_tick).to_i32().unwrap_or(0);
            let y = central_point + px_offset;
            if y < 0 || y >= area.height {
                continue;
            }
            sell_buckets[y as usize] += *quantity;
        }

        let max_val = buy_buckets
            .iter()
            .cloned()
            .max()
            .unwrap_or(Decimal::ZERO)
            .max(sell_buckets.iter().cloned().max().unwrap_or(Decimal::ZERO))
            .max(self.book_entry_range);

        if max_val.is_zero() {
            return;
        }

        self.book_entry_range = max_val;

        for (i, val) in buy_buckets.iter().enumerate() {
            if val.is_zero() {
                continue;
            }
            let width = (val / max_val * max_width).to_f32().unwrap_or(0.0);
            let y = i as f32 + area.top as f32;

            if self.px_per_tick == Decimal::from(3) {
                self.dt.fill_rect(
                    left,
                    y - 1.0,
                    width,
                    3.0,
                    &Source::Solid(self.color_schema.volume_buy.into()),
                    &DrawOptions::new(),
                );
            } else if self.px_per_tick >= Decimal::from(5) {
                self.dt.fill_rect(
                    left,
                    y - (self.px_per_tick.to_f32().unwrap() / 2.0).floor(),
                    width,
                    self.px_per_tick.to_f32().unwrap() - 2.0,
                    &Source::Solid(self.color_schema.volume_buy.into()),
                    &DrawOptions::new(),
                );
            } else {
                self.dt.fill_rect(
                    left,
                    y,
                    width,
                    1.0,
                    &Source::Solid(self.color_schema.volume_buy.into()),
                    &DrawOptions::new(),
                );
            }
        }

        for (i, val) in sell_buckets.iter().enumerate() {
            if val.is_zero() {
                continue;
            }
            let width = (val / max_val * max_width).to_f32().unwrap_or(0.0);
            let y = i as f32 + area.top as f32;

            if self.px_per_tick == Decimal::from(3) {
                self.dt.fill_rect(
                    left,
                    y - 1.0,
                    width,
                    3.0,
                    &Source::Solid(self.color_schema.volume_sell.into()),
                    &DrawOptions::new(),
                );
            } else if self.px_per_tick >= Decimal::from(5) {
                self.dt.fill_rect(
                    left,
                    y - (self.px_per_tick.to_f32().unwrap() / 2.0).floor(),
                    width,
                    self.px_per_tick.to_f32().unwrap() - 2.0,
                    &Source::Solid(self.color_schema.volume_sell.into()),
                    &DrawOptions::new(),
                );
            } else {
                self.dt.fill_rect(
                    left,
                    y,
                    width,
                    1.0,
                    &Source::Solid(self.color_schema.volume_sell.into()),
                    &DrawOptions::new(),
                );
            }
        }
    }

    fn draw_order_book(&mut self, order_book_state: &OrderBookState) {
        let area = self.layout.order_book_area;

        self.dt.fill_rect(
            area.left as f32,
            area.top as f32,
            area.width as f32,
            area.height as f32,
            &Source::Solid(self.color_schema.background.into()),
            &DrawOptions::new(),
        );

        let left = area.left as f32;
        let right = (area.left + area.width) as f32 - 10.0;
        let max_width = Decimal::from((right - left) as u32);

        let mut bid_buckets: Vec<Decimal> = vec![Decimal::ZERO; area.height as usize];
        let mut ask_buckets: Vec<Decimal> = vec![Decimal::ZERO; area.height as usize];

        for (price, quantity) in order_book_state.bids.iter() {
            let price_diff = (self.center_price - *price) / self.tick_size;
            let px_offset = (price_diff * self.px_per_tick).to_i32().unwrap_or(0);
            let y = self.center_px as i32 + px_offset;
            if y < 0 || y >= area.height {
                continue;
            }

            bid_buckets[y as usize] += *quantity;
        }

        for (price, quantity) in order_book_state.asks.iter() {
            let price_diff = (self.center_price - *price) / self.tick_size;
            let px_offset = (price_diff * self.px_per_tick).to_i32().unwrap_or(0);
            let y = self.center_px as i32 + px_offset;
            if y < 0 || y >= area.height {
                continue;
            }

            ask_buckets[y as usize] += *quantity;
        }

        let max_val = bid_buckets
            .iter()
            .cloned()
            .max()
            .unwrap_or(Decimal::ZERO)
            .max(ask_buckets.iter().cloned().max().unwrap_or(Decimal::ZERO))
            .max(self.book_entry_range);
        if max_val.is_zero() {
            return;
        }

        self.book_entry_range = max_val;

        for (i, val) in bid_buckets.iter().enumerate() {
            if val.is_zero() {
                continue;
            }

            let width = (val / max_val * max_width).to_f32().unwrap_or(0.0);
            let y = i as f32 + area.top as f32;

            if self.px_per_tick == Decimal::from(3) {
                self.dt.fill_rect(
                    left,
                    y - 1.0,
                    width,
                    3.0,
                    &Source::Solid(self.color_schema.bid_bar.into()),
                    &DrawOptions::new(),
                );
            } else if self.px_per_tick >= Decimal::from(5) {
                self.dt.fill_rect(
                    left,
                    y - (self.px_per_tick.to_f32().unwrap() / 2.0).floor(),
                    width,
                    self.px_per_tick.to_f32().unwrap() - 2.0,
                    &Source::Solid(self.color_schema.bid_bar.into()),
                    &DrawOptions::new(),
                );
            } else {
                self.dt.fill_rect(
                    left,
                    y,
                    width,
                    1.0,
                    &Source::Solid(self.color_schema.bid_bar.into()),
                    &DrawOptions::new(),
                );
            }
        }

        for (i, val) in ask_buckets.iter().enumerate() {
            if val.is_zero() {
                continue;
            }

            let width = (val / max_val * max_width).to_f32().unwrap_or(0.0);
            let y = i as f32 + area.top as f32;

            if self.px_per_tick == Decimal::from(3) {
                self.dt.fill_rect(
                    left,
                    y - 1.0,
                    width,
                    3.0,
                    &Source::Solid(self.color_schema.ask_bar.into()),
                    &DrawOptions::new(),
                );
            } else if self.px_per_tick >= Decimal::from(5) {
                self.dt.fill_rect(
                    left,
                    y - (self.px_per_tick.to_f32().unwrap() / 2.0).floor(),
                    width,
                    self.px_per_tick.to_f32().unwrap() - 2.0,
                    &Source::Solid(self.color_schema.ask_bar.into()),
                    &DrawOptions::new(),
                );
            } else {
                self.dt.fill_rect(
                    left,
                    y,
                    width,
                    1.0,
                    &Source::Solid(self.color_schema.ask_bar.into()),
                    &DrawOptions::new(),
                );
            }
        }
    }

    fn draw_candles(
        &mut self,
        candles_state: &CandlesState,
        open_interest_state: &OpenInterestState,
    ) {
        let area = self.layout.candles_area;
        self.dt.fill_rect(
            area.left as f32,
            area.top as f32,
            area.width as f32,
            area.height as f32,
            &Source::Solid(self.color_schema.background.into()),
            &DrawOptions::new(),
        );

        let candles = candles_state.to_vec();
        if candles.is_empty() {
            return;
        }

        let current_price = candles.last().unwrap().close;

        let candle_width = 15;
        let body_width = 11;
        let volume_height = 100;

        for (i, candle) in candles.iter().rev().enumerate() {
            let x = area.width + area.left - (i as i32) * candle_width - 15;

            let open_y = self.price_to_px(candle.open);
            let close_y = self.price_to_px(candle.close);
            let high_y = self.price_to_px(candle.high);
            let low_y = self.price_to_px(candle.low);

            let color: SolidSource = if candle.is_bullish() {
                self.color_schema.bullish_candle.into()
            } else {
                self.color_schema.bearish_candle.into()
            };

            let mut pb = PathBuilder::new();
            pb.move_to(x as f32, high_y as f32);
            pb.line_to(x as f32, low_y as f32);
            let path = pb.finish();

            self.dt.stroke(
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

            self.dt
                .fill(&path, &Source::Solid(color), &DrawOptions::new());
        }

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

        self.dt.fill_rect(
            area.left as f32,
            (area.height - volume_height - 5) as f32,
            area.width as f32,
            (volume_height + 5) as f32,
            &Source::Solid(self.color_schema.background.into()),
            &DrawOptions::new(),
        );

        self.dt.fill_rect(
            area.left as f32,
            (area.height - volume_height - 5) as f32,
            area.width as f32,
            1.,
            &Source::Solid(self.color_schema.border.into()),
            &DrawOptions::new(),
        );

        let vh_dec = Decimal::from(volume_height);
        if max_volume > Decimal::ZERO {
            for (i, candle) in candles.iter().rev().enumerate() {
                let x = area.width + area.left - (i as i32) * candle_width - 15;

                // Compute bar height proportional to volume
                let bar_height = ((candle.volume / max_volume) * vh_dec)
                    .to_i32()
                    .unwrap_or(0)
                    .max(1);

                let oi_height = if max_oi > Decimal::ZERO && !oi_diff.is_zero() {
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

                if oi_height <= 0 {
                    continue;
                }

                let bar_top = (area.top + area.height) - bar_height;
                let bar_left = x - (body_width / 2);

                let vol_color: SolidSource = if candle.is_bullish() {
                    self.color_schema.bullish_candle.into()
                } else {
                    self.color_schema.bearish_candle.into()
                };

                let mut pb = PathBuilder::new();
                pb.rect(bar_left as f32, bar_top as f32, 3., bar_height as f32);
                let path = pb.finish();
                self.dt
                    .fill(&path, &Source::Solid(vol_color), &DrawOptions::new());

                let oi_top = (area.top + area.height) - oi_height;
                let mut pb = PathBuilder::new();
                pb.rect((bar_left + 6) as f32, oi_top as f32, 3., oi_height as f32);
                let path = pb.finish();
                self.dt.fill(
                    &path,
                    &Source::Solid(self.color_schema.open_interest.into()),
                    &DrawOptions::new(),
                );
            }
        }

        if max_oi > Decimal::ZERO {
            if !oi_diff.is_zero() {
                let oi_height = (max_oi / Decimal::from(100) / oi_diff * vh_dec)
                    .to_i32()
                    .unwrap_or(0);
                let oi_top = (area.top + area.height) - oi_height;
                let mut pb = PathBuilder::new();
                pb.rect((area.width - 3) as f32, oi_top as f32, 2., oi_height as f32);
                let path = pb.finish();
                self.dt.fill(
                    &path,
                    &Source::Solid(self.color_schema.scale_bar.into()),
                    &DrawOptions::new(),
                );
            }
        }

        // current price line
        let mut pb = PathBuilder::new();
        pb.move_to(area.left as f32, self.price_to_px(current_price) as f32);
        pb.line_to(
            (area.left + area.width) as f32,
            self.price_to_px(current_price) as f32,
        );
        let path = pb.finish();

        self.dt.stroke(
            &path,
            &Source::Solid(self.color_schema.crosshair.into()),
            &StrokeStyle {
                width: 1.0,
                cap: LineCap::Round,
                join: LineJoin::Round,
                ..Default::default()
            },
            &DrawOptions::new(),
        );
    }

    fn adjust_center(&mut self, price: Decimal) {
        if (price - self.center_price).abs() / self.tick_size * self.px_per_tick
            >= Decimal::from(self.layout.height / 4)
        {
            self.center_price = price;
            self.center_px = (self.layout.height / 2) as usize;
        }
    }

    pub fn to_pixes_buffer(&self) -> Vec<u32> {
        self.dt.get_data().iter().map(|&pixel| pixel).collect()
    }
}
