#![allow(dead_code)]

use nih_plug::midi::NoteEvent;
use nih_plug::prelude::ProcessContext;
use crate::sequencer::midi_to_frequency;

pub struct MidiCCState {
    cc_msb: [u8; 32],
    nrpn_msb: Option<u8>,
    nrpn_lsb: Option<u8>,
    last_cc_values: [f32; 128],
}

impl Default for MidiCCState {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiCCState {
    pub fn new() -> Self {
        Self {
            cc_msb: [0; 32],
            nrpn_msb: None,
            nrpn_lsb: None,
            last_cc_values: [0.0; 128],
        }
    }

    pub fn get_cc(&self, cc: u8) -> f32 {
        self.last_cc_values.get(cc as usize).copied().unwrap_or(0.0)
    }

    pub fn get_cc_14bit(&self, cc: u8) -> f32 {
        if cc < 32 {
            let msb = self.cc_msb[cc as usize] as u16;
            let lsb = self.last_cc_values.get((cc + 32) as usize).map(|v| (*v * 127.0) as u16).unwrap_or(0);
            let value_14bit = (msb << 7) | lsb;
            value_14bit as f32 / 16383.0
        } else {
            self.get_cc(cc)
        }
    }

    pub fn process_cc(&mut self, cc: u8, value: f32) {
        if let Some(slot) = self.last_cc_values.get_mut(cc as usize) {
            *slot = value;
        }

        match cc {
            0..=31 => {
                self.cc_msb[cc as usize] = (value * 127.0) as u8;
            }
            99 => {
                self.nrpn_msb = Some((value * 127.0) as u8);
            }
            98 => {
                self.nrpn_lsb = Some((value * 127.0) as u8);
            }
            _ => {}
        }
    }

