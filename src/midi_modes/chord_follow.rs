use crate::sequencer::NotePool;

pub struct ChordFollowState {
    held_notes: [Option<u8>; 128],
    dirty: bool,
}

impl ChordFollowState {
    pub fn new() -> Self {
        Self {
            held_notes: [None; 128],
            dirty: false,
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: u8) {
        self.held_notes[note as usize] = Some(velocity);
        self.dirty = true;
    }

    pub fn note_off(&mut self, note: u8) {
        if self.held_notes[note as usize].is_some() {
            self.held_notes[note as usize] = None;
            self.dirty = true;
        }
    }

    pub fn is_dirty(&mut self) -> bool {
        let d = self.dirty;
        self.dirty = false;
        d
    }

    pub fn clear(&mut self) {
        self.held_notes = [None; 128];
        self.dirty = true;
    }

    pub fn get_held_notes(&self) -> Vec<(u8, u8)> {
        self.held_notes
            .iter()
            .enumerate()
            .filter_map(|(note, vel)| vel.map(|v| (note as u8, v)))
            .collect()
    }

    pub fn build_note_pool(&self) -> NotePool {
        let mut pool = NotePool::new();
        let held: Vec<(u8, u8)> = self.get_held_notes();

        if held.is_empty() {
            return pool;
        }

        let root_note = held[0].0;
        pool.set_root_note(root_note);

        for &(note, velocity) in &held {
            let chance = velocity as f32 / 127.0;
            pool.set_note(note, chance, 0.0);
        }

        pool
    }
}
