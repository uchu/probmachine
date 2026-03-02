use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::sequencer::ml_suggest::{BeatSuggester, PitchSuggester};
use crate::sequencer::melodic_engine::MelodySuggester;

const LZ4_MAGIC: &[u8; 4] = b"LZ4\0";

pub fn decompress_if_needed(data: &[u8]) -> Vec<u8> {
    if data.len() >= 8 && &data[0..4] == LZ4_MAGIC {
        let uncompressed_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        lz4_flex::decompress(&data[8..], uncompressed_size).unwrap_or_default()
    } else {
        data.to_vec()
    }
}

fn read_and_decompress(path: &Path) -> Result<Vec<u8>, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    Ok(decompress_if_needed(&data))
}

fn read_distribution_count(path: &Path) -> usize {
    let Ok(data) = std::fs::read(path) else { return 0 };
    let decompressed = decompress_if_needed(&data);
    if decompressed.len() >= 9 {
        u32::from_le_bytes([decompressed[5], decompressed[6], decompressed[7], decompressed[8]]) as usize
    } else {
        0
    }
}

#[derive(Clone)]
pub struct PerformanceParams {
    pub len_mod_1_target: f32,
    pub len_mod_1_amount: f32,
    pub len_mod_1_prob: f32,
    pub len_mod_2_target: f32,
    pub len_mod_2_amount: f32,
    pub len_mod_2_prob: f32,
    pub vel_strength_target: f32,
    pub vel_strength_amount: f32,
    pub vel_strength_prob: f32,
    pub vel_length_target: f32,
    pub vel_length_amount: f32,
    pub vel_length_prob: f32,
    pub pos_mod_1_target: f32,
    pub pos_mod_1_shift: f32,
    pub pos_mod_1_prob: f32,
    pub pos_mod_2_target: f32,
    pub pos_mod_2_shift: f32,
    pub pos_mod_2_prob: f32,
}

fn parse_performance(data: &[u8]) -> Option<PerformanceParams> {
    if data.len() < 77 || &data[0..4] != b"PFDT" {
        return None;
    }
    let _version = data[4];
    let mut offset = 5;
    let mut vals = [0.0f32; 18];
    for v in &mut vals {
        *v = f32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
    }
    Some(PerformanceParams {
        len_mod_1_target: vals[0],
        len_mod_1_amount: vals[1],
        len_mod_1_prob: vals[2],
        len_mod_2_target: vals[3],
        len_mod_2_amount: vals[4],
        len_mod_2_prob: vals[5],
        vel_strength_target: vals[6],
        vel_strength_amount: vals[7],
        vel_strength_prob: vals[8],
        vel_length_target: vals[9],
        vel_length_amount: vals[10],
        vel_length_prob: vals[11],
        pos_mod_1_target: vals[12],
        pos_mod_1_shift: vals[13],
        pos_mod_1_prob: vals[14],
        pos_mod_2_target: vals[15],
        pos_mod_2_shift: vals[16],
        pos_mod_2_prob: vals[17],
    })
}

pub struct MlDataset {
    pub name: String,
    pub beat: BeatSuggester,
    pub pitch: PitchSuggester,
    pub melody: MelodySuggester,
    pub performance: Option<PerformanceParams>,
}

impl MlDataset {
    pub fn builtin() -> Self {
        let beat = BeatSuggester::new();
        let pitch = PitchSuggester::new();
        let melody = MelodySuggester::new();
        let perf_data = include_bytes!("perf_data.bin");
        let performance = parse_performance(perf_data);
        Self {
            name: "Built-in".to_string(),
            beat,
            pitch,
            melody,
            performance,
        }
    }

    pub fn load_from_dir(path: &Path, name: &str) -> Result<Self, String> {
        let beat_data = read_and_decompress(&path.join("beat_data.bin"))?;
        let pitch_data = read_and_decompress(&path.join("pitch_data.bin"))?;
        let melody_data = read_and_decompress(&path.join("melody_data.bin"))?;

        let beat = BeatSuggester::from_data(&beat_data);
        let pitch = PitchSuggester::from_data(&pitch_data);
        let melody = MelodySuggester::from_data(&melody_data);

        let performance = if path.join("perf_data.bin").exists() {
            let perf_data = read_and_decompress(&path.join("perf_data.bin"))?;
            parse_performance(&perf_data)
        } else {
            None
        };

        if !beat.is_available() {
            return Err("Beat data is empty or invalid".to_string());
        }

        Ok(Self {
            name: name.to_string(),
            beat,
            pitch,
            melody,
            performance,
        })
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct DatasetInfo {
    pub name: String,
    pub dir_name: String,
    pub beat_count: usize,
    pub pitch_count: usize,
    pub melody_count: usize,
}

pub fn get_datasets_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|mut path| {
        path.push("Device");
        path.push("datasets");
        path
    })
}

pub fn list_datasets() -> Vec<DatasetInfo> {
    let builtin_beat = BeatSuggester::new();
    let builtin_pitch = PitchSuggester::new();
    let builtin_melody = MelodySuggester::new();

    let mut datasets = vec![DatasetInfo {
        name: "Built-in".to_string(),
        dir_name: String::new(),
        beat_count: builtin_beat.distribution_count(),
        pitch_count: builtin_pitch.distribution_count(),
        melody_count: builtin_melody.fragment_count(),
    }];

    let Some(datasets_dir) = get_datasets_dir() else {
        return datasets;
    };

    let Ok(entries) = std::fs::read_dir(&datasets_dir) else {
        return datasets;
    };

    let mut external: Vec<DatasetInfo> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }

            let dir_name = entry.file_name().to_string_lossy().to_string();
            if !path.join("beat_data.bin").exists() {
                return None;
            }

            let name = format_dataset_name(&dir_name);
            let beat_count = read_distribution_count(&path.join("beat_data.bin"));
            let pitch_count = read_distribution_count(&path.join("pitch_data.bin"));
            let melody_count = read_distribution_count(&path.join("melody_data.bin"));

            Some(DatasetInfo {
                name,
                dir_name,
                beat_count,
                pitch_count,
                melody_count,
            })
        })
        .collect();

    external.sort_by(|a, b| a.name.cmp(&b.name));
    datasets.extend(external);
    datasets
}

fn format_dataset_name(dir_name: &str) -> String {
    dir_name.replace('-', " ").replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + c.as_str()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn load_dataset(name: &str) -> Result<Arc<MlDataset>, String> {
    if name == "Built-in" {
        return Ok(Arc::new(MlDataset::builtin()));
    }

    let datasets_dir = get_datasets_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?;

    let dir = datasets_dir.join(name);
    if !dir.exists() {
        return Err(format!("Dataset directory not found: {}", dir.display()));
    }

    MlDataset::load_from_dir(&dir, name).map(Arc::new)
}
