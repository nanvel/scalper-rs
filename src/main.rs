mod data;
mod graphics;
mod streams;

use data::Config;
use graphics::{CandlesRenderer, DomRenderer};
use minifb::{Key, Window, WindowOptions};
use raqote::DrawTarget;
use std::env;
use streams::{start_candles_stream, start_dom_stream};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: Symbol argument is required");
        eprintln!("Usage: {} <SYMBOL>", args[0]);
        std::process::exit(1);
    }

    let symbol = &args[1];
    let config = Config::default();

    let mut window_width = 800;
    let mut window_height = 600;

    let mut window = Window::new(
        "Scalper",
        window_width,
        window_height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let candles_state = start_candles_stream(symbol.to_string(), "5m".to_string(), 100).await;
    let dom_state = start_dom_stream(symbol.to_string(), 500).await;

    let mut dt = DrawTarget::new(window_width as i32, window_height as i32);
    let candles_renderer =
        CandlesRenderer::new(window_width as i32 - 100, window_height as i32, 0, 0);
    let dom_renderer = DomRenderer::new(100, window_height as i32, window_width as i32 - 100, 0);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let (new_width, new_height) = window.get_size() {
            if new_width != window_width || new_height != window_height {
                window_width = new_width;
                window_height = new_height;
                dt = DrawTarget::new(window_width as i32, window_height as i32);
            }
        }

        candles_renderer.render(candles_state.read().await, &mut dt, &config);
        dom_renderer.render(dom_state.read().await, &mut dt, &config);

        let pixels_buffer: Vec<u32> = dt.get_data().iter().map(|&pixel| pixel).collect();
        window
            .update_with_buffer(&pixels_buffer, window_width, window_height)
            .unwrap();
        sleep(Duration::from_millis(100)).await;
    }
}
