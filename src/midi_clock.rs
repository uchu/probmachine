pub struct MidiClockPll {
    // PI loop filter state
    phase_error_integral: f64,
    estimated_bpm: f64,
    estimated_tick_interval: f64,
    last_tick_time: f64,
    locked: bool,
    tick_count: u32,

    // PI coefficients (derived from bandwidth + damping)
    kp: f64,
    ki: f64,

    // Outlier detection
    outlier_streak: u32,
    outlier_streak_interval: f64,

    // Timeout
    time_since_last_tick: f64,
}

const PPQN: f64 = 24.0;
const MIN_BPM: f64 = 20.0;
const MAX_BPM: f64 = 300.0;
const LOCK_TICKS: u32 = 48; // 2 beats worth
const OUTLIER_THRESHOLD: f64 = 0.4; // 40% deviation
const FORCE_REACQUIRE_COUNT: u32 = 6;

impl MidiClockPll {
    pub fn new() -> Self {
        let bandwidth = 2.0;
        let damping = 0.707;
        let omega = 2.0 * std::f64::consts::PI * bandwidth;
        let kp = 2.0 * damping * omega;
        let ki = omega * omega;

        Self {
            phase_error_integral: 0.0,
            estimated_bpm: 120.0,
            estimated_tick_interval: 60.0 / (120.0 * PPQN),
            last_tick_time: -1.0,
            locked: false,
            tick_count: 0,
            kp,
            ki,
            outlier_streak: 0,
            outlier_streak_interval: 0.0,
            time_since_last_tick: 0.0,
        }
    }

    pub fn process_tick(&mut self, timestamp_seconds: f64) {
        if self.last_tick_time < 0.0 {
            self.last_tick_time = timestamp_seconds;
            self.tick_count = 1;
            return;
        }

        let interval = timestamp_seconds - self.last_tick_time;
        if interval <= 0.0 {
            return;
        }

        let deviation = (interval - self.estimated_tick_interval).abs() / self.estimated_tick_interval;

        if deviation > OUTLIER_THRESHOLD {
            self.outlier_streak += 1;
            // Track what the outliers agree on
            if self.outlier_streak == 1 {
                self.outlier_streak_interval = interval;
            } else {
                self.outlier_streak_interval = self.outlier_streak_interval * 0.7 + interval * 0.3;
            }

            if self.outlier_streak >= FORCE_REACQUIRE_COUNT {
                // Step change detected — force re-acquire
                self.estimated_tick_interval = self.outlier_streak_interval;
                self.estimated_bpm = 60.0 / (self.estimated_tick_interval * PPQN);
                self.phase_error_integral = 0.0;
                self.outlier_streak = 0;
                self.tick_count = LOCK_TICKS; // consider locked immediately
                self.locked = true;
            }

            self.last_tick_time = timestamp_seconds;
            self.time_since_last_tick = 0.0;
            return;
        }

        self.outlier_streak = 0;

        let error = interval - self.estimated_tick_interval;
        self.phase_error_integral += error * interval;
        let correction = self.kp * error + self.ki * self.phase_error_integral;
        self.estimated_tick_interval += correction;

        // Clamp to valid BPM range
        let min_interval = 60.0 / (MAX_BPM * PPQN);
        let max_interval = 60.0 / (MIN_BPM * PPQN);
        self.estimated_tick_interval = self.estimated_tick_interval.clamp(min_interval, max_interval);

        self.estimated_bpm = 60.0 / (self.estimated_tick_interval * PPQN);

        self.last_tick_time = timestamp_seconds;
        self.time_since_last_tick = 0.0;
        self.tick_count = self.tick_count.saturating_add(1);

        if self.tick_count >= LOCK_TICKS {
            self.locked = true;
        }
    }

    pub fn advance_samples(&mut self, num_samples: u32, sample_rate: f32) {
        let dt = num_samples as f64 / sample_rate as f64;
        self.time_since_last_tick += dt;

        let timeout = self.estimated_tick_interval * 2.5;
        if self.time_since_last_tick > timeout && self.locked {
            self.locked = false;
            self.tick_count = 0;
            self.phase_error_integral = 0.0;
        }
    }

    pub fn bpm(&self) -> f64 {
        self.estimated_bpm
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }
}
