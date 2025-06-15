use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

pub struct SoundManager {
    _stream: OutputStream,
    sink: Arc<Mutex<Sink>>,
}

impl SoundManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        sink.set_volume(0.3);

        Ok(Self {
            _stream: stream,
            sink: Arc::new(Mutex::new(sink)),
        })
    }

    pub fn play_move(&self) {
        // Simple click sound for regular moves
        if let Ok(sink) = self.sink.lock() {
            let source = SineWave::new(440.0)
                .take_duration(Duration::from_millis(50))
                .amplify(0.2);
            sink.append(source);
        }
    }

    pub fn play_capture(&self) {
        // Two-tone sound for captures
        if let Ok(sink) = self.sink.lock() {
            let source1 = SineWave::new(523.0)
                .take_duration(Duration::from_millis(100))
                .amplify(0.3);
            let source2 = SineWave::new(392.0)
                .take_duration(Duration::from_millis(100))
                .amplify(0.3);
            sink.append(source1);
            sink.append(source2);
        }
    }

    pub fn play_check(&self) {
        // Alert sound for check
        if let Ok(sink) = self.sink.lock() {
            let source = SineWave::new(880.0)
                .take_duration(Duration::from_millis(200))
                .amplify(0.4);
            sink.append(source);
        }
    }

    pub fn play_game_over(&self) {
        // Victory/defeat sound
        if let Ok(sink) = self.sink.lock() {
            // Descending tones for game over
            let source1 = SineWave::new(523.0)
                .take_duration(Duration::from_millis(150))
                .amplify(0.3);
            let source2 = SineWave::new(392.0)
                .take_duration(Duration::from_millis(150))
                .amplify(0.3);
            let source3 = SineWave::new(261.0)
                .take_duration(Duration::from_millis(300))
                .amplify(0.3);
            sink.append(source1);
            sink.append(source2);
            sink.append(source3);
        }
    }
}
