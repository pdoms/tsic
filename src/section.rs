use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub const MIN_IN_SEC: f64 = 60.0;
pub const QUARTER_NOTE: f64 = 4.0; // bpm is defined as quarter notes per minutes

#[derive(Clone, Serialize, Deserialize)]
pub struct Section {
    pub bpm: u32,
    pub time_signature: TimeSignature,
    pub measures: Option<u32>,
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
