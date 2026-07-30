# Architecture
`hyper-core` owns HTTP, files, validation, SQLite, recovery, and lifecycle rules. The CLI and Tauri adapter are thin clients of `DownloadService`; React invokes typed commands and renders snapshots.
