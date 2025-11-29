use rodio::{OutputStream, Sink, Source, source::SineWave};
use std::time::Duration;

fn play_order_filled_sound() -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    // Pleasant ascending chime
    let frequencies = [523.25, 659.25, 783.99]; // C5, E5, G5

    for freq in frequencies.iter() {
        let tone = SineWave::new(*freq)
            .take_duration(Duration::from_millis(120))
            .amplify(0.18);
        sink.append(tone);

        // Small gap between notes
        let gap = SineWave::new(0.0).take_duration(Duration::from_millis(30));
        sink.append(gap);
    }

    sink.sleep_until_end();
    Ok(())
}

pub fn play_order_filled_async() {
    std::thread::spawn(|| {
        if let Err(e) = play_order_filled_sound() {
            eprintln!("Sound error: {}", e);
        }
    });
}
