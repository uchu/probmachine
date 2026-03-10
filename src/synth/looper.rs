const MAX_LOOP_SECONDS: f64 = 10.0;
const CROSSFADE_LEN: usize = 64;
const SINC_BETA: f64 = 7.857;

#[derive(Clone, Copy, PartialEq)]
pub enum LoopDirection {
    Forward,
    Reverse,
    PingPong,
}

impl LoopDirection {
    pub fn from_index(i: i32) -> Self {
        match i {
            1 => Self::Reverse,
            2 => Self::PingPong,
            _ => Self::Forward,
        }
    }
}

pub struct PitchedLooper {
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    buffer_length: usize,
    write_pos: usize,

    read_phase: f64,
    playing: bool,
    playback_elapsed: usize,

    prev_bar_index: u64,
    repeat_gain: f64,
    ping_forward: bool,
    record_frequency: f64,

    was_playing: bool,
    auto_recording: bool,
    auto_rec_samples_written: usize,

    mix_slew: f64,
    slew_coeff: f64,
    sample_rate: f64,

    doppler_envelope: f64,
    doppler_coef: f64,
}

impl PitchedLooper {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let max_samples = (sr * MAX_LOOP_SECONDS) as usize;
        let slew_coeff = 1.0 - (-1.0 / (sr * 0.005)).exp();

