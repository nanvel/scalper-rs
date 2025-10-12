mod data;
mod graphics;
mod streams;

use graphics::CandlesRenderer;
use minifb::{Key, Window, WindowOptions};
use streams::start_candles_stream;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let window_width = 800;
    let window_height = 600;

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

    let candles_store = start_candles_stream("MYROUSDT".to_string(), "5m".to_string(), 100).await;
    let renderer = CandlesRenderer::new(window_width as i32, window_height as i32);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // if let (new_width, new_height) = window.get_size() {
        //     if new_width != window_width || new_height != window_height {
        //         window_width = new_width;
        //         window_height = new_height;
        //     }
        // }

        let dt = renderer.render(&candles_store.read().await.to_vec());
        let buffer: Vec<u32> = dt.get_data().iter().map(|&pixel| pixel).collect();
        window
            .update_with_buffer(&buffer, window_width, window_height)
            .unwrap();
        sleep(Duration::from_millis(100)).await;
    }
}
