mod exchanges;
mod graphics;
mod models;
mod trader;

use crate::exchanges::ExchangeFactory;
use crate::models::{Log, LogLevel, Orders};
use crate::trader::Trader;
use console::Term;
use graphics::{
    CandlesRenderer, OrderBookRenderer, OrderFlowRenderer, StatusRenderer, TextRenderer,
};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use models::{ColorSchema, Config, Interval, Layout, LogManager, PxPerTick};
use raqote::DrawTarget;
use rust_decimal::Decimal;
use std::sync::mpsc;

fn main() {
    let (logs_sender, logs_receiver) = mpsc::channel();
    let (orders_sender, orders_receiver) = mpsc::channel();

    let mut logs_manager = LogManager::new(logs_receiver, Term::stdout());

    let config = Config::load().unwrap_or_else(|err| {
        logs_manager.log_error(&format!("Error loading config: {}", err));
        std::process::exit(1);
    });

    let mut interval = Interval::M1;
    let mut exchange = ExchangeFactory::create(
        config.exchange.as_str(),
        config.symbol.clone(),
        200,
        &config,
        logs_sender.clone(),
        orders_sender,
    )
    .unwrap_or_else(|err| {
        logs_manager.log_error(&format!("Error creating exchange: {}", err));
        std::process::exit(1);
    });

    let (symbol, shared_state) = exchange.start(interval).unwrap_or_else(|err| {
        logs_manager.log_error(&format!("Error starting streams: {}", err));
        std::process::exit(1);
    });

    let mut size_range = Decimal::ZERO;

    let mut window_width = config.window_width;
    let mut window_height = config.window_height;

    let mut window = Window::new(
        &format!("{} - {}", symbol.slug, exchange.name()),
        window_width,
        window_height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();
    window.set_target_fps(60);

    let mut dt = DrawTarget::new(window_width as i32, window_height as i32);
    let mut layout = Layout::new(window_width as i32, window_height as i32);
    let text_renderer =
        TextRenderer::new("/System/Library/Fonts/SFNSMono.ttf").unwrap_or_else(|err| {
            eprintln!("Error loading font: {}", err);
            std::process::exit(1);
        });
    let mut candles_renderer = CandlesRenderer::new(layout.candles_area);
    let mut order_book_renderer = OrderBookRenderer::new(layout.order_book_area);
    let mut order_flow_renderer = OrderFlowRenderer::new(layout.order_flow_area);
    let mut status_renderer = StatusRenderer::new(layout.status_area);

    let mut trader = Trader::new(
        symbol.clone(),
        Orders::new(),
        [
            config.lot_mult_1.unwrap(),
            config.lot_mult_2.unwrap(),
            config.lot_mult_3.unwrap(),
            config.lot_mult_4.unwrap(),
        ],
        config.lot_size.unwrap(),
        config.sl_pnl,
    );

    let mut center: Option<Decimal> = None;
    let mut px_per_tick = PxPerTick::default();
    let mut force_redraw = true;
    let mut left_was_pressed = false;
    let mut sl_triggered = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        match orders_receiver.try_recv() {
            Ok(value) => {
                trader.consume_order(value);
                force_redraw = true;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {}
        };

        {
            let order_book = shared_state.order_book.read().unwrap();
            trader.set_bid_ask(order_book.bid(), order_book.ask());
        }

        logs_manager.consume();

        // pause recenter if ctrl is pressed
        if !(center.is_some()
            && (window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl)))
        {
            if trader.bid.is_some() && trader.ask.is_some() {
                let current_center = Some(
                    ((trader.bid.unwrap() + trader.ask.unwrap())
                        / Decimal::from(2)
                        / symbol.tick_size)
                        .floor()
                        * symbol.tick_size,
                );

                if center.is_some() {
                    if (center.unwrap() - current_center.unwrap()).abs() / symbol.tick_size
                        * px_per_tick.get()
                        >= Decimal::from(window_height / 4)
                    {
                        center = current_center;
                    }
                } else {
                    center = current_center;
                }
            }
        }

        let (new_width, new_height) = window.get_size();
        if new_width != window_width || new_height != window_height {
            window_width = new_width;
            window_height = new_height;

            dt = DrawTarget::new(window_width as i32, window_height as i32);
            layout = Layout::new(window_width as i32, window_height as i32);
            candles_renderer = CandlesRenderer::new(layout.candles_area);
            order_book_renderer = OrderBookRenderer::new(layout.order_book_area);
            order_flow_renderer = OrderFlowRenderer::new(layout.order_flow_area);
            status_renderer = StatusRenderer::new(layout.status_area);

            force_redraw = true;
        }

        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
            trader.set_size_multiplier_index(0);
        }

        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
            trader.set_size_multiplier_index(1);
        }

        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
            trader.set_size_multiplier_index(2);
        }

        if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) {
            trader.set_size_multiplier_index(3);
        }

        if window.is_key_pressed(Key::Equal, minifb::KeyRepeat::No) {
            if let Some(new_order) = trader.market_buy() {
                exchange.place_order(new_order);
            }
        }

        if window.is_key_pressed(Key::Minus, minifb::KeyRepeat::No) {
            if let Some(new_order) = trader.market_sell() {
                exchange.place_order(new_order);
            }
        }

        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) {
            if let Some(new_order) = trader.flat() {
                exchange.place_order(new_order);
            }
        }

        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            if let Some(new_order) = trader.reverse() {
                exchange.place_order(new_order);
            }
        }

        if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No)
            && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift))
        {
            px_per_tick.scale_out();
            size_range = Decimal::ZERO;
            force_redraw = true;
        }

        if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No)
            && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift))
        {
            px_per_tick.scale_in();
            size_range = Decimal::ZERO;
            force_redraw = true;
        }

        if window.is_key_pressed(Key::Right, minifb::KeyRepeat::No)
            && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift))
        {
            let new_interval = interval.up();
            if new_interval != interval {
                interval = new_interval;
                exchange.set_interval(new_interval);
                center = None;
                force_redraw = true;
            }
        }

        if window.is_key_pressed(Key::Left, minifb::KeyRepeat::No)
            && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift))
        {
            let new_interval = interval.down();
            if new_interval != interval {
                interval = new_interval;
                exchange.set_interval(new_interval);
                center = None;
                force_redraw = true;
            }
        }

        let left_pressed = window.get_mouse_down(MouseButton::Left);
        if left_pressed && !left_was_pressed {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
                if window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl) {
                    if let Some(center_price) = center {
                        let price = (Decimal::from(layout.candles_area.height / 2 - y as i32)
                            / px_per_tick.get())
                        .floor()
                            * symbol.tick_size
                            + center_price;
                        if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift)
                        {
                            if let Some(new_order) = trader.limit(price) {
                                exchange.place_order(new_order);
                            }
                        } else {
                            if let Some(new_order) = trader.stop(price) {
                                exchange.place_order(new_order);
                            }
                        };
                    }
                }
            }
        }
        left_was_pressed = left_pressed;

        if window.is_key_pressed(Key::C, minifb::KeyRepeat::No) {
            for o in trader.get_open_orders() {
                exchange.cancel_order(o.id.clone());
            }
        }

        if trader.bid.is_some() && !sl_triggered {
            if let Some(sl_pnl) = config.sl_pnl {
                if trader.get_pnl() < -sl_pnl.abs() {
                    sl_triggered = true;
                    trader.flat();
                    logs_sender
                        .send(Log::new(
                            LogLevel::Error("SL".to_string()),
                            format!("Stop-loss triggered at pnl: {:.2}", trader.get_pnl()),
                        ))
                        .unwrap()
                }
            }
        }

        let color_schema = ColorSchema::for_theme(config.theme);

        if let Some(center_price) = center {
            candles_renderer.render(
                shared_state.candles.read().unwrap(),
                shared_state.open_interest.read().unwrap(),
                &mut dt,
                &text_renderer,
                &color_schema,
                symbol.tick_size,
                center_price,
                px_per_tick.get(),
                trader.get_open_orders(),
                trader.get_last_closed_order(),
                trader.get_sl_price(),
                force_redraw,
            );
            order_book_renderer.render(
                shared_state.order_book.read().unwrap(),
                &mut dt,
                &color_schema,
                symbol.tick_size,
                center_price,
                px_per_tick.get(),
                &mut size_range,
                force_redraw,
            );
            order_flow_renderer.render(
                shared_state.order_flow.read().unwrap(),
                &mut dt,
                &color_schema,
                symbol.tick_size,
                center_price,
                px_per_tick.get(),
                &mut size_range,
                force_redraw,
            );
        }
        status_renderer.render(
            interval,
            &mut dt,
            &text_renderer,
            &color_schema,
            &trader,
            &logs_manager.status(),
        );

        let pixels_buffer: Vec<u32> = dt.get_data().iter().map(|&pixel| pixel).collect();
        window
            .update_with_buffer(&pixels_buffer, window_width, window_height)
            .unwrap();

        force_redraw = false;
    }

    exchange.stop()
}
