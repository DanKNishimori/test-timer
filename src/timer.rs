use rodio::{source::SineWave, OutputStream, Sink, Source};
use std::{
    thread,
    time::{Duration, Instant},
};

use crate::stages::Stages;

fn play_alarm() {
    thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let mut audio_well_played = false;

        if let Ok(f) = std::fs::File::open("audios/alarm-clock-ringing.mp3") {
            let file = std::io::BufReader::new(f);
            if let Ok(source) = rodio::Decoder::new(file) {
                sink.append(source);
                sink.sleep_until_end();
                audio_well_played = true;
            }
        }

        if !audio_well_played {
            eprintln!("Arquivo de áudio não encontrado.");
            sink.append(SineWave::new(440.0).amplify(0.20));
            thread::sleep(Duration::from_secs(2));
            sink.stop();
        }
    });
}

pub struct Timer {
    pub stages: Stages,
    pub is_paused: bool,
    start_time: Option<Instant>,
    elapsed_before_pause: Duration,
}

impl Timer {
    pub fn new(stages: Stages) -> Self {
        Self {
            stages,
            start_time: None,
            is_paused: false,
            elapsed_before_pause: Duration::ZERO,
        }
    }

    pub fn has_stared(&self) -> bool {
        self.start_time.is_some()
    }
    pub fn time_left(&self) -> Duration {
        match self.stages.get_current() {
            Some(s) => s.duration.saturating_sub(self.current_elapsed()),
            None => Duration::ZERO,
        }
    }

    pub fn toggle_play(&mut self) {
        if self.start_time.is_none() {
            self.is_paused = false;
        } else if !self.is_paused {
            self.elapsed_before_pause = self.current_elapsed();
            self.start_time = None;
            return;
        } else {
            self.is_paused = !self.is_paused;
        }
        self.start_time = Some(Instant::now() - self.elapsed_before_pause);
    }

    pub fn check_progress(&mut self, loop_enabled: bool) {
        if self.time_left().is_zero() {
            play_alarm();

            thread::sleep(Duration::from_secs(1));

            if self.stages.peek_next().is_some() {
                self.stages.next();
            } else if loop_enabled {
                self.stages.reset();
            } else {
                self.reset(true);
                return;
            }
            self.start_time = Some(Instant::now());
        }
    }

    pub fn reset(&mut self, reset_all: bool) {
        self.start_time = None;
        self.is_paused = false;
        self.elapsed_before_pause = Duration::ZERO;

        if reset_all {
            self.stages.reset();
        }
    }

    fn current_elapsed(&self) -> Duration {
        if self.is_paused {
            return self.elapsed_before_pause;
        }
        match self.start_time {
            Some(time) => time.elapsed(),
            None => self.elapsed_before_pause,
        }
    }
}
