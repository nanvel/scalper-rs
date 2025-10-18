mod binance;
mod graphics;
mod models;
mod use_cases;

use binance::api::load_symbol;
use graphics::{CandlesRenderer, DomRenderer, StatusRenderer};
use minifb::{Key, Window, WindowOptions};
use models::{CandlesState, DomState};
use models::{Config, Layout};
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

    let candles_limit = 100;
    let shared_candles_state = Arc::new(RwLock::new(CandlesState::new(candles_limit)));
    let shared_dom_state = Arc::new(RwLock::new(DomState::new(symbol.tick_size)));

    listen_streams(
        shared_candles_state.clone(),
        shared_dom_state.clone(),
        symbol.slug.to_string(),
        "5m".to_string(),
        candles_limit,
        500,
    );

    let mut dt = DrawTarget::new(window_width as i32, window_height as i32);
    let mut layout = Layout::new(window_width as i32, window_height as i32, &config);
    let mut candles_renderer = CandlesRenderer::new(layout.candles_area);
    let mut dom_renderer = DomRenderer::new(layout.dom_area);
    let mut status_renderer = StatusRenderer::new(layout.status_area);

    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let center = shared_dom_state.read().unwrap().center();
        let px_per_tick = Decimal::from_str("0.1").unwrap();

        if let (new_width, new_height) = window.get_size() {
            if new_width != window_width || new_height != window_height {
                window_width = new_width;
                window_height = new_height;

                dt = DrawTarget::new(window_width as i32, window_height as i32);
                layout = Layout::new(window_width as i32, window_height as i32, &config);
                candles_renderer = CandlesRenderer::new(layout.candles_area);
                dom_renderer = DomRenderer::new(layout.dom_area);
                status_renderer = StatusRenderer::new(layout.status_area);
            }
        }

        if let Some(center_price) = center {
            candles_renderer.render(
                shared_candles_state.read().unwrap(),
                &mut dt,
                &config,
                symbol.tick_size,
                center_price,
                px_per_tick,
            );
            dom_renderer.render(
                shared_dom_state.read().unwrap(),
                &mut dt,
                &config,
                symbol.tick_size,
                center_price,
                px_per_tick,
            );
        }
        status_renderer.render(&symbol.slug, &mut dt, &config);

        let pixels_buffer: Vec<u32> = dt.get_data().iter().map(|&pixel| pixel).collect();
        window
            .update_with_buffer(&pixels_buffer, window_width, window_height)
            .unwrap();
    }
}
