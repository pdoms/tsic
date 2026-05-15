use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    config::{Config, PROFILE_DIR, try_load_profile},
    section::{Section, TimeSignature},
    snd::click,
};

pub struct Project {
    pub name: String,
    pub profile: Config,
    pub file_path: PathBuf,
    pub sections: Vec<Section>,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[name]         {}", self.name)?;
        writeln!(f, "[profile]      {}", self.profile.name)?;
        writeln!(f, "[file_path]    {}", self.file_path.to_str().unwrap())?;
        writeln!(f, "[sections]")?;

        for (i, section) in self.sections.iter().enumerate() {
            writeln!(
                f,
                "  [{i}] bpm: {}; time_signature: {}, measures: {}",
                section.bpm,
                section.time_signature,
                section.measures.map(|v| v as i32).unwrap_or(-1)
            )?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProjectFile {
    pub name: String,
    pub profile: String,
    pub file_path: PathBuf,
    pub sections: Vec<Section>,
}

impl Project {
    pub fn new(name: &str, profile: Config, file_path: &Path) -> Self {
        Self {
            name: name.to_string(),
            profile,
            file_path: file_path.to_path_buf(),
            sections: vec![],
        }
    }

    pub fn append_section(
        &mut self,
        bpm: Option<u32>,
        time_sig: Option<String>,
        measures: Option<u32>,
    ) -> Result<(), String> {
        let section = if let Some(last) = self.sections.last() {
            if last.measures.is_none() {
                return Err(format!(
                    "[tsic] illegal operation. Section at index: {} has no measurement. Appending a new section will have no effect",
                    self.sections.len() - 1
                ));
            }
            let time_signature = if let Some(ts_s) = time_sig {
                TimeSignature::try_from(ts_s.as_str())?
            } else {
                last.time_signature.clone()
            };
            Section {
                bpm: bpm.unwrap_or(last.bpm),
                time_signature,
                measures,
            }
        } else {
            let time_signature = if let Some(ts_s) = time_sig {
                TimeSignature::try_from(ts_s.as_str())?
            } else {
                TimeSignature {
                    beats_per_bar: self.profile.beats_per_bar,
                    beat_unit: self.profile.beat_unit,
                }
            };
            Section {
                bpm: bpm.unwrap_or(self.profile.bpm),
                time_signature,
                measures,
            }
        };
        self.sections.push(section);
        println!(
            "[tsic] appended section - new num of sections: {}",
            self.sections.len()
        );

        Ok(())
    }

    pub fn from_disk(file_path: &Path) -> Result<Self, String> {
        let data = std::fs::read(file_path).map_err(|err| err.to_string())?;
        let parsed = toml::from_slice::<ProjectFile>(&data).map_err(|err| err.to_string())?;
        let profiles_path = std::path::Path::new(PROFILE_DIR);
        let profile_path = profiles_path.join(format!("{}.toml", parsed.profile).as_str());
        let profile = try_load_profile(&profile_path, &parsed.profile).unwrap_or_else(|| {
            panic!(
                "[tsic] Profile at {} not found.",
                profile_path.to_str().unwrap()
            )
        });

        Ok(Project {
            name: parsed.name,
            profile,
            file_path: file_path.to_path_buf(),
            sections: parsed.sections,
        })
    }
    pub fn to_disk(&self) -> Result<(), String> {
        let file_data = ProjectFile {
            name: self.name.clone(),
            profile: self.profile.name.clone(),
            file_path: self.file_path.clone(),
            sections: self.sections.clone(),
        };

        let file_str = toml::to_string(&file_data)
            .map_err(|err| format!("[tsic] could not serialize project: {err}"))?;
        std::fs::write(&self.file_path, file_str.as_bytes()).map_err(|err| err.to_string())?;
        println!(
            "[tsic] project {} saved to {}",
            self.name,
            self.file_path.to_str().unwrap()
        );
        Ok(())
    }

    pub fn to_wav(&self, outpath: &Path) -> Result<(), String> {
        let total_duration: f64 = self
            .sections
            .iter()
            .map(|section| {
                let beat_duration = section.beat_duration();
                let measure_duration = beat_duration * section.time_signature.beats_per_bar as f64;
                measure_duration * section.measures.unwrap_or(1) as f64
            })
            .sum();
        let buf_size = (total_duration * self.profile.sample_rate as f64) as usize;
        let mut buf = vec![0i16; buf_size];

        let is_one = self.sections.len() == 1;

        let mut cursor = 0.0_f64;

        for (section_id, section) in self.sections.iter().enumerate() {
            let beat_duration = section.beat_duration();
            let num_measures = if let Some(measures) = section.measures {
                measures
            } else if is_one {
                self.profile.num_meassures_fallback
            } else {
                return Err(format!(
                    "[tsic] this project has more than one section and section {section_id} has no number of measurements defined."
                ));
            };
            for _measure in 0..num_measures {
                for beat in 0..section.time_signature.beats_per_bar {
                    let sample_offset = (cursor * self.profile.sample_rate as f64) as usize;
                    let freq = if beat == 0 {
                        self.profile.sound.accent_frequency_hz
                    } else {
                        self.profile.sound.frequency_hz
                    };
                    click(
                        &mut buf,
                        sample_offset,
                        freq,
                        self.profile.sound.envelope_decay_secs,
                        self.profile.sound.sound_duration_secs,
                        self.profile.sample_rate as f64,
                    );
                    cursor += beat_duration;
                }
            }
        }

        println!("[tsic] prepared buffer");

        //TODO make this either part of the profile
        //or as args
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.profile.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(outpath, spec).map_err(|err| err.to_string())?;

        for sample in &buf {
            writer
                .write_sample(*sample)
                .map_err(|err| err.to_string())?;
        }
        writer.finalize().map_err(|err| err.to_string())?;
        println!("[tsic] wrote {}", outpath.to_str().unwrap());

        Ok(())
    }
}
