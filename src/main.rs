mod binance;
mod graphics;
mod models;
mod use_cases;

use binance::api::load_symbol;
use graphics::{CandlesRenderer, DomRenderer, OrderFlowRenderer, StatusRenderer};
use minifb::{Key, Window, WindowOptions};
use models::{CandlesState, Config, DomState, Layout, OrderFlowState, PxPerTick};
use raqote::DrawTarget;
use rust_decimal::{Decimal, prelude::FromStr};
use std::env;
use std::sync::{Arc, RwLock};
use tokio::runtime;
use use_cases::listen_streams::listen_streams;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: Symbol argument is required");
        eprintln!("Usage: {} <SYMBOL>", args[0]);
        std::process::exit(1);
    }

    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let symbol_slug = &args[1];
    let symbol = rt
        .block_on(load_symbol(&symbol_slug))
        .unwrap_or_else(|err| {
            eprintln!("Error loading symbol {}: {}", symbol_slug, err);
            std::process::exit(1);
        });

    let config = Config::default();

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

    let interval = "5m";
    let candles_limit = 100;
    let shared_candles_state = Arc::new(RwLock::new(CandlesState::new(candles_limit)));
    let shared_dom_state = Arc::new(RwLock::new(DomState::new(symbol.tick_size)));
    let shared_order_flow_state = Arc::new(RwLock::new(OrderFlowState::new()));

    listen_streams(
        shared_candles_state.clone(),
        shared_dom_state.clone(),
        shared_order_flow_state.clone(),
        symbol.slug.to_string(),
        interval.to_string(),
        candles_limit,
        500,
    );

    let mut dt = DrawTarget::new(window_width as i32, window_height as i32);
    let mut layout = Layout::new(window_width as i32, window_height as i32, &config);
    let mut candles_renderer = CandlesRenderer::new(layout.candles_area);
    let mut dom_renderer = DomRenderer::new(layout.dom_area);
    let mut order_flow_renderer = OrderFlowRenderer::new(layout.order_flow_area);
    let mut status_renderer = StatusRenderer::new(layout.status_area);

    window.set_target_fps(60);

    let mut center: Option<Decimal> = None;
    let mut px_per_tick = PxPerTick::new(
        config.px_per_tick_initial,
        config.px_per_tick_choices.clone(),
    );
    while window.is_open() && !window.is_key_down(Key::Escape) {
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

        if let (new_width, new_height) = window.get_size() {
            if new_width != window_width || new_height != window_height {
                window_width = new_width;
                window_height = new_height;

                dt = DrawTarget::new(window_width as i32, window_height as i32);
                layout = Layout::new(window_width as i32, window_height as i32, &config);
                candles_renderer = CandlesRenderer::new(layout.candles_area);
                dom_renderer = DomRenderer::new(layout.dom_area);
                order_flow_renderer = OrderFlowRenderer::new(layout.order_flow_area);
                status_renderer = StatusRenderer::new(layout.status_area);
            }
        }

        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            if scroll_y > 0.0 {
                px_per_tick.scale_in()
            } else if scroll_y < 0.0 {
                px_per_tick.scale_out()
            }
        }

        if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No)
            && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift))
        {
            px_per_tick.scale_in()
        }

        if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No)
            && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift))
        {
            px_per_tick.scale_out()
        }

        if let Some(center_price) = center {
            candles_renderer.render(
                shared_candles_state.read().unwrap(),
                &mut dt,
                &config,
                symbol.tick_size,
                center_price,
                px_per_tick.get(),
            );
            dom_renderer.render(
                shared_dom_state.read().unwrap(),
                &mut dt,
                &config,
                symbol.tick_size,
                center_price,
                px_per_tick.get(),
            );
            order_flow_renderer.render(
                shared_order_flow_state.read().unwrap(),
                &mut dt,
                &config,
                symbol.tick_size,
                center_price,
                px_per_tick.get(),
            );
        }
        status_renderer.render(&symbol.slug, interval, &mut dt, &config);

        let pixels_buffer: Vec<u32> = dt.get_data().iter().map(|&pixel| pixel).collect();
        window
            .update_with_buffer(&pixels_buffer, window_width, window_height)
            .unwrap();
    }
}
