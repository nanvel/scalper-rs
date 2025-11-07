mod binance;
mod exchanges;
mod graphics;
mod models;
mod use_cases;

use crate::exchanges::ExchangeFactory;
use crate::models::Orders;
use graphics::{CandlesRenderer, DomRenderer, OrderFlowRenderer, StatusRenderer, TextRenderer};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use models::{ColorSchema, Config, Interval, Layout, MessageManager, PxPerTick};
use raqote::DrawTarget;
use rust_decimal::{Decimal, prelude::FromStr};
use std::env;
use std::sync::mpsc;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: Symbol argument is required");
        eprintln!("Usage: {} <SYMBOL>", args[0]);
        std::process::exit(1);
    }

    let config = Config::load().unwrap_or_else(|err| {
        eprintln!("Error loading config: {}", err);
        std::process::exit(1);
    });

    let mut interval = Interval::M1;
    let mut exchange = ExchangeFactory::create(
        "binance_usd_futures",
        args[1].clone(),
        interval,
        200,
        &config,
    )
    .unwrap_or_else(|err| {
        eprintln!("Error creating exchange: {}", err);
        std::process::exit(1);
    });

    let (symbol, shared_state, orders_receiver, messages_receiver) =
        exchange.start().unwrap_or_else(|err| {
            eprintln!("Error starting streams: {}", err);
            std::process::exit(1);
        });

    let mut alerts_manager = MessageManager::new(messages_receiver);

    let symbol_slug = &args[1];

    // let size_base_1 = symbol.tune_quantity(config.size_1.unwrap() / ticker_price, ticker_price);
    // let size_base_2 = symbol.tune_quantity(config.size_2.unwrap() / ticker_price, ticker_price);
    // let size_base_3 = symbol.tune_quantity(config.size_3.unwrap() / ticker_price, ticker_price);

    let mut window_width = 800;
    let mut window_height = 600;

    let mut window = Window::new(
        &format!("Scalper - {}", symbol.slug),
        window_width,
        window_height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let mut size_range = Decimal::ZERO;
    let mut size = config.size_1.unwrap();
    // let mut size_base = size_base_1;

    let mut dt = DrawTarget::new(window_width as i32, window_height as i32);
    let mut layout = Layout::new(window_width as i32, window_height as i32);
    let text_renderer =
        TextRenderer::new("/System/Library/Fonts/SFNSMono.ttf").unwrap_or_else(|err| {
            eprintln!("Error loading font: {}", err);
            std::process::exit(1);
        });
    let mut candles_renderer = CandlesRenderer::new(layout.candles_area);
    let mut dom_renderer = DomRenderer::new(layout.dom_area);
    let mut order_flow_renderer = OrderFlowRenderer::new(layout.order_flow_area);
    let mut status_renderer = StatusRenderer::new(layout.status_area);

    let mut orders = Orders::new();

    window.set_target_fps(60);

    let mut center: Option<Decimal> = None;
    let mut px_per_tick = PxPerTick::default();
    let mut force_redraw = true;
    let mut left_was_pressed = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        match orders_receiver.try_recv() {
            Ok(value) => {
                orders.on_order(value);
                dbg!("Received: {}", value);
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {}
        };

        alerts_manager.update();

        // pause recenter if ctrl is pressed
        if !center.is_some()
            || !(window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl))
        {
            let dom = shared_state.dom.read().unwrap();
            let best_bid = dom.bid();
            let best_ask = dom.ask();
            if best_bid.is_some() && best_ask.is_some() {
                let current_center = Some(
                    ((*best_bid + *best_ask) / Decimal::from(2) / symbol.tick_size).floor()
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

        if let (new_width, new_height) = window.get_size() {
            if new_width != window_width || new_height != window_height {
                window_width = new_width;
                window_height = new_height;

                dt = DrawTarget::new(window_width as i32, window_height as i32);
                layout = Layout::new(window_width as i32, window_height as i32);
                candles_renderer = CandlesRenderer::new(layout.candles_area);
                dom_renderer = DomRenderer::new(layout.dom_area);
                order_flow_renderer = OrderFlowRenderer::new(layout.order_flow_area);
                status_renderer = StatusRenderer::new(layout.status_area);

                force_redraw = true;
            }
        }

        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
            size = config.size_1.unwrap();
            // size_base = size_base_1;
        }

        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
            size = config.size_2.unwrap();
            // size_base = size_base_2;
        }

        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
            size = config.size_3.unwrap();
            // size_base = size_base_3;
        }

        if window.is_key_pressed(Key::Equal, minifb::KeyRepeat::No) {
            // rt.block_on(trader.buy(&client, &symbol, size_base))
        }

        if window.is_key_pressed(Key::Minus, minifb::KeyRepeat::No) {
            // rt.block_on(trader.sell(&client, &symbol, size_base))
        }

        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) {
            // rt.block_on(trader.flat(&client, &symbol))
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
                    dbg!(x, y);
                }
            }
        }
        left_was_pressed = left_pressed;

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
                orders.all(),
                force_redraw,
            );
            dom_renderer.render(
                shared_state.dom.read().unwrap(),
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
        {
            let dom_state = shared_state.dom.read().unwrap();
            status_renderer.render(
                interval,
                size,
                &mut dt,
                &text_renderer,
                &color_schema,
                orders.pnl(dom_state.bid(), dom_state.ask()),
                orders.base_balance(),
            );
        }
        let active_alerts = alerts_manager.get_active_alerts();
        if active_alerts.is_empty() {
            window.set_title(&format!("Scalper - {}", symbol.slug));
        } else {
            window.set_title(&active_alerts.iter().next().unwrap().message);
        }

        let pixels_buffer: Vec<u32> = dt.get_data().iter().map(|&pixel| pixel).collect();
        window
            .update_with_buffer(&pixels_buffer, window_width, window_height)
            .unwrap();

        force_redraw = false;
    }

    exchange.stop()
}
