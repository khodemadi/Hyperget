# Architecture

The Rust service and SQLite store own download, scheduler, settings, and aggregate-status truth. Tauri exposes typed commands. React polls through one non-overlapping guarded refresh loop and renders the returned global values; filtered rows are never used to derive authoritative totals. Browser inbox polling is independently guarded and cleaned up on unmount.

Desktop errors cross one typed command boundary and are rendered as operation notifications. Render-time exceptions are contained by the application Error Boundary. URL-derived labels use defensive parsing so malformed persisted values cannot take down the component tree.
`hyper-core` owns HTTP, files, validation, SQLite, recovery, and lifecycle rules. The CLI and Tauri adapter are thin clients of `DownloadService`; React invokes typed commands and renders snapshots.
