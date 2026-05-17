use std::path::Path;

use midly::{Format::SingleTrack, Header, Smf, Timing::Metrical, Track};

use crate::{
    config::Config,
    section::{MIN_IN_SEC, QUARTER_NOTE, Section},
};

const DEFAULT_TICKS_PER_BEAT: u16 = 480;
const DEFAULT_DRUM_CHANNEL: u8 = 9;
const DEFAULT_NOTE_ACCENT: u8 = 76; // high wood block
const DEFAULT_NOTE_NORMAL: u8 = 37; // side stick
const DEFAULT_NOTE_DURATION_TICKS: u32 = 10; // how long the note is held
const DEFAULT_VELOCITY_ACCENT: u8 = 127;
const DEFAULT_VELOCITY_NORMAL: u8 = 90;

const SIXTY_SECONDS_IN_MICROSECONDS: f64 = MIN_IN_SEC * 1_000_000.0;

pub struct MidiConfigs {
    /// midi channel (0-15 - zero-indexed)
    channel: u8,
    /// ticks per beat (quarter note)
    ticks_per_beat: u16,
    /// accent note
    note_accent: u8,
    /// normal (click) note
    note_normal: u8,
    /// how the note is held
    duration: u32,
    /// velocity accent
    velocity_accent: u8,
    /// velocity normal
    velocity_normal: u8,
}

impl Default for MidiConfigs {
    fn default() -> Self {
        Self {
            channel: DEFAULT_DRUM_CHANNEL,
            ticks_per_beat: DEFAULT_TICKS_PER_BEAT,
            note_accent: DEFAULT_NOTE_ACCENT,
            note_normal: DEFAULT_NOTE_NORMAL,
            duration: DEFAULT_NOTE_DURATION_TICKS,
            velocity_accent: DEFAULT_VELOCITY_ACCENT,
            velocity_normal: DEFAULT_VELOCITY_NORMAL,
        }
    }
}

impl MidiConfigs {
    pub fn channel(&mut self, v: u8) {
        self.channel = v;
    }
    pub fn ticks_per_beat(&mut self, v: u16) {
        self.ticks_per_beat = v;
    }
    pub fn note_accent(&mut self, v: u8) {
        self.note_accent = v;
    }
    pub fn note_normal(&mut self, v: u8) {
        self.note_normal = v;
    }
    pub fn duration(&mut self, v: u32) {
        self.duration = v;
    }
    pub fn velocity_accent(&mut self, v: u8) {
        self.velocity_accent = v;
    }
    pub fn velocity_normal(&mut self, v: u8) {
        self.velocity_normal = v;
    }
}

pub fn write_midi(
    sections: &[Section],
    midi_config: &MidiConfigs,
    profile: &Config,
    path: &Path,
) -> Result<(), String> {
    let mut track: Track = vec![];
    let mut last_tick: u32 = 0;
    #[allow(unused)]
    let mut cursor_secs: f64 = 0.0;
    let mut cursor_ticks: u32 = 0;
    let mut last_bpm: Option<u32> = None;

    for section in sections {
        let bpm = section.bpm as f64;
        let measures = section.measures.unwrap_or(profile.num_meassures_fallback);
        let time_sig = &section.time_signature;
        let beat_dur_secs = section.beat_duration();
        let beat_dur_ticks = beat_duration_ticks(time_sig.beat_unit, midi_config.ticks_per_beat);

        if last_bpm != Some(section.bpm) {
            let tempo = (SIXTY_SECONDS_IN_MICROSECONDS / bpm) as u32;
            let delta = cursor_ticks.saturating_sub(last_tick);
            track.push(midly::TrackEvent {
                delta: delta.into(),
                kind: midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo.into())),
            });
            last_tick = cursor_ticks;
        }

        for _ in 0..measures {
            for beat in 0..time_sig.beats_per_bar {
                let (note, vel) = if beat == 0 {
                    (midi_config.note_accent, midi_config.velocity_accent)
                } else {
                    (midi_config.note_normal, midi_config.velocity_normal)
                };
                let note_on_tick = cursor_ticks;
                cursor_ticks += beat_dur_ticks;

                //NoteOn
                let delta = note_on_tick.saturating_sub(last_tick);
                track.push(midly::TrackEvent {
                    delta: delta.into(),
                    kind: midly::TrackEventKind::Midi {
                        channel: midi_config.channel.into(),
                        message: midly::MidiMessage::NoteOn {
                            key: note.into(),
                            vel: vel.into(),
                        },
                    },
                });

                //Note off
                track.push(midly::TrackEvent {
                    delta: midi_config.duration.into(),
                    kind: midly::TrackEventKind::Midi {
                        channel: midi_config.channel.into(),
                        message: midly::MidiMessage::NoteOff {
                            key: note.into(),
                            vel: 0.into(),
                        },
                    },
                });
                last_tick = note_on_tick + midi_config.duration;
                cursor_secs += beat_dur_secs;
            }
        }
        last_bpm = Some(section.bpm);
    }

    track.push(midly::TrackEvent {
        delta: 0.into(),
        kind: midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });

    let smf = Smf {
        header: Header {
            format: SingleTrack,
            timing: Metrical(midi_config.ticks_per_beat.into()),
        },
        tracks: vec![track],
    };
    smf.save(path).map_err(|err| err.to_string())?;
    println!("[tsic] MIDE file written to '{}'", path.to_str().unwrap());
    Ok(())
}

fn beat_duration_ticks(beat_unit: u32, ticks_per_beat: u16) -> u32 {
    let quarter_note_ticks = ticks_per_beat as f64;
    (quarter_note_ticks * (QUARTER_NOTE / beat_unit as f64)) as u32
}
