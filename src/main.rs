mod binance;
mod graphics;
mod models;

use binance::api::load_symbol;
use binance::streams::{start_candles_stream, start_dom_stream};
use graphics::{CandlesRenderer, DomRenderer, StatusRenderer};
use minifb::{Key, Window, WindowOptions};
use models::{Config, Layout, Scale};
use raqote::DrawTarget;
use std::env;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: Symbol argument is required");
        eprintln!("Usage: {} <SYMBOL>", args[0]);
        std::process::exit(1);
    }

    let symbol_slug = &args[1];
    let symbol = load_symbol(&symbol_slug).await.unwrap_or_else(|err| {
        eprintln!("Error loading symbol {}: {}", symbol_slug, err);
        std::process::exit(1);
    });
    let mut tick_size = symbol.tick_size;

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

    let candles_state = start_candles_stream(symbol.slug.to_string(), "5m".to_string(), 100).await;
    let dom_state = start_dom_stream(symbol.slug.to_string(), 500).await;

    let mut dt = DrawTarget::new(window_width as i32, window_height as i32);
    let mut layout = Layout::new(window_width as i32, window_height as i32, &config);
    let mut scale = Scale::default();
    let mut candles_renderer = CandlesRenderer::new(layout.candles_area);
    let mut dom_renderer = DomRenderer::new(layout.dom_area);
    let mut status_renderer = StatusRenderer::new(layout.status_area);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let (new_width, new_height) = window.get_size() {
            if new_width != window_width || new_height != window_height {
                window_width = new_width;
                window_height = new_height;

                dt = DrawTarget::new(window_width as i32, window_height as i32);
                layout = Layout::new(window_width as i32, window_height as i32, &config);
                scale = Scale::default();
                candles_renderer = CandlesRenderer::new(layout.candles_area);
                dom_renderer = DomRenderer::new(layout.dom_area);
                status_renderer = StatusRenderer::new(layout.status_area);
            }
        }

        candles_renderer.render(candles_state.read().unwrap(), &mut dt, &config, &mut scale);
        dom_renderer.render(dom_state.read().unwrap(), &mut dt, &config, &scale);
        status_renderer.render(&symbol.slug, &mut dt, &config);

        let pixels_buffer: Vec<u32> = dt.get_data().iter().map(|&pixel| pixel).collect();
        window
            .update_with_buffer(&pixels_buffer, window_width, window_height)
            .unwrap();
        sleep(Duration::from_millis(100)).await;
    }
}
