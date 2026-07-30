# Changelog

## v0.2.2-alpha

- Removed the bottom Quick Download URL bar and added real backend global progress above the compact status bar.
- Added byte-weighted known-size progress, unknown-size reporting, combined active speed, ETA, active filename, and persisted queue execution state.
- Fixed unsafe URL rendering, unhandled Tauri command failures, overlapping polling, duplicate inbox polling, and wildcard dialog replacement.
- Replaced scheduler `try_lock` failures with serialized scheduling and made Pause All block queued starts.
- Added a Hyper Get React error boundary and in-app operation error notifications with redaction.
## [0.1.0-alpha.1] - Unreleased
### Added
- Persistent SQLite download queue, resumable streaming core, CLI, and Tauri desktop shell.
- Remote metadata probing, validators, checksum verification, lifecycle validation, and recovery.
- Persistent priority ordering, concurrency scheduling, aggregate live status, and rate limits.
- Windows NSIS/MSI release jobs with SHA-256 checksums.
- Custom clipboard, multiple-URL and wildcard batch dialogs backed by Rust batch expansion.
- Buildable Chromium and Firefox extension packages plus a bounded Native Messaging host.
- Stable dialog focus, single-wildcard batches, persistent Quick Download, and OS-aware destinations.
