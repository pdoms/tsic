mod args;
mod config;
mod midi;
mod play;
mod project;
mod section;
mod snd;
mod tap;
mod visuals;

use std::path::Path;

use clap::Parser;

use crate::{
    args::Arguments, config::{try_load_profile, Config, PROFILE_DIR, PROJECTS_DIR}, midi::MidiConfigs, play::{play, play_simple}, project::Project, section::TimeSignature, tap::tap_temp
};

fn main() {
    let profiles_path = std::path::Path::new(PROFILE_DIR);
    let _ = std::fs::create_dir_all(profiles_path)
        .map_err(|err| eprintln!("[tsic] error creating {PROFILE_DIR} - {err}"));
    let projects_path = std::path::Path::new(PROJECTS_DIR);
    let _ = std::fs::create_dir_all(projects_path)
        .map_err(|err| eprintln!("[tsic] error creating {PROJECTS_DIR} - {err}"));

    let cli = Arguments::parse();

    let mut init_profile = Config::default();

    match cli.command {
        args::Cmd::Defaults => {
            println!("[tsic] defaults:\n{init_profile}\n")
        }
        args::Cmd::Profile {
            name,
            sr,
            bpm,
            bpb,
            bu,
            freq,
            acc,
            dur,
            dec,
            fb,
        } => {
            init_profile.name(name.as_str());
            // try loading it
            let profile_path_current = profiles_path.join(format!("{name}.toml"));

            let profile_exists = std::fs::exists(&profile_path_current).is_ok_and(|b| b);
            if profile_exists {
                init_profile = try_load_profile(&profile_path_current, name.as_str()).unwrap();
            }
            let mut has_updates = false;

            if let Some(sr) = sr {
                init_profile.sample_rate(sr);
                has_updates = true;
            }
            if let Some(bpm) = bpm {
                init_profile.bpm(bpm);
                has_updates = true;
            }
            if let Some(bpb) = bpb {
                init_profile.beats_per_bar(bpb);
                has_updates = true;
            }
            if let Some(bu) = bu {
                init_profile.beat_unit(bu);
                has_updates = true;
            }
            if let Some(freq) = freq {
                init_profile.sound_frequency_hz(freq);
                has_updates = true;
            }
            if let Some(fallback) = fb {
                init_profile.measure_fallback(fallback);
                has_updates = true;
            }
            if let Some(acc) = acc {
                init_profile.sound_accent_frequency_hz(acc);
                has_updates = true;
            }
            if let Some(dur) = dur {
                init_profile.sound_sound_duration_secs(dur);
                has_updates = true;
            }
            if let Some(dec) = dec {
                init_profile.sound_envelope_decay_secs(dec);
                has_updates = true;
            }

            if profile_exists && !has_updates {
                // this is the case we have to consider when
            }
            //write to file
            let data = match toml::to_string_pretty(&init_profile) {
                Ok(s) => s,
                Err(err) => {
                    eprintln!("[tsic] - serializing profile {name}: {err}");
                    std::process::exit(1);
                }
            };
            match std::fs::write(&profile_path_current, data) {
                Ok(_) => println!(
                    "[tsic] saved profile to {}",
                    profile_path_current.to_str().unwrap()
                ),
                Err(err) => {
                    eprintln!("[tsic] - error writing profile {name}: {err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::New { name, profile } => {
            //TODO check if it exists!
            let use_profile = if &profile == "default" {
                init_profile
            } else if let Some(prof) =
                try_load_profile(&profiles_path.join(format!("{profile}.toml")), &profile)
            {
                prof
            } else {
                init_profile
            };
            println!("[tsic] loaded profile '{}'", profile);
            let file_path = projects_path.join(format!("{name}.toml"));
            let project = Project::new(name.as_str(), use_profile, &file_path);
            if let Err(err) = project.to_disk() {
                eprintln!("[tsic] - error writing project {name}: {err}");
                std::process::exit(1);
            }
        }
        args::Cmd::Append {
            name,
            bpm,
            time_sig,
            measures,
        } => {
            let file_path = projects_path.join(format!("{name}.toml"));
            match Project::from_disk(&file_path) {
                Ok(mut project) => {
                    if let Err(err) = project.append_section(bpm, time_sig, measures) {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                    if let Err(err) = project.to_disk() {
                        eprintln!("[tsic] - error writing project {name}: {err}");
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::Insert {
            name,
            position,
            bpm,
            time_sig,
            measures,
        } => {
            let file_path = projects_path.join(format!("{name}.toml"));
            match Project::from_disk(&file_path) {
                Ok(mut project) => {
                    if let Err(err) = project.insert_section_at(position, measures, bpm, time_sig) {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                    println!("[tsic] inserted section at position {position}");
                    if let Err(err) = project.to_disk() {
                        eprintln!("[tsic] - error writing project {name}: {err}");
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::Edit {
            name,
            position,
            bpm,
            time_sig,
            measures,
        } => {
            let file_path = projects_path.join(format!("{name}.toml"));
            match Project::from_disk(&file_path) {
                Ok(mut project) => {
                    if let Err(err) = project.edit_section(position, measures, bpm, time_sig) {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                    println!("[tsic] edited section at position {position}");
                    if let Err(err) = project.to_disk() {
                        eprintln!("[tsic] - error writing project {name}: {err}");
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::RemoveSection { name, position } => {
            let file_path = projects_path.join(format!("{name}.toml"));
            match Project::from_disk(&file_path) {
                Ok(mut project) => {
                    if project.remove_section(position) {
                        println!("[tsic] removed section from position {position}");
                    } else {
                        println!(
                            "[tsic] skipped removing section from position {position} (out of bounds)"
                        );
                    }
                    if let Err(err) = project.to_disk() {
                        eprintln!("[tsic] - error writing project {name}: {err}");
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::ProfilesList => list_entities(profiles_path, "profiles"),
        args::Cmd::ProjectsList => list_entities(projects_path, "projects"),
        args::Cmd::Project { name } => {
            let file_path = projects_path.join(format!("{name}.toml"));
            match Project::from_disk(&file_path) {
                Ok(project) => {
                    println!("[tsic] project\n{}", project);
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::Wav { name, outfile } => {
            let file_path = projects_path.join(format!("{name}.toml"));
            match Project::from_disk(&file_path) {
                Ok(mut project) => {
                    let file_name = outfile
                        .unwrap_or(Path::new(format!("./{name}.wav").as_str()).to_path_buf());
                    if let Err(err) = project.write_wav(&file_name) {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::Midi {
            name,
            outfile,
            channel,
            ticks_per_beat,
            accent,
            normal,
            duration,
            vel_accent,
            vel_normal,
        } => {
            let file_path = projects_path.join(format!("{name}.toml"));
            match Project::from_disk(&file_path) {
                Ok(project) => {
                    let file_name = outfile
                        .unwrap_or(Path::new(format!("./{name}.mid").as_str()).to_path_buf());

                    let mut midi_configs = MidiConfigs::default();
                    if let Some(ch) = channel {
                        midi_configs.channel(ch);
                    }
                    if let Some(tpb) = ticks_per_beat {
                        midi_configs.ticks_per_beat(tpb);
                    }
                    if let Some(acc) = accent {
                        midi_configs.note_accent(acc);
                    }
                    if let Some(nor) = normal {
                        midi_configs.note_normal(nor);
                    }
                    if let Some(dur) = duration {
                        midi_configs.duration(dur);
                    }
                    if let Some(vel_acc) = vel_accent {
                        midi_configs.velocity_accent(vel_acc);
                    }
                    if let Some(vel_nor) = vel_normal {
                        midi_configs.velocity_normal(vel_nor);
                    }

                    if let Err(err) = project.write_midi(&midi_configs, &file_name) {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::Play { name, visualize } => {
            let file_path = projects_path.join(format!("{name}.toml"));
            match Project::from_disk(&file_path) {
                Ok(mut project) => {
                    if visualize {
                        project.start_events();
                    }

                    match project.raw_buffer() {
                        Ok(buffer) => {
                            if let Err(err) =
                                play(buffer, project.profile.sample_rate, project.events)
                            {
                                eprintln!("{err}");
                                std::process::exit(1);
                            }
                        }
                        Err(err) => {
                            eprintln!("{err}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        args::Cmd::Tap => {
            if let Err(err) = tap_temp() {
                eprintln!("{err}");
                std::process::exit(1);
            }
        },
        args::Cmd::Metronome { bpm, signature, profile } => {
            let use_profile = if let Some(name) = profile {
                // try loading it
                let profile_path_current = profiles_path.join(format!("{name}.toml"));

                let profile_exists = std::fs::exists(&profile_path_current).is_ok_and(|b| b);
                if profile_exists {
                    try_load_profile(&profile_path_current, name.as_str()).unwrap()
                } else {
                    eprintln!("[tsic] error: profile name was provided, but the profile {} could not be found", profile_path_current.to_str().unwrap());
                    std::process::exit(1);
                }
            } else {
                init_profile
            };

            let time_sig = if let Some(ts) = signature {
                match TimeSignature::try_from(ts.as_str()) {
                    Ok(ts) => ts,
                    Err(err) =>  {
                eprintln!("{err}");
                std::process::exit(1);
                    }
                }
            } else {
                use_profile.get_time_signature()
            };
            let use_bpm = if let Some(bs) = bpm {
                bs
            } else {
                use_profile.bpm
            };

            if let Err(err) = play_simple(use_bpm, time_sig, use_profile) {
                eprintln!("{err}");
                std::process::exit(1);
            }

        }
    }
}

fn list_entities(entity_path: &Path, msg: &str) {
    let read_dir = match std::fs::read_dir(entity_path) {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!(
                "[tsic]: listing profiles from {}: {err}",
                entity_path.to_str().unwrap()
            );
            std::process::exit(1);
        }
    };

    println!("[tsic] {msg}:");
    for entry in read_dir {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            println!(
                "  + {} at {}",
                entry.path().file_stem().unwrap().to_str().unwrap(),
                entry.path().to_str().unwrap()
            );
        }
    }
}
