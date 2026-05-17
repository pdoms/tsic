use rodio::{buffer::SamplesBuffer, ChannelCount, DeviceSinkBuilder, SampleRate};

pub fn play(buf: Vec<i16>, sample_rate: u32) -> Result<(), String> {
    let handle = DeviceSinkBuilder::open_default_sink().map_err(|e| e.to_string())?;
    let player = rodio::Player::connect_new(&handle.mixer());
    let source = SamplesBuffer::new(ChannelCount::new(1u16).unwrap(), SampleRate::new(sample_rate).unwrap(), buf.iter().map(|s| *s as f32 / i16::MAX as f32).collect::<Vec<f32>>());
    player.append(source);
    player.sleep_until_end();
    Ok(())

}
