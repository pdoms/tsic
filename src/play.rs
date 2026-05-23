use std::sync::mpsc::Sender;
use std::{
    path::Path,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use rodio::{
    ChannelCount, DeviceSinkBuilder, Player, SampleRate, Source, buffer::SamplesBuffer,
    cpal::BufferSize,
};

use crate::{
    config::Config,
    project::Project,
    section::{Section, TimeSignature},
    visuals::{BeatEvent, print_beat},
};

const BUFFER_SIZE: u32 = 256;

const KEY_TOGGLE_PAUSE: char = ' ';
const KEY_QUIT: char = 'q';
const KEY_NEXT: char = 'n';
const KEY_PREVIOUS: char = 'p';
const KEY_SECTION_START: char = 's';
const KEY_TRACK_START: char = 'b';

const KEYBINDS: [(&str, &str); 6] = [
    ("SPACE", "toggle pause/resume"),
    ("q", "quit"),
    ("n", "go to and start next section"),
    ("p", "go to and start previous section"),
    ("s", "got to beginning of current section"),
    ("b", "got to beginning of the track"),
];

fn print_key_binds() {
    println!("[tsic] Key bindings:\n");
    for (key, explain) in KEYBINDS {
        println!("       >> {:width$} -> {}", key, explain, width = 8);
    }
}

struct SectionSource {
    inner: SamplesBuffer,
    done_tx: Sender<usize>,
    section_index: usize,
    sent: bool,
}

impl SectionSource {
    fn new(buf: Vec<f32>, sample_rate: u32, done_tx: Sender<usize>, section_index: usize) -> Self {
        Self {
            inner: SamplesBuffer::new(
                ChannelCount::new(1).unwrap(),
                SampleRate::new(sample_rate).unwrap(),
                buf,
            ),
            done_tx,
            section_index,
            sent: false,
        }
    }
}

impl Iterator for SectionSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next();
        if sample.is_none() && !self.sent {
            self.done_tx.send(self.section_index).ok();
            self.sent = true;
        }
        sample
    }
}