        Self {
            buffer_l: vec![0.0; max_samples],
            buffer_r: vec![0.0; max_samples],
            buffer_length: 0,
            write_pos: 0,
            read_phase: 0.0,
            playing: false,
            playback_elapsed: 0,
            prev_bar_index: u64::MAX,
            repeat_gain: 1.0,
            ping_forward: true,
            record_frequency: 440.0,
            was_playing: false,
            auto_recording: false,
            auto_rec_samples_written: 0,
            mix_slew: 0.0,
            slew_coeff,
            sample_rate: sr,
            doppler_envelope: 0.0,
            doppler_coef: (-1.0 / (sr * 0.3)).exp(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = sample_rate as f64;
        let max_samples = (sr * MAX_LOOP_SECONDS) as usize;
        self.buffer_l.resize(max_samples, 0.0);
        self.buffer_r.resize(max_samples, 0.0);
        self.reset();
        self.sample_rate = sr;
        self.slew_coeff = 1.0 - (-1.0 / (sr * 0.005)).exp();
        self.doppler_coef = (-1.0 / (sr * 0.3)).exp();
    }

    fn reset(&mut self) {
        self.buffer_length = 0;
        self.write_pos = 0;
        self.playing = false;
        self.auto_recording = false;
        self.auto_rec_samples_written = 0;
        self.prev_bar_index = u64::MAX;
        self.repeat_gain = 1.0;
        self.mix_slew = 0.0;
        self.doppler_envelope = 0.0;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        input_l: &[f32],
        input_r: &[f32],
        enabled: bool,
        pitch_semitones: f64,
        length_beats: f64,
        start_offset: f64,
        direction: LoopDirection,
        mix: f64,
        decay: f64,
        stutter: i32,
        key_track: bool,
        freeze: bool,
        bar_index: u64,
        bpm: f64,
        current_frequency: f64,
        seq_playing: bool,
        auto_rec_beats: f64,
        auto_rec_interval_index: i32,
        doppler: f64,
    ) {
        if !enabled {
            self.mix_slew = 0.0;
            return;
        }

        if !seq_playing && self.was_playing {
            self.playing = false;
            self.auto_recording = false;
            self.mix_slew = 0.0;
        }
        self.was_playing = seq_playing;

        let beat_samples = 60.0 / bpm.max(20.0) * self.sample_rate;

        if seq_playing && bar_index != self.prev_bar_index {
            let interval_bars = Self::auto_rec_interval_bars(auto_rec_interval_index);
            if bar_index % interval_bars == 0 {
                self.auto_recording = true;
                self.auto_rec_samples_written = 0;
                self.write_pos = 0;
                self.buffer_length = 0;
                self.playing = false;
                self.record_frequency = current_frequency.max(20.0);
            }
        }

        if self.auto_recording {
            let target_samples = (auto_rec_beats * beat_samples) as usize;
            if self.auto_rec_samples_written >= target_samples || self.write_pos >= self.buffer_l.len() {
                self.auto_recording = false;
                if self.write_pos > CROSSFADE_LEN {
                    self.buffer_length = self.write_pos;
                    self.apply_record_crossfade();
                    self.repeat_gain = 1.0;
                    self.trigger_playback(start_offset, doppler);
                }
            }
        }

        let recording = self.auto_recording;

        if bar_index != self.prev_bar_index && !recording && self.buffer_length > CROSSFADE_LEN {
            if self.prev_bar_index != u64::MAX {
                if !freeze {
                    self.repeat_gain *= decay;
                }
                if self.repeat_gain > 0.001 {
                    self.trigger_playback(start_offset, doppler);
                } else {
                    self.playing = false;
                }
            }
            self.prev_bar_index = bar_index;
        }
        if self.prev_bar_index == u64::MAX {
            self.prev_bar_index = bar_index;
        }

        let base_pitch_ratio = 2.0_f64.powf(pitch_semitones / 12.0);
        let mut key_ratio = 1.0;
        if key_track && self.record_frequency > 20.0 && current_frequency > 20.0 {
            key_ratio = current_frequency / self.record_frequency;
        }

        let length_samples = (length_beats * beat_samples) as usize;

        let stutter_divs = Self::stutter_divisions(stutter) as usize;
        let max_buf = self.buffer_l.len();
        let buf_len = self.buffer_length;
        let num_samples = buffer_l.len();

        for i in 0..num_samples {
            if recording && self.write_pos < max_buf {
                self.buffer_l[self.write_pos] = input_l[i];
                self.buffer_r[self.write_pos] = input_r[i];
                self.write_pos += 1;
                if self.auto_recording {
                    self.auto_rec_samples_written += 1;
                }
            }

            if !recording && self.playing && buf_len > 0 && length_samples > 0 {
                if self.playback_elapsed >= length_samples {
                    self.playing = false;
                    continue;
                }

                let pitch_ratio = base_pitch_ratio
                    * key_ratio
                    * 2.0_f64.powf(self.doppler_envelope / 12.0);
                self.doppler_envelope *= self.doppler_coef;

                let fade_in = (self.playback_elapsed as f64 / CROSSFADE_LEN as f64).min(1.0);
                let remaining = length_samples - self.playback_elapsed;
                let fade_out = (remaining as f64 / CROSSFADE_LEN as f64).min(1.0);
                let envelope = fade_in * fade_out;

                let read_pos = if stutter_divs > 1 {
                    self.compute_stutter_position(
                        stutter_divs, length_samples, start_offset,
                        pitch_ratio, direction, buf_len,
                    )
                } else {
                    self.read_phase
                };

                let clamped = read_pos.clamp(0.0, (buf_len - 1) as f64);
                let (sl, sr) = self.read_interpolated(clamped);

                self.mix_slew += (mix - self.mix_slew) * self.slew_coeff;
                let gain = (self.repeat_gain * self.mix_slew * envelope) as f32;

                buffer_l[i] += sl * gain;
                buffer_r[i] += sr * gain;

                if stutter_divs <= 1 {
                    self.advance_phase(pitch_ratio, start_offset, buf_len, direction);
                }

                self.playback_elapsed += 1;
            }
        }
    }

    fn apply_record_crossfade(&mut self) {
        let len = self.buffer_length;
        if len <= CROSSFADE_LEN * 2 {
            return;
        }
        for i in 0..CROSSFADE_LEN {
            let t = i as f32 / CROSSFADE_LEN as f32;
            let fade_in = t * t * (3.0 - 2.0 * t);
            self.buffer_l[i] *= fade_in;
            self.buffer_r[i] *= fade_in;
            let end_idx = len - 1 - i;
            self.buffer_l[end_idx] *= fade_in;
            self.buffer_r[end_idx] *= fade_in;
        }
    }

    fn trigger_playback(&mut self, start_offset: f64, doppler: f64) {
        self.read_phase = start_offset * self.buffer_length as f64;
        self.playing = true;
        self.playback_elapsed = 0;
        self.ping_forward = true;
        self.doppler_envelope = doppler;
    }

    fn compute_stutter_position(
        &self,
        stutter_divs: usize,
        length_samples: usize,
        start_offset: f64,
        pitch_ratio: f64,
        direction: LoopDirection,
        buf_len: usize,
    ) -> f64 {
        let time_per_slice = length_samples / stutter_divs.max(1);
        let current_slice = if time_per_slice > 0 {
            (self.playback_elapsed / time_per_slice).min(stutter_divs - 1)
        } else {
            0
        };
        let buf_per_slice = buf_len as f64 / stutter_divs as f64;
        let start = start_offset * buf_len as f64;
        let slice_start = start + current_slice as f64 * buf_per_slice;
        let local_elapsed = self.playback_elapsed % time_per_slice.max(1);
        let local_phase = local_elapsed as f64 * pitch_ratio;

        match direction {
            LoopDirection::Forward => {
                let p = local_phase % buf_per_slice.max(1.0);
                slice_start + p
            }
            LoopDirection::Reverse => {
                let p = local_phase % buf_per_slice.max(1.0);
                slice_start + buf_per_slice - p - 1.0
            }
            LoopDirection::PingPong => {
                let cycle = buf_per_slice * 2.0;
                if cycle < 1.0 { return slice_start; }
                let p = local_phase % cycle;
                if p < buf_per_slice {
                    slice_start + p
                } else {
                    slice_start + cycle - p
                }
            }
        }
    }

    fn advance_phase(
        &mut self,
        pitch_ratio: f64,
        start_offset: f64,
        buf_len: usize,
        direction: LoopDirection,
    ) {
        let start = start_offset * buf_len as f64;
        let end = buf_len as f64;

        match direction {
            LoopDirection::Forward => {
                self.read_phase += pitch_ratio;
                if self.read_phase >= end {
                    self.read_phase = start;
                }
            }
            LoopDirection::Reverse => {
                self.read_phase -= pitch_ratio;
                if self.read_phase < start {
                    self.read_phase = end - 1.0;
                }
            }
            LoopDirection::PingPong => {
                if self.ping_forward {
                    self.read_phase += pitch_ratio;
                    if self.read_phase >= end {
                        self.read_phase = end - 1.0;
                        self.ping_forward = false;
                    }
                } else {
                    self.read_phase -= pitch_ratio;
                    if self.read_phase < start {
                        self.read_phase = start;
                        self.ping_forward = true;
                    }
                }
            }
        }
    }

    #[inline]
    fn read_interpolated(&self, phase: f64) -> (f32, f32) {
        let len = self.buffer_length;
        if len == 0 {
            return (0.0, 0.0);
        }
        let idx = phase.floor() as isize;
        let frac = phase - phase.floor();
        (
            Self::sinc16(&self.buffer_l[..len], idx, frac),
            Self::sinc16(&self.buffer_r[..len], idx, frac),
        )
    }

    #[inline]
    fn sinc16(buf: &[f32], idx: isize, frac: f64) -> f32 {
        let len = buf.len() as isize;
        let mut sum = 0.0_f64;
        let mut win_sum = 0.0_f64;
        for i in -7_isize..=8 {
            let d = frac - i as f64;
            let s = if d.abs() < 1e-9 {
                1.0
            } else {
                let pi_d = std::f64::consts::PI * d;
                (pi_d.sin() / pi_d) * Self::kaiser_window(d, 8.0, SINC_BETA)
            };
            let sample_idx = (idx + i).clamp(0, len - 1) as usize;
            sum += buf[sample_idx] as f64 * s;
            win_sum += s;
        }
        if win_sum.abs() > 1e-9 { (sum / win_sum) as f32 } else { 0.0 }
    }

    #[inline]
    fn kaiser_window(n: f64, half_len: f64, beta: f64) -> f64 {
        let r = n / half_len;
        if r.abs() > 1.0 {
            return 0.0;
        }
        Self::bessel_i0(beta * (1.0 - r * r).sqrt()) / Self::bessel_i0(beta)
    }

    #[inline]
    fn bessel_i0(x: f64) -> f64 {
        let mut sum = 1.0_f64;
        let mut term = 1.0_f64;
        let x_half = x * 0.5;
        for k in 1..=16 {
            term *= (x_half / k as f64) * (x_half / k as f64);
            sum += term;
        }
        sum
    }

    fn stutter_divisions(index: i32) -> u32 {
        match index {
            1 => 2,
            2 => 4,
            3 => 8,
            4 => 16,
            _ => 1,
        }
    }

    fn auto_rec_interval_bars(index: i32) -> u64 {
        match index {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            _ => 4,
        }
    }
}
