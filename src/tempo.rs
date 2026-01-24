#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TempoSource {
    Manual,
    Host,
}

pub struct TempoTracker {
    manual_bpm: f64,
    host_bpm: Option<f64>,
    current_bpm: f64,
    source: TempoSource,
    host_is_playing: bool,
}

impl TempoTracker {
    pub fn new(_sample_rate: f64) -> Self {
        Self {
            manual_bpm: 120.0,
            host_bpm: None,
            current_bpm: 120.0,
            source: TempoSource::Manual,
            host_is_playing: false,
        }
    }

    pub fn set_manual_bpm(&mut self, bpm: f64) {
        self.manual_bpm = bpm.clamp(20.0, 300.0);
        self.update_current_bpm();
    }

    pub fn get_manual_bpm(&self) -> f64 {
        self.manual_bpm
    }

    pub fn set_host_tempo(&mut self, bpm: Option<f64>, is_playing: bool) {
        self.host_bpm = bpm;
        self.host_is_playing = is_playing;
        self.update_current_bpm();
    }

    fn update_current_bpm(&mut self) {
        if self.host_is_playing {
            if let Some(host_bpm) = self.host_bpm {
                if host_bpm > 0.0 {
                    self.source = TempoSource::Host;
                    self.current_bpm = host_bpm;
                    return;
                }
            }
        }

        self.source = TempoSource::Manual;
        self.current_bpm = self.manual_bpm;
    }

    pub fn get_bpm(&self) -> f64 {
        self.current_bpm
    }

    pub fn get_source(&self) -> TempoSource {
        self.source
    }

    pub fn is_locked(&self) -> bool {
        self.source != TempoSource::Manual
    }

    pub fn increment_bpm(&mut self) {
        if !self.is_locked() {
            self.set_manual_bpm(self.manual_bpm + 1.0);
        }
    }

    pub fn decrement_bpm(&mut self) {
        if !self.is_locked() {
            self.set_manual_bpm(self.manual_bpm - 1.0);
        }
    }
}

impl Default for TempoTracker {
    fn default() -> Self {
        Self::new(44100.0)
    }
}