impl Source for SectionSource {
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

pub fn play(
    sections: &[Section],
    beat_events: &[Vec<BeatEvent>],
    config: &Config,
) -> Result<(), String> {
    print_key_binds();
    println!("\n[tsic] Hit SPACE to start");

    let mut handle = DeviceSinkBuilder::from_default_device()
        .map_err(|err| err.to_string())?
        .with_buffer_size(BufferSize::Fixed(BUFFER_SIZE))
        .open_stream()
        .map_err(|err| err.to_string())?;

    handle.log_on_drop(false);
    let player = Player::connect_new(handle.mixer());
    player.pause();

    let (done_tx, done_rx): (Sender<usize>, Receiver<usize>) = mpsc::channel();

    let num_sections = sections.len();
    let mut current = 0;

    let buf = sections[current].render_with_measures_f32(config);
    player.append(SectionSource::new(
        buf,
        config.sample_rate,
        done_tx.clone(),
        current,
    ));

    let mut event_idx = 0;
    let mut section_start = Instant::now();
    let mut pause_offset = 0.0_f64;
    let mut pause_start: Option<Instant> = Some(Instant::now());

    terminal::enable_raw_mode().map_err(|err| err.to_string())?;

    loop {
        if !player.is_paused()
            && !beat_events.is_empty()
            && let Some(event) = beat_events[current].get(event_idx)
        {
            let elapsed = section_start.elapsed().as_secs_f64() - pause_offset;
            if elapsed >= event.time {
                //for event in &beat_events[0] {
                //    eprintln!("beat {} measure {} time {:.3}", event.beat, event.measure, event.time);
                //}
                //                    eprintln!("beat {} measure {} time {:.3} elapsed {:.3}",
                //                        event.beat, event.measure, event.time, elapsed
                //                        );
                let next_section_event = if current + 1 < num_sections {
                    beat_events[current + 1].first()
                } else {
                    None
                };
                print_beat(event, next_section_event);
                event_idx += 1;
            }
        }
        if done_rx.try_recv().is_ok() && current + 1 < num_sections {
            current += 1;
            let buf = sections[current].render_with_measures_f32(config);
            player.append(SectionSource::new(
                buf,
                config.sample_rate,
                done_tx.clone(),
                current,
            ));
            event_idx = 0;
            section_start = Instant::now();
            pause_offset = 0.0;
            pause_start = None;
        }

        if event::poll(Duration::from_millis(10)).map_err(|err| err.to_string())?
            && let Event::Key(key) = event::read().map_err(|err| err.to_string())?
        {
            match key.code {
                KeyCode::Char(KEY_TOGGLE_PAUSE) => {
                    // toggle pause/play
                    if player.is_paused() {
                        if let Some(ps) = pause_start.take() {
                            pause_offset += ps.elapsed().as_secs_f64();
                        }
                        player.play();
                    } else {
                        pause_start = Some(Instant::now());
                        player.pause();
                    }
                }
                KeyCode::Char(KEY_NEXT) if current + 1 < num_sections => {
                    // go to next section
                    current += 1;
                    player.stop();
                    let buf = sections[current].render_with_measures_f32(config);
                    player.append(SectionSource::new(
                        buf,
                        config.sample_rate,
                        done_tx.clone(),
                        current,
                    ));
                    event_idx = 0;
                    section_start = Instant::now();
                    pause_offset = 0.0;
                    pause_start = None;
                }
                KeyCode::Char(KEY_PREVIOUS) => {
                    // go to previous section
                    current = current.saturating_sub(1);
                    player.stop();
                    let buf = sections[current].render_with_measures_f32(config);
                    player.append(SectionSource::new(
                        buf,
                        config.sample_rate,
                        done_tx.clone(),
                        current,
                    ));
                    event_idx = 0;
                    section_start = Instant::now();
                    pause_offset = 0.0;
                    pause_start = None;
                }
                KeyCode::Char(KEY_SECTION_START) => {
                    // go to start of current section
                    player.stop();
                    let buf = sections[current].render_with_measures_f32(config);
                    player.append(SectionSource::new(
                        buf,
                        config.sample_rate,
                        done_tx.clone(),
                        current,
                    ));
                    event_idx = 0;
                    section_start = Instant::now();
                    pause_offset = 0.0;
                    pause_start = None;
                }
                KeyCode::Char(KEY_TRACK_START) => {
                    // go to track start
                    current = 0;
                    player.stop();
                    let buf = sections[current].render_with_measures_f32(config);
                    player.append(SectionSource::new(
                        buf,
                        config.sample_rate,
                        done_tx.clone(),
                        current,
                    ));
                    event_idx = 0;
                    section_start = Instant::now();
                    pause_offset = 0.0;
                    pause_start = None;
                }
                KeyCode::Char(KEY_QUIT) => {
                    player.stop();
                    break;
                }
                _ => {}
            }
        }
        if player.empty() && current == num_sections - 1 {
            break;
        }
    }

    terminal::disable_raw_mode().map_err(|err| err.to_string())?;
    Ok(())
}

pub fn play_simple(bpm: u32, time_sig: TimeSignature, config: Config) -> Result<(), String> {
    let sample_rate = config.sample_rate;
    let mut project = Project::new("", config, Path::new("."));
    project.sections.push(Section {
        name: None,
        bpm,
        time_signature: time_sig,
        measures: Some(512),
    });
    let buf = project.raw_buffer()?;

    let mut handle = DeviceSinkBuilder::from_default_device()
        .map_err(|err| err.to_string())?
        .with_buffer_size(BufferSize::Fixed(256))
        .open_stream()
        .map_err(|err| err.to_string())?;
    handle.log_on_drop(false);

    let player = Player::connect_new(handle.mixer());

    let buf_f32: Vec<f32> = buf.iter().map(|s| *s as f32 / i16::MAX as f32).collect();
    player.append(SamplesBuffer::new(
        ChannelCount::new(1).unwrap(),
        SampleRate::new(sample_rate).unwrap(),
        buf_f32.clone(),
    ));

    terminal::enable_raw_mode().map_err(|err| err.to_string())?;

    loop {
        if player.empty() {
            player.append(SamplesBuffer::new(
                ChannelCount::new(1).unwrap(),
                SampleRate::new(sample_rate).unwrap(),
                buf_f32.clone(),
            ));
        }
        if event::poll(Duration::from_millis(100)).map_err(|err| err.to_string())?
            && let Event::Key(key) = event::read().map_err(|err| err.to_string())?
        {
            match key.code {
                KeyCode::Char(' ') => {
                    if player.is_paused() {
                        player.play();
                    } else {
                        player.pause();
                    }
                }
                KeyCode::Char('q') => {
                    player.stop();
                    break;
                }
                _ => {}
            }
        }
    }
    terminal::disable_raw_mode().map_err(|err| err.to_string())?;
    Ok(())
}
