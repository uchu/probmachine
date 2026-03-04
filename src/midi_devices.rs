use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use nih_plug::midi::NoteEvent;
use crate::midi_learn::{CcMapping, MidiLearnMappings};

#[derive(Clone, Copy)]
pub struct RawMidiMessage {
    pub data: [u8; 3],
    pub len: u8,
}

pub type MidiInputQueue = Arc<Mutex<VecDeque<RawMidiMessage>>>;
pub type MidiOutputQueue = Arc<Mutex<VecDeque<RawMidiMessage>>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub input_device_name: Option<String>,
    pub output_device_name: Option<String>,
    pub input_channel: MidiChannel,
    pub output_channel: u8,
    pub midi_mode: u8,
    #[serde(default)]
    pub midi_learn_mappings: Vec<CcMapping>,
    #[serde(default)]
    pub selector_cc: Option<u8>,
    #[serde(default)]
    pub value_cc: Option<u8>,
    #[serde(default = "default_true")]
    pub soft_takeover: bool,
    #[serde(default)]
    pub midi_clock_in: bool,
    #[serde(default)]
    pub midi_clock_out: bool,
    #[serde(default)]
    pub midi_transport_in: bool,
    #[serde(default)]
    pub midi_transport_out: bool,
}

fn default_true() -> bool { true }

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            input_device_name: None,
            output_device_name: None,
            input_channel: MidiChannel::All,
            output_channel: 0,
            midi_mode: 0,
            midi_learn_mappings: Vec::new(),
            selector_cc: None,
            value_cc: None,
            soft_takeover: true,
            midi_clock_in: false,
            midi_clock_out: false,
            midi_transport_in: false,
            midi_transport_out: false,
        }
    }
}

