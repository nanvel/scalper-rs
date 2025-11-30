use rodio::{OutputStream, Sink, Source, source::SineWave};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Sound {
    OrderFilled,
    Warning,
}

impl Sound {
    pub fn play(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Sound::OrderFilled => self.play_frequencies(&[523.25, 659.25, 783.99]),
            Sound::Warning => self.play_frequencies(&[523.25, 659.25, 783.99]),
        }
    }

    fn play_frequencies(&self, frequencies: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

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
}
