use crate::section::TimeSignature;

pub struct BeatEvent {
    pub time: f64,
    pub beat: u32,
    pub beats_per_bar: u32,
    pub measure: u32,
    pub num_measures: u32,
    pub section_index: usize,
    pub num_sections: usize,
    pub bpm: u32,
    pub time_sig: TimeSignature,
}

pub fn print_beat(event: &BeatEvent, next_event: Option<&BeatEvent>) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if event.beat == 0 && event.measure == 0 {
        let next = if let Some(next_event) = next_event {
            format!(
                " -> [Next: {} | {} BPM]",
                next_event.time_sig, next_event.bpm
            )
        } else {
            String::new()
        };
        writeln!(
            out,
            "\n[Section {}/{} | {}/{} | {} BPM]{}",
            event.section_index + 1,
            event.num_sections,
            event.time_sig.beats_per_bar,
            event.time_sig.beat_unit,
            event.bpm,
            next
        )
        .unwrap();
        writeln!(out).unwrap();
        writeln!(out).unwrap();
    }

    let mut bar = format!("  bar {}/{} | ", event.measure + 1, event.num_measures);
    let prefix_len = bar.len();

    for i in 0..event.beats_per_bar {
        let ch = if i == 0 {
            'X'
        } else if i <= event.beat { '+' } else { '.' };
        bar.push(ch);
        bar.push(' ');
    }
    let caret_offset = prefix_len + (event.beat as usize * 2);
    let caret_ch = '^';
    let caret_line = format!("{:width$}{}", "", caret_ch, width = caret_offset);
    write!(out, "\x1B[2A").unwrap(); // up 2 lines
    write!(out, "\x1B[2K").unwrap(); // clear line
    writeln!(out, "{}", bar).unwrap();
    write!(out, "\x1B[2K").unwrap(); // clear line
    writeln!(out, "{}", caret_line).unwrap();

    out.flush().unwrap();
}