    pub fn get_nrpn_address(&self) -> Option<u16> {
        if let (Some(msb), Some(lsb)) = (self.nrpn_msb, self.nrpn_lsb) {
            Some(((msb as u16) << 7) | (lsb as u16))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExternalNoteEvent {
    pub note: u8,
    pub velocity: u8,
    pub frequency: f64,
    pub is_note_on: bool,
    pub timing: u32,
}

pub struct MidiState {
    pub cc_state: MidiCCState,
    pub external_notes: Vec<ExternalNoteEvent>,
    active_notes: [bool; 128],
    active_note_count: u8,
    pub midi_input_enabled: bool,
}

impl Default for MidiState {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiState {
    pub fn new() -> Self {
        Self {
            cc_state: MidiCCState::new(),
            external_notes: Vec::with_capacity(16),
            active_notes: [false; 128],
            active_note_count: 0,
            midi_input_enabled: true,
        }
    }

    pub fn clear_notes(&mut self) {
        self.external_notes.clear();
    }

    pub fn process_event(&mut self, event: NoteEvent<()>) {
        match event {
            NoteEvent::NoteOn { note, velocity, timing, .. } => {
                if self.midi_input_enabled {
                    let velocity_u8 = (velocity * 127.0).round() as u8;
                    self.external_notes.push(ExternalNoteEvent {
                        note,
                        velocity: velocity_u8,
                        frequency: midi_to_frequency(note) as f64,
                        is_note_on: true,
                        timing,
                    });
                    if !self.active_notes[note as usize] {
                        self.active_notes[note as usize] = true;
                        self.active_note_count = self.active_note_count.saturating_add(1);
                    }
                }
            }
            NoteEvent::NoteOff { note, timing, .. } => {
                if self.midi_input_enabled && self.active_notes[note as usize] {
                    self.external_notes.push(ExternalNoteEvent {
                        note,
                        velocity: 0,
                        frequency: midi_to_frequency(note) as f64,
                        is_note_on: false,
                        timing,
                    });
                    self.active_notes[note as usize] = false;
                    self.active_note_count = self.active_note_count.saturating_sub(1);
                }
            }
            NoteEvent::MidiCC { cc, value, .. } => {
                self.cc_state.process_cc(cc, value);
            }
            _ => {}
        }
    }

    pub fn get_note_event_at(&self, sample_offset: u32) -> Option<&ExternalNoteEvent> {
        self.external_notes.iter().find(|e| e.timing == sample_offset)
    }

    pub fn has_active_note(&self) -> bool {
        self.active_note_count > 0
    }
}

pub struct MidiOutput {
    pending_notes: Vec<PendingMidiNote>,
    clock_counter: f64,
    samples_per_clock: f64,
    last_was_playing: bool,
    output_channel: u8,
}

#[derive(Clone, Copy)]
struct PendingMidiNote {
    note: u8,
    velocity: u8,
    is_on: bool,
    sample_offset: u32,
}

impl Default for MidiOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiOutput {
    pub fn new() -> Self {
        Self {
            pending_notes: Vec::with_capacity(32),
            clock_counter: 0.0,
            samples_per_clock: 0.0,
            last_was_playing: false,
            output_channel: 0,
        }
    }

    pub fn set_output_channel(&mut self, channel: u8) {
        self.output_channel = channel.min(15);
    }

    pub fn update_tempo(&mut self, sample_rate: f32, tempo: f64) {
        let beats_per_second = tempo / 60.0;
        let clocks_per_beat = 24.0;
        let clocks_per_second = beats_per_second * clocks_per_beat;
        self.samples_per_clock = sample_rate as f64 / clocks_per_second;
    }

    pub fn queue_note_on(&mut self, note: u8, velocity: u8, sample_offset: u32) {
        self.pending_notes.push(PendingMidiNote {
            note,
            velocity,
            is_on: true,
            sample_offset,
        });
    }

    pub fn queue_note_off(&mut self, note: u8, sample_offset: u32) {
        self.pending_notes.push(PendingMidiNote {
            note,
            velocity: 0,
            is_on: false,
            sample_offset,
        });
    }

    pub fn send_pending<P: nih_plug::prelude::Plugin>(
        &mut self,
        context: &mut impl ProcessContext<P>,
        is_playing: bool,
        buffer_samples: usize,
        sample_rate: f32,
        tempo: f64,
    ) {
        self.update_tempo(sample_rate, tempo);

        if is_playing && !self.last_was_playing {
            self.clock_counter = 0.0;
        }
        self.last_was_playing = is_playing;

        for pending in self.pending_notes.drain(..) {
            if pending.is_on {
                context.send_event(NoteEvent::NoteOn {
                    timing: pending.sample_offset,
                    voice_id: None,
                    channel: self.output_channel,
                    note: pending.note,
                    velocity: pending.velocity as f32 / 127.0,
                });
            } else {
                context.send_event(NoteEvent::NoteOff {
                    timing: pending.sample_offset,
                    voice_id: None,
                    channel: self.output_channel,
                    note: pending.note,
                    velocity: 0.0,
                });
            }
        }

        if is_playing && self.samples_per_clock > 0.0 {
            let mut sample_in_buffer = 0u32;
            while sample_in_buffer < buffer_samples as u32 {
                let samples_until_clock = self.samples_per_clock - self.clock_counter;
                if samples_until_clock <= 0.0 {
                    self.clock_counter = 0.0;
                    continue;
                }

                let next_clock_sample = sample_in_buffer + samples_until_clock as u32;
                if next_clock_sample < buffer_samples as u32 {
                    self.clock_counter = 0.0;
                    sample_in_buffer = next_clock_sample + 1;
                } else {
                    self.clock_counter += (buffer_samples as u32 - sample_in_buffer) as f64;
                    break;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.pending_notes.clear();
    }
}

pub struct MidiProcessor {
    pub input: MidiState,
    pub output: MidiOutput,
    current_sequencer_note: Option<u8>,
}

impl Default for MidiProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiProcessor {
    pub fn new() -> Self {
        Self {
            input: MidiState::new(),
            output: MidiOutput::new(),
            current_sequencer_note: None,
        }
    }

    pub fn process_incoming_event(&mut self, event: NoteEvent<()>) {
        self.input.process_event(event);
    }

    pub fn begin_buffer(&mut self) {
        self.input.clear_notes();
    }

    pub fn note_on_from_sequencer(&mut self, midi_note: u8, velocity: u8, sample_offset: u32) {
        if let Some(old_note) = self.current_sequencer_note {
            self.output.queue_note_off(old_note, sample_offset);
        }
        self.output.queue_note_on(midi_note, velocity, sample_offset);
        self.current_sequencer_note = Some(midi_note);
    }

    pub fn note_off_from_sequencer(&mut self, sample_offset: u32) {
        if let Some(note) = self.current_sequencer_note.take() {
            self.output.queue_note_off(note, sample_offset);
        }
    }

    pub fn send_output<P: nih_plug::prelude::Plugin>(
        &mut self,
        context: &mut impl ProcessContext<P>,
        is_playing: bool,
        buffer_samples: usize,
        sample_rate: f32,
        tempo: f64,
    ) {
        self.output.send_pending(context, is_playing, buffer_samples, sample_rate, tempo);
    }

    pub fn get_cc(&self, cc: u8) -> f32 {
        self.input.cc_state.get_cc(cc)
    }

    #[allow(dead_code)]
    pub fn get_cc_14bit(&self, cc: u8) -> f32 {
        self.input.cc_state.get_cc_14bit(cc)
    }

    pub fn set_midi_input_enabled(&mut self, enabled: bool) {
        self.input.midi_input_enabled = enabled;
    }

    pub fn set_output_channel(&mut self, channel: u8) {
        self.output.set_output_channel(channel);
    }

    pub fn stop_all_notes(&mut self, sample_offset: u32) {
        if let Some(note) = self.current_sequencer_note.take() {
            self.output.queue_note_off(note, sample_offset);
        }
    }

    pub fn clear_all(&mut self) {
        if let Some(note) = self.current_sequencer_note.take() {
            self.output.queue_note_off(note, 0);
        }
        self.output.clear();
    }
}
