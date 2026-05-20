use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event},
    terminal,
};
use rodio::{ChannelCount, DeviceSinkBuilder, SampleRate, buffer::SamplesBuffer};

use crate::{
    config::{
        DEFAULT_ENVELOPE_DECAY_SECS, DEFAULT_FREQUENCY_HZ, DEFAULT_SAMPLE_RATE,
        DEFAULT_SOUND_DURATION_SECS,
    },
    snd::click,
};

const MAX_TAPS: usize = 12;

pub fn tap_temp() -> Result<(), String> {
    let mut tap_times: VecDeque<Instant> = VecDeque::with_capacity(MAX_TAPS);

    println!("[tsic] Tap SPACE to the beat, 'q' to quit\r\n");

    let mut handle = DeviceSinkBuilder::from_default_device()
        .map_err(|e| e.to_string())?
        .with_buffer_size(rodio::cpal::BufferSize::Fixed(256))
        .open_stream()
        .map_err(|e| e.to_string())?;
    handle.log_on_drop(false);
    let sample_rate = DEFAULT_SAMPLE_RATE as f64;
    let player = rodio::Player::connect_new(handle.mixer());
    let mut click_buf = vec![0i16; (sample_rate * DEFAULT_SOUND_DURATION_SECS) as usize];
    click(
        &mut click_buf,
        0,
        DEFAULT_FREQUENCY_HZ,
        DEFAULT_ENVELOPE_DECAY_SECS,
        DEFAULT_SOUND_DURATION_SECS,
        sample_rate,
    );
    let buf_f32: Vec<f32> = click_buf
        .iter()
        .map(|s| *s as f32 / i16::MAX as f32)
        .collect();

    terminal::enable_raw_mode().map_err(|err| err.to_string())?;

    loop {
        if event::poll(Duration::from_millis(100)).map_err(|e| e.to_string())?
            && let Event::Key(key) = event::read().map_err(|err| err.to_string())?
        {
            match key.code {
                event::KeyCode::Char('q') => {
                    break;
                }
                event::KeyCode::Char(' ') => {
                    player.append(SamplesBuffer::new(
                        ChannelCount::new(1u16).unwrap(),
                        SampleRate::new(sample_rate as u32).unwrap(),
                        buf_f32.clone(),
                    ));

                    let now = Instant::now();

                    if let Some(last) = tap_times.back()
                        && last.elapsed().as_secs_f64() > 3.0
                    {
                        tap_times.clear();
                    }

                    if tap_times.len() == MAX_TAPS {
                        tap_times.pop_front();
                    }

                    tap_times.push_back(now);

                    if tap_times.len() > 1 {
                        let intervals: Vec<f64> = tap_times
                            .iter()
                            .zip(tap_times.iter().skip(1))
                            .map(|(a, b)| b.duration_since(*a).as_secs_f64())
                            .collect();
                        let avg = intervals.iter().sum::<f64>() / intervals.len() as f64;
                        let bpm = (60.0 / avg).round() as u32;
                        print!("\r\x1B[2K  taps: {}    BPM: {}  ", tap_times.len(), bpm);
                    } else {
                        print!("\r\x1B[2K  taps: 1    BPM: --  ");
                    }

                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
                _ => {}
            }
        }
    }
    terminal::disable_raw_mode().map_err(|err| err.to_string())?;
    if tap_times.len() > 1 {
        let intervals: Vec<f64> = tap_times
            .iter()
            .zip(tap_times.iter().skip(1))
            .map(|(a, b)| b.duration_since(*a).as_secs_f64())
            .collect();
        let avg = intervals.iter().sum::<f64>() / intervals.len() as f64;
        let bpm = (60.0 / avg).round() as u32;
        print!("\r\n Final BPM: {}  ", bpm);
    }
    Ok(())
}
