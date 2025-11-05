mod binance;
mod exchanges;
mod graphics;
mod models;
mod use_cases;

use binance::BinanceClient;
use graphics::{CandlesRenderer, DomRenderer, OrderFlowRenderer, StatusRenderer, TextRenderer};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use models::{
    CandlesState, ColorSchema, Config, DomState, Interval, Layout, MessageManager,
    OpenInterestState, OrderFlowState, PxPerTick, Trader,
};
use raqote::DrawTarget;
use rust_decimal::{Decimal, prelude::FromStr};
use std::env;
use std::sync::{Arc, RwLock, mpsc};
use tokio::runtime;
use use_cases::listen_open_interest::listen_open_interest;
use use_cases::listen_orders::listen_orders;
use use_cases::listen_streams::listen_streams;

fn restart_streams(
    handle: std::thread::JoinHandle<()>,
    stop_tx: tokio::sync::oneshot::Sender<()>,
    shared_candles_state: Arc<RwLock<CandlesState>>,
    shared_dom_state: Arc<RwLock<DomState>>,
    shared_order_flow_state: Arc<RwLock<OrderFlowState>>,
    symbol_slug: String,
    interval: Interval,
    candles_limit: usize,
) -> (
    std::thread::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
) {
    // best-effort shutdown of previous listener
    let _ = stop_tx.send(());
    let _ = handle.join();

    shared_candles_state.write().unwrap().clear();
    shared_dom_state.write().unwrap().clear();

    listen_streams(
        shared_candles_state,
        shared_dom_state,
        shared_order_flow_state,
        symbol_slug,
        interval,
        candles_limit,
        500,
    )
}

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

    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let client = BinanceClient::new(
        config.binance_access_key.clone(),
        config.binance_secret_key.clone(),
    );
    let mut trader = Trader::new();

    let symbol_slug = &args[1];
    let symbol = rt
        .block_on(client.get_symbol(&symbol_slug))
        .unwrap_or_else(|err| {
            eprintln!("Error loading symbol {}: {}", symbol_slug, err);
            std::process::exit(1);
        });
    let ticker_price = rt
        .block_on(client.get_ticker_price(&symbol_slug))
        .unwrap_or_else(|err| {
            eprintln!("Error loading ticker price for {}: {}", symbol_slug, err);
            std::process::exit(1);
        });

    let size_base_1 = symbol.tune_quantity(config.size_1.unwrap() / ticker_price, ticker_price);
    let size_base_2 = symbol.tune_quantity(config.size_2.unwrap() / ticker_price, ticker_price);
    let size_base_3 = symbol.tune_quantity(config.size_3.unwrap() / ticker_price, ticker_price);

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

    let (alert_sender, alert_receiver) = mpsc::channel();
    let (orders_sender, orders_receiver) = mpsc::channel();
    let mut alerts_manager = MessageManager::new(alert_receiver);

    let mut interval = Interval::M5;
    let candles_limit = 100;
    let shared_candles_state = Arc::new(RwLock::new(CandlesState::new(candles_limit)));
    let shared_dom_state = Arc::new(RwLock::new(DomState::new(symbol.tick_size)));
    let shared_order_flow_state = Arc::new(RwLock::new(OrderFlowState::new()));
    let shared_open_interest_state = Arc::new(RwLock::new(OpenInterestState::new()));

    let mut size_range = Decimal::ZERO;
    let mut size = config.size_1.unwrap();
    let mut size_base = size_base_1;

    let (mut handle, mut stop_tx) = listen_streams(
        shared_candles_state.clone(),
        shared_dom_state.clone(),
        shared_order_flow_state.clone(),
        symbol.slug.to_string(),
        interval,
        candles_limit,
        500,
    );

    listen_open_interest(shared_open_interest_state.clone(), symbol.slug.to_string());
    listen_orders(
        &config,
        symbol.slug.to_string(),
        alert_sender.clone(),
        orders_sender.clone(),
    );

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

    window.set_target_fps(60);

    let mut center: Option<Decimal> = None;
    let mut px_per_tick = PxPerTick::default();
    let mut force_redraw = true;
    let mut left_was_pressed = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        match orders_receiver.try_recv() {
            Ok(value) => {
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
            if let current_center = shared_dom_state.read().unwrap().center() {
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
            size_base = size_base_1;
        }

        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
            size = config.size_2.unwrap();
            size_base = size_base_2;
        }

        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
            size = config.size_3.unwrap();
            size_base = size_base_3;
        }

        if window.is_key_pressed(Key::Equal, minifb::KeyRepeat::No) {
            rt.block_on(trader.buy(&client, &symbol, size_base))
        }

        if window.is_key_pressed(Key::Minus, minifb::KeyRepeat::No) {
            rt.block_on(trader.sell(&client, &symbol, size_base))
        }

        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) {
            rt.block_on(trader.flat(&client, &symbol))
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
                shared_candles_state.write().unwrap().set_interval(interval);
                // Restart streams with new interval: stop previous, join, then start new
                let (new_handle, new_stop) = restart_streams(
                    handle,
                    stop_tx,
                    shared_candles_state.clone(),
                    shared_dom_state.clone(),
                    shared_order_flow_state.clone(),
                    symbol.slug.to_string(),
                    interval,
                    candles_limit,
                );
                handle = new_handle;
                stop_tx = new_stop;
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
                // Restart streams with new interval: stop previous, join, then start new
                let (new_handle, new_stop) = restart_streams(
                    handle,
                    stop_tx,
                    shared_candles_state.clone(),
                    shared_dom_state.clone(),
                    shared_order_flow_state.clone(),
                    symbol.slug.to_string(),
                    interval,
                    candles_limit,
                );
                handle = new_handle;
                stop_tx = new_stop;
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
                shared_candles_state.read().unwrap(),
                shared_open_interest_state.read().unwrap(),
                &mut dt,
                &text_renderer,
                &color_schema,
                symbol.tick_size,
                center_price,
                px_per_tick.get(),
                trader.orders(),
                force_redraw,
            );
            dom_renderer.render(
                shared_dom_state.read().unwrap(),
                &mut dt,
                &color_schema,
                symbol.tick_size,
                center_price,
                px_per_tick.get(),
                &mut size_range,
                force_redraw,
            );
            order_flow_renderer.render(
                shared_order_flow_state.read().unwrap(),
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
            let dom_state = shared_dom_state.read().unwrap();
            status_renderer.render(
                interval,
                size,
                &mut dt,
                &text_renderer,
                &color_schema,
                trader.pnl(dom_state.bid(), dom_state.ask()),
                trader.base_balance(),
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

    let _ = stop_tx.send(());
    let _ = handle.join();
}
