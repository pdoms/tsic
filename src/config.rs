use std::path::Path;

use serde::{Deserialize, Serialize};

pub const DEFAULT_SAMPLE_RATE: u32 = 44100;
pub const DEFAULT_BPM: u32 = 120;
pub const DEFAULT_BEATS_PER_BAR: u32 = 4;
pub const DEFAULT_BEAT_UNIT: u32 = 4;

pub const DEFAULT_FREQUENCY_HZ: f64 = 1200.0;
pub const DEFAULT_ACCENT_FREQUENCY_HZ: f64 = 800.0;
pub const DEFAULT_SOUND_DURATION_SECS: f64 = 0.2;
pub const DEFAULT_ENVELOPE_DECAY_SECS: f64 = 0.1;

pub const DEFAULT_NUM_MEASSURES: u32 = 128;

pub const PROFILE_DIR: &str = "./profiles";
pub const PROJECTS_DIR: &str = "./projects";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub sample_rate: u32,
    pub bpm: u32,
    pub beats_per_bar: u32,
    pub beat_unit: u32,
    pub num_meassures_fallback: u32,
    pub sound: Sound,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            sample_rate: DEFAULT_SAMPLE_RATE,
            bpm: DEFAULT_BPM,
            beats_per_bar: DEFAULT_BEATS_PER_BAR,
            beat_unit: DEFAULT_BEAT_UNIT,
            num_meassures_fallback: DEFAULT_NUM_MEASSURES,
            sound: Sound::default(),
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "sample_rate:            {}", self.sample_rate)?;
        writeln!(f, "bpm:                    {}", self.bpm)?;
        writeln!(f, "beats_per_bar:          {}", self.beats_per_bar)?;
        writeln!(f, "beat_unit:              {}", self.beat_unit)?;
        writeln!(f, "meassures_fallback:     {}", self.num_meassures_fallback)?;
        writeln!(f, "sound.frequency:        {} hz", self.sound.frequency_hz)?;
        writeln!(
            f,
            "sound.accent_frequency: {} hz",
            self.sound.accent_frequency_hz
        )?;
        writeln!(
            f,
            "sound.sound_duration:   {} seconds",
            self.sound.accent_frequency_hz
        )?;
        writeln!(
            f,
            "sound.envelope_decay:   {} seconds",
            self.sound.accent_frequency_hz
        )
    }
}

impl Config {
    pub fn name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    pub fn sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }
    pub fn bpm(&mut self, bpm: u32) {
        self.bpm = bpm;
    }
    pub fn beats_per_bar(&mut self, beats_per_bar: u32) {
        self.beats_per_bar = beats_per_bar;
    }
    pub fn beat_unit(&mut self, beat_unit: u32) {
        self.beat_unit = beat_unit;
    }
    pub fn measure_fallback(&mut self, num: u32) {
        self.num_meassures_fallback = num;
    }
    pub fn sound_frequency_hz(&mut self, sound_frequency_hz: f64) {
        self.sound.frequency_hz = sound_frequency_hz;
    }
    pub fn sound_accent_frequency_hz(&mut self, accent_frequency_hz: f64) {
        self.sound.accent_frequency_hz = accent_frequency_hz;
    }
    pub fn sound_sound_duration_secs(&mut self, sound_duration_secs: f64) {
        self.sound.sound_duration_secs = sound_duration_secs;
    }
    pub fn sound_envelope_decay_secs(&mut self, envelope_decay_secs: f64) {
        self.sound.envelope_decay_secs = envelope_decay_secs;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Sound {
    pub frequency_hz: f64,
    pub accent_frequency_hz: f64,
    pub sound_duration_secs: f64,
    pub envelope_decay_secs: f64,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            frequency_hz: DEFAULT_FREQUENCY_HZ,
            accent_frequency_hz: DEFAULT_ACCENT_FREQUENCY_HZ,
            sound_duration_secs: DEFAULT_SOUND_DURATION_SECS,
            envelope_decay_secs: DEFAULT_ENVELOPE_DECAY_SECS,
        }
    }
}

pub fn try_load_profile(profile_path: &Path, name: &str) -> Option<Config> {
    if name == "default" {
        return Some(Config::default());
    }
    let data = match std::fs::read(profile_path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("[tsci] error loading profile {name}: {err}");
            std::process::exit(1);
        }
    };
    match toml::from_slice::<Config>(&data) {
        Ok(prof) => Some(prof),
        Err(err) => {
            eprintln!("[tsci] error deserializing {name}: {err}");
            std::process::exit(1);
        }
    };
    None
}
