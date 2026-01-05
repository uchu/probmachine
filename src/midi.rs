#![allow(dead_code)]

use nih_plug::midi::NoteEvent;
use crate::params::DeviceParams;
use std::sync::Arc;

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

pub fn process_midi_event(event: NoteEvent<()>, midi_state: &mut MidiCCState, _params: &Arc<DeviceParams>) {
    if let NoteEvent::MidiCC { cc, value, .. } = event {
        midi_state.process_cc(cc, value);
    }
}