impl SettingsConfig {
    pub fn midi_learn_mappings_data(&self) -> MidiLearnMappings {
        MidiLearnMappings {
            mappings: self.midi_learn_mappings.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiChannel {
    All,
    Channel(u8),
}

impl MidiChannel {
    pub fn label(&self) -> String {
        match self {
            MidiChannel::All => "All".to_string(),
            MidiChannel::Channel(ch) => format!("{}", ch + 1),
        }
    }

    pub fn all_options() -> Vec<MidiChannel> {
        let mut opts = vec![MidiChannel::All];
        for i in 0..16 {
            opts.push(MidiChannel::Channel(i));
        }
        opts
    }

    pub fn matches(&self, channel: u8) -> bool {
        match self {
            MidiChannel::All => true,
            MidiChannel::Channel(ch) => *ch == channel,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MidiDeviceInfo {
    pub name: String,
}

pub struct MidiDeviceManager {
    config: SettingsConfig,
    input_devices: Vec<MidiDeviceInfo>,
    output_devices: Vec<MidiDeviceInfo>,
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: Option<MidiOutputConnection>,
    input_queue: MidiInputQueue,
    output_queue: MidiOutputQueue,
}

impl MidiDeviceManager {
    pub fn new() -> Self {
        Self {
            config: SettingsConfig::default(),
            input_devices: Vec::new(),
            output_devices: Vec::new(),
            input_connection: None,
            output_connection: None,
            input_queue: Arc::new(Mutex::new(VecDeque::with_capacity(256))),
            output_queue: Arc::new(Mutex::new(VecDeque::with_capacity(256))),
        }
    }

    pub fn input_queue(&self) -> MidiInputQueue {
        self.input_queue.clone()
    }

    pub fn output_queue(&self) -> MidiOutputQueue {
        self.output_queue.clone()
    }

    pub fn input_devices(&self) -> &[MidiDeviceInfo] {
        &self.input_devices
    }

    pub fn output_devices(&self) -> &[MidiDeviceInfo] {
        &self.output_devices
    }

    pub fn input_channel(&self) -> &MidiChannel {
        &self.config.input_channel
    }

    pub fn output_channel(&self) -> u8 {
        self.config.output_channel
    }

    pub fn connected_input_name(&self) -> Option<&str> {
        self.config.input_device_name.as_deref()
    }

    pub fn connected_output_name(&self) -> Option<&str> {
        self.config.output_device_name.as_deref()
    }

    pub fn refresh_devices(&mut self) {
        self.input_devices.clear();
        self.output_devices.clear();

        if let Ok(midi_in) = MidiInput::new("PhaseBurn probe") {
            for port in midi_in.ports() {
                if let Ok(name) = midi_in.port_name(&port) {
                    self.input_devices.push(MidiDeviceInfo { name });
                }
            }
        }

        if let Ok(midi_out) = MidiOutput::new("PhaseBurn probe") {
            for port in midi_out.ports() {
                if let Ok(name) = midi_out.port_name(&port) {
                    self.output_devices.push(MidiDeviceInfo { name });
                }
            }
        }
    }

    pub fn connect_input(&mut self, name: &str) -> bool {
        self.disconnect_input();

        let Ok(midi_in) = MidiInput::new("PhaseBurn input") else {
            return false;
        };

        let port = midi_in.ports().into_iter().enumerate().find(|(i, _)| {
            midi_in.port_name(&midi_in.ports()[*i]).map(|n| n == name).unwrap_or(false)
        });

        let Some((_, port)) = port else {
            return false;
        };

        let queue = self.input_queue.clone();
        let channel_filter = self.config.input_channel.clone();

        let result = midi_in.connect(
            &port,
            "PhaseBurn input",
            move |_timestamp, data, _| {
                if data.is_empty() {
                    return;
                }

                let status = data[0];
                let is_realtime = matches!(status, 0xF8 | 0xFA | 0xFB | 0xFC);
                if !is_realtime && !(0x80..0xF0).contains(&status) {
                    return;
                }

                if !is_realtime {
                    let msg_channel = status & 0x0F;
                    if !channel_filter.matches(msg_channel) {
                        return;
                    }
                }

                let mut msg = RawMidiMessage {
                    data: [0; 3],
                    len: data.len().min(3) as u8,
                };
                for (i, &byte) in data.iter().take(3).enumerate() {
                    msg.data[i] = byte;
                }

                if let Ok(mut q) = queue.try_lock() {
                    if q.len() < 512 {
                        q.push_back(msg);
                    }
                }
            },
            (),
        );

        match result {
            Ok(conn) => {
                self.input_connection = Some(conn);
                self.config.input_device_name = Some(name.to_string());
                true
            }
            Err(_) => false,
        }
    }

    pub fn connect_output(&mut self, name: &str) -> bool {
        self.disconnect_output();

        let Ok(midi_out) = MidiOutput::new("PhaseBurn output") else {
            return false;
        };

        let port = midi_out.ports().into_iter().enumerate().find(|(i, _)| {
            midi_out.port_name(&midi_out.ports()[*i]).map(|n| n == name).unwrap_or(false)
        });

        let Some((_, port)) = port else {
            return false;
        };

        match midi_out.connect(&port, "PhaseBurn output") {
            Ok(conn) => {
                self.output_connection = Some(conn);
                self.config.output_device_name = Some(name.to_string());
                true
            }
            Err(_) => false,
        }
    }

    pub fn disconnect_input(&mut self) {
        if let Some(conn) = self.input_connection.take() {
            conn.close();
        }
        self.config.input_device_name = None;
    }

    pub fn disconnect_output(&mut self) {
        if let Some(conn) = self.output_connection.take() {
            conn.close();
        }
        self.config.output_device_name = None;
    }

    pub fn set_input_channel(&mut self, channel: MidiChannel) {
        self.config.input_channel = channel;
        if let Some(name) = self.config.input_device_name.clone() {
            self.connect_input(&name);
        }
    }

    pub fn set_output_channel(&mut self, channel: u8) {
        self.config.output_channel = channel.min(15);
    }

    pub fn set_midi_mode(&mut self, mode: u8) {
        self.config.midi_mode = mode;
    }

    pub fn set_midi_learn_mappings(&mut self, mappings: Vec<CcMapping>) {
        self.config.midi_learn_mappings = mappings;
    }

    pub fn set_selector_cc(&mut self, cc: Option<u8>) {
        self.config.selector_cc = cc;
    }

    pub fn set_value_cc(&mut self, cc: Option<u8>) {
        self.config.value_cc = cc;
    }

    pub fn set_soft_takeover(&mut self, enabled: bool) {
        self.config.soft_takeover = enabled;
    }

    pub fn set_midi_clock_in(&mut self, enabled: bool) {
        self.config.midi_clock_in = enabled;
    }

    pub fn set_midi_clock_out(&mut self, enabled: bool) {
        self.config.midi_clock_out = enabled;
    }

    pub fn set_midi_transport_in(&mut self, enabled: bool) {
        self.config.midi_transport_in = enabled;
    }

    pub fn set_midi_transport_out(&mut self, enabled: bool) {
        self.config.midi_transport_out = enabled;
    }

    pub fn has_feedback_risk(&self) -> bool {
        let (Some(in_name), Some(out_name)) = (&self.config.input_device_name, &self.config.output_device_name) else {
            return false;
        };
        if in_name != out_name {
            return false;
        }
        match &self.config.input_channel {
            MidiChannel::All => true,
            MidiChannel::Channel(ch) => *ch == self.config.output_channel,
        }
    }

    pub fn auto_select_if_single(&mut self) {
        if self.config.input_device_name.is_none() && self.input_devices.len() == 1 {
            let name = self.input_devices[0].name.clone();
            self.connect_input(&name);
        }
        if self.config.output_device_name.is_none() && self.output_devices.len() == 1 {
            let name = self.output_devices[0].name.clone();
            self.connect_output(&name);
        }
    }

    pub fn flush_output(&mut self) {
        let Some(conn) = &mut self.output_connection else {
            return;
        };
        if let Ok(mut q) = self.output_queue.try_lock() {
            while let Some(msg) = q.pop_front() {
                let len = msg.len as usize;
                let _ = conn.send(&msg.data[..len]);
            }
        }
    }

    pub fn save_config(&self) {
        let Some(path) = settings_file_path() else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn load_config(&mut self) -> SettingsConfig {
        let Some(path) = settings_file_path() else {
            return self.config.clone();
        };
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<SettingsConfig>(&data) {
                self.config = cfg;
            }
        }
        self.config.clone()
    }

    pub fn reconnect_saved_devices(&mut self) {
        if let Some(name) = self.config.input_device_name.clone() {
            self.connect_input(&name);
        }
        if let Some(name) = self.config.output_device_name.clone() {
            self.connect_output(&name);
        }
    }
}

fn settings_file_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|mut path| {
        path.push("Device");
        path.push("settings.json");
        path
    })
}

pub fn raw_midi_to_note_event(msg: &RawMidiMessage) -> Option<NoteEvent<()>> {
    if msg.len < 2 {
        return None;
    }
    let status = msg.data[0] & 0xF0;
    let channel = msg.data[0] & 0x0F;
    let note = msg.data[1];
    let velocity = if msg.len >= 3 { msg.data[2] } else { 0 };

    match status {
        0x90 if velocity > 0 => Some(NoteEvent::NoteOn {
            timing: 0,
            voice_id: None,
            channel,
            note,
            velocity: velocity as f32 / 127.0,
        }),
        0x90 | 0x80 => Some(NoteEvent::NoteOff {
            timing: 0,
            voice_id: None,
            channel,
            note,
            velocity: velocity as f32 / 127.0,
        }),
        0xB0 if msg.len >= 3 => Some(NoteEvent::MidiCC {
            timing: 0,
            channel,
            cc: note,
            value: velocity as f32 / 127.0,
        }),
        _ => None,
    }
}
