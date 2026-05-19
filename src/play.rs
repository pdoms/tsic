use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread, time::{Duration, Instant}
};

use crossterm::terminal;
use rodio::{ChannelCount, DeviceSinkBuilder, SampleRate, buffer::SamplesBuffer};

use crate::visuals::{BeatEvent, print_beat};

pub fn play(buf: Vec<i16>, sample_rate: u32, events: Vec<BeatEvent>) -> Result<(), String> {

    println!("[tsic] controls: Pause/Resume with 'space' | Quit with 'q'");

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

    let pause_offset = Arc::new(Mutex::new(0.0_f64));
    let pause_offset_visual = Arc::clone(&pause_offset);
    let pause_start = Arc::new(Mutex::new(None::<Instant>));
    let stop = Arc::new(AtomicBool::new(false));
    let stop_visual = Arc::clone(&stop);
    let paused = Arc::new(AtomicBool::new(false));
    let paused_visual = Arc::clone(&paused);

    let visual_handle = if !events.is_empty() {
        let start = Instant::now();



        Some(thread::spawn(move || {
            for (i, event) in events.iter().enumerate() {
                loop {
                    if stop_visual.load(Ordering::Relaxed) {
                        return;
                    }
                    if !paused_visual.load(Ordering::Relaxed) {
                        let elapsed = start.elapsed().as_secs_f64() - *pause_offset_visual.lock().unwrap();
                        let wait = event.time - elapsed;
                        if wait <= 0.0 {
                            break;
                        }
                    }

                }
                if stop_visual.load(Ordering::Relaxed) {
                    return;
                }
                print_beat(event, events.get(i + 1));
            }
        }))
    }
    else {
        None
    };

    // input thread 
    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    loop {
        if crossterm::event::poll(Duration::from_millis(100)).map_err(|e| e.to_string())? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read().map_err(|e| e.to_string())? {
                match key.code {
                crossterm::event::KeyCode::Char(' ') => {
                    if player.is_paused() {
                        if let Some(ps) = pause_start.lock().unwrap().take() {
                            *pause_offset.lock().unwrap() += ps.elapsed().as_secs_f64();
                        }
                        player.play();
                        paused.store(false, Ordering::Relaxed);
                    } else {
                        *pause_start.lock().unwrap() = Some(Instant::now());
                        player.pause();
                        paused.store(true, Ordering::Relaxed);
                    }
                }
                crossterm::event::KeyCode::Char('q') => {
                    player.stop();
                    break;
                }
                _ => {}
                }
            }
        }
        if player.empty() {
            break;
        }
    }

    stop.store(true, Ordering::Relaxed);
    if let Some(handle) = visual_handle {
        handle.join().unwrap();
    }
    terminal::disable_raw_mode().map_err(|e| e.to_string())?;

    Ok(())
}
