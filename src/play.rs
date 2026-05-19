use std::{
    thread,
    time::{Duration, Instant},
};

use rodio::{ChannelCount, DeviceSinkBuilder, SampleRate, buffer::SamplesBuffer};

use crate::visuals::{BeatEvent, print_beat};

pub fn play(buf: Vec<i16>, sample_rate: u32, events: Vec<BeatEvent>) -> Result<(), String> {
    let mut handle = DeviceSinkBuilder::open_default_sink().map_err(|e| e.to_string())?;
    handle.log_on_drop(false);
    let player = rodio::Player::connect_new(handle.mixer());
    let source = SamplesBuffer::new(
        ChannelCount::new(1u16).unwrap(),
        SampleRate::new(sample_rate).unwrap(),
        buf.iter()
            .map(|s| *s as f32 / i16::MAX as f32)
            .collect::<Vec<f32>>(),
    );
    player.append(source);

    if !events.is_empty() {
        let start = Instant::now();
        thread::spawn(move || {
            for (i, event) in events.iter().enumerate() {
                let wait = event.time - start.elapsed().as_secs_f64();
                if wait > 0.0 {
                    thread::sleep(Duration::from_secs_f64(wait));
                }
                print_beat(event, events.get(i + 1));
            }
        });
    }

    player.sleep_until_end();
    Ok(())
}
