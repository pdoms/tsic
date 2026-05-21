# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-20

### Added

- Multi-time-signature click track engine — define sections with independent time signatures and BPM
- WAV output — render click track to a 16-bit PCM WAV file
- MIDI output — render click track to a standard MIDI file with GM drum channel
- Real-time playback via rodio with low-latency ALSA buffer configuration
- Terminal visualizer — live beat display with pause-aware timing, showing current position in bar, section and project
- Tap tempo subcommand — tap SPACE to detect BPM with audible feedback, capped 12-tap rolling average
- Pause/resume and quit keybindings during playback (SPACE / q)
- Section management — append, insert, and edit sections via CLI
- Config system with per-section BPM and time signature overrides and global fallbacks
- MIDI configuration — customizable drum notes, velocities, channel and tick resolution
- GitHub Actions CI — format check, clippy, and tests on every pull request
- Release workflow — automated Linux and Windows binaries attached to GitHub Releases
- Metronome - use bpm and time signature and play infinitely
