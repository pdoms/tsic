use std::f64::consts::PI;

pub fn click(buf: &mut [i16], offset: usize, freq: f64, decay: f64, duration: f64, sr: f64) {
    let num_samples = (duration * sr) as usize;
    for i in 0..num_samples {
        let t = i as f64 / sr;
        let envelope = (-t / decay).exp();
        let sample = envelope * (2.0 * PI * freq * t).sin();
        buf[offset + i] = (sample * i16::MAX as f64) as i16;
    }
}
