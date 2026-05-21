use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Print the default settings
    Defaults,
    /// Create a Profile - for default values, run the
    /// 'defaults' command
    Profile {
        /// The name of the profile.
        /// it will be stored as ./profiles/<NAME>.toml,
        /// If profile does not exist, and no optional
        /// arguments are provided, default arguments are stored.
        /// If it already exists, it will be updated.
        /// If profile does exist and no arguments are provided,
        /// the profile will be deleted.
        name: String,
        /// Sample Rate
        #[arg(long)]
        sr: Option<u32>,
        /// Beats Per Minute
        #[arg(long)]
        bpm: Option<u32>,
        /// Beats Per Bar
        #[arg(long)]
        bpb: Option<u32>,
        /// Beats Unit
        #[arg(long)]
        bu: Option<u32>,
        /// Base Click Frequency in hz (decimal)
        #[arg(long)]
        freq: Option<f64>,
        /// Accent Click Frequency in hz (decimal)
        #[arg(long)]
        acc: Option<f64>,
        /// Duration in seconds (decimal) of the click
        #[arg(long)]
        dur: Option<f64>,
        /// Duration of the envelope decay in seconds (decimal)
        #[arg(long)]
        dec: Option<f64>,
        /// A fallback for sections without number of measurements
        #[arg(long)]
        fb: Option<u32>,
    },
    /// Lists all available profile names
    ProfilesList,
    /// Lists all available project names
    ProjectsList,
    /// Prints the project of the provided name to stdout.
    /// Note: the profile is not printed.
    Project { name: String },
    /// Creates a new project
    New {
        name: String,
        #[arg(short, long, default_value_t = String::from("default"))]
        profile: String,
    },
    /// append a section to the current project.
    /// Except for <measures>, omitted values are taken from the previous
    /// section. If there is  no previous
    /// section, it defaults to profile values
    Append {
        name: String,
        /// Beats Per Minute
        #[arg(long, short)]
        bpm: Option<u32>,
        /// Time Signature
        #[arg(long, short)]
        time_sig: Option<String>,
        /// Number of Measures this section has
        /// If the section before the new section is open ended,
        /// this will throw an error
        #[arg(long, short)]
        measures: Option<u32>,
    },
    /// insert a section at the provided index.
    /// Omitted values are taken from the previous
    /// section. If there is  no previous
    /// section it defaults to profile defaults.
    Insert {
        name: String,
        /// the index at which the section should be inserted
        #[arg(long, short)]
        position: usize,
        /// Beats Per Minute
        #[arg(long, short)]
        bpm: Option<u32>,
        /// Time Signature
        #[arg(long, short)]
        time_sig: Option<String>,
        /// Number of Measures this section has
        #[arg(long, short)]
        measures: u32,
    },
    /// Edit a section
    Edit {
        name: String,
        /// the index/position of the section to be edited
        #[arg(long, short)]
        position: usize,
        /// Beats Per Minute
        #[arg(long, short)]
        bpm: Option<u32>,
        /// Time Signature
        #[arg(long, short)]
        time_sig: Option<String>,
        /// Number of Measures this section has
        #[arg(long, short)]
        measures: Option<u32>,
    },
    /// Remove a section
    RemoveSection {
        name: String,
        /// the index/position of the section to be edited
        #[arg(long, short)]
        position: usize,
    },
    Wav {
        /// name of the project
        name: String,
        /// provide path or name to outfile. Defaults to "./Wav.name.wav"
        outfile: Option<PathBuf>,
    },
    /// writes the project to disk in the midi format.
    /// Midi defaults can be printed but not used in
    /// a profile, so values are provided here
    Midi {
        /// name of the project
        name: String,
        /// provide path or name to outfile. Defaults to "./Wav.name.wav"
        outfile: Option<PathBuf>,
        /// midi channel (0-15 - zero-indexed)
        #[arg(long, short)]
        channel: Option<u8>,
        /// ticks per beat (quarter note)
        #[arg(long, short)]
        ticks_per_beat: Option<u16>,
        /// accent note
        #[arg(long, short)]
        accent: Option<u8>,
        /// normal (click) note
        #[arg(long, short)]
        normal: Option<u8>,
        /// how the note is held
        #[arg(long, short)]
        duration: Option<u32>,
        /// velocity accent
        #[arg(long)]
        vel_accent: Option<u8>,
        /// velocity normal
        #[arg(long)]
        vel_normal: Option<u8>,
    },
    Play {
        name: String,
        /// whether to run visualization while playing
        #[arg(long, short, default_value_t = false)]
        visualize: bool,
    },
    /// Tab with the space bar a temp and display it in
    /// the terminal
    Tap,
    /// Provide bpm, signature and a config and have the mentronome ticking away
    Metronome {
        /// the desired bpm
        #[arg(short, long)]
        bpm: Option<u32>,
        /// time signature (e.g. 4/4) can be provided or taken from the a profile
        #[arg(short, long)]
        signature: Option<String>,
        /// provide a profile as a template. If not provided, it falls back to
        /// the default profile.
        #[arg(short, long)]
        profile: Option<String>,
    },
}
