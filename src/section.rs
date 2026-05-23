use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{config::Config, snd::click, visuals::BeatEvent};

pub const MIN_IN_SEC: f64 = 60.0;
pub const QUARTER_NOTE: f64 = 4.0; // bpm is defined as quarter notes per minutes

#[derive(Clone, Serialize, Deserialize)]
pub struct Section {
    pub bpm: u32,
    pub time_signature: TimeSignature,
    pub measures: Option<u32>,
    pub name: Option<String>,
}

impl Section {
    pub fn beat_duration(&self) -> f64 {
        MIN_IN_SEC / self.bpm as f64 / (self.time_signature.beat_unit as f64 / QUARTER_NOTE)
    }

    pub fn bpm(&mut self, bpm: u32) {
        self.bpm = bpm;
    }
    pub fn time_signature_str(&mut self, time_sig: &str) -> Result<(), String> {
        self.time_signature = TimeSignature::try_from(time_sig)?;
        Ok(())
    }

    pub fn measures(&mut self, measures: Option<u32>) {
        self.measures = measures;
    }

    pub fn duration_secs(&self) -> f64 {
        let beat_duration = self.beat_duration();
        let measure_duration = beat_duration * self.time_signature.beats_per_bar as f64;
        measure_duration * self.measures.unwrap_or(1) as f64
    }

    pub fn name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// does not handle measures being None
    pub fn render_with_measures_f32(&self, profile: &Config) -> Vec<f32> {
        let duration = self.duration_secs();
        let mut buf = vec![0i16; (duration * profile.sample_rate as f64) as usize];

        let beat_duration = self.beat_duration();
        let num_measures = self.measures.unwrap_or(profile.num_meassures_fallback);
        let mut cursor = 0.0_f64;

        for _ in 0..num_measures {
            for beat in 0..self.time_signature.beats_per_bar {
                let sample_offset = (cursor * profile.sample_rate as f64) as usize;
                let freq = if beat == 0 {
                    profile.sound.accent_frequency_hz
                } else {
                    profile.sound.frequency_hz
                };
                click(
                    &mut buf,
                    sample_offset,
                    freq,
                    profile.sound.envelope_decay_secs,
                    profile.sound.sound_duration_secs,
                    profile.sample_rate as f64,
                );
                cursor += beat_duration;
            }
        }
        buf.iter().map(|s| *s as f32 / i16::MAX as f32).collect()
    }

    pub fn get_beat_events_with_measures(
        &self,
        section_id: usize,
        num_sections: usize,
        profile: &Config,
    ) -> Vec<BeatEvent> {
        let mut events = vec![];

        let mut cursor = 0.0_f64;

        let beat_duration = self.beat_duration();
        let num_measures = self.measures.unwrap_or(profile.num_meassures_fallback);
        for measure in 0..num_measures {
            for beat in 0..self.time_signature.beats_per_bar {
                events.push(BeatEvent {
                    section_name: self.name.clone(),
                    time: cursor,
                    beat,
                    beats_per_bar: self.time_signature.beats_per_bar,
                    measure,
                    num_measures: self.measures.unwrap_or(profile.num_meassures_fallback),
                    section_index: section_id,
                    num_sections,
                    bpm: self.bpm,
                    time_sig: self.time_signature.clone(),
                });
                cursor += beat_duration;
            }
        }
        events
    }
}

/// beats_per_bar/beat_unit
#[derive(Clone, Serialize, Deserialize)]
pub struct TimeSignature {
    pub beats_per_bar: u32,
    pub beat_unit: u32,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            beats_per_bar: 4,
            beat_unit: 4,
        }
    }
}

impl TryFrom<&str> for TimeSignature {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split_once("/") {
            Some((bpb, bu)) => match (bpb.parse::<u32>(), bu.parse::<u32>()) {
                (Ok(bpb), Ok(bu)) => Ok(TimeSignature {
                    beats_per_bar: bpb,
                    beat_unit: bu,
                }),
                _ => Err(format!("[tsic] error parsing {value} to TimeSignature")),
            },
            None => Err(format!("[tsic] error parsing {value} to TimeSignature")),
        }
    }
}

impl Display for TimeSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.beats_per_bar, self.beat_unit)
    }
}
