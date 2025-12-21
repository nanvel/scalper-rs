mod exchanges;
mod models;
mod renderer;
mod trader;
mod utils;

use crate::exchanges::ExchangeFactory;
use crate::models::{Log, LogLevel, Orders, Sound};
use crate::renderer::Renderer;
use crate::trader::Trader;
use crate::utils::{allow_sleep, prevent_sleep};
use console::Term;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use models::{ColorSchema, Config, Interval, LogManager};
use rust_decimal::Decimal;
use std::sync::mpsc;

fn main() {
    let (logs_sender, logs_receiver) = mpsc::channel();
    let (orders_sender, orders_receiver) = mpsc::channel();

    let mut logs_manager = LogManager::new(logs_receiver, Term::stdout(), false);

    let config = Config::load().unwrap_or_else(|err| {
        logs_manager.log_error(&format!("Error loading config: {}", err));
        std::process::exit(1);
    });

    logs_manager.set_with_sound(config.sound);

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

    let mut window = Window::new(
        &format!("{} - {}", symbol.slug, exchange.name()),
        config.window_width,
        config.window_height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();
    window.set_target_fps(60);

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

    let font = SystemSource::new()
        .select_best_match(&[FamilyName::Monospace], &Properties::new())
        .unwrap()
        .load()
        .unwrap();
    let mut renderer = Renderer::new(
        config.window_width,
        config.window_height,
        symbol.tick_size,
        ColorSchema::for_theme(config.theme),
        font,
    );

    prevent_sleep();

    let mut force_redraw = true;
    let mut left_was_pressed = false;
    let mut sl_triggered = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        match orders_receiver.try_recv() {
            Ok(value) => {
                let order_str = value.to_string();
                let filled = trader.consume_order(value);
                if filled {
                    logs_sender
                        .send(Log::new(
                            LogLevel::Info,
                            order_str,
                            Some(Sound::OrderFilled),
                        ))
                        .ok();
                }
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

        let ctrl_pressed = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
        let shift_pressed =
            window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);

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

        if window.is_key_pressed(Key::N, minifb::KeyRepeat::No) {
            shared_state.order_flow.write().unwrap().reset();
        }

        if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No) && shift_pressed {
            renderer.scale_out();
            force_redraw = true;
        }

        if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No) && shift_pressed {
            renderer.scale_in();
            force_redraw = true;
        }

        if window.is_key_pressed(Key::Right, minifb::KeyRepeat::No) && shift_pressed {
            let new_interval = interval.up();
            if new_interval != interval {
                interval = new_interval;
                exchange.set_interval(new_interval);
                force_redraw = true;
            }
        }

        if window.is_key_pressed(Key::Left, minifb::KeyRepeat::No) && shift_pressed {
            let new_interval = interval.down();
            if new_interval != interval {
                interval = new_interval;
                exchange.set_interval(new_interval);
                force_redraw = true;
            }
        }

        let left_pressed = window.get_mouse_down(MouseButton::Left);
        if left_pressed && !left_was_pressed && ctrl_pressed {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
                let price = renderer.px_to_price(y as i32);
                if price > Decimal::ZERO {
                    if shift_pressed {
                        if let Some(new_order) = trader.stop(price) {
                            exchange.place_order(new_order);
                        }
                    } else {
                        if let Some(new_order) = trader.limit(price) {
                            exchange.place_order(new_order);
                        }
                    };
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
                            None,
                        ))
                        .unwrap()
                }
            }
        }

        let (window_width, window_height) = window.get_size();
        renderer.set_size(window_width, window_height);
        renderer.render(
            &shared_state,
            &trader,
            logs_manager.status(),
            interval,
            ctrl_pressed,
            force_redraw,
        );

        let pixels_buffer: Vec<u32> = renderer.to_pixes_buffer();
        window
            .update_with_buffer(&pixels_buffer, window_width, window_height)
            .unwrap();

        force_redraw = false;
    }

    exchange.stop();

    allow_sleep();
}
