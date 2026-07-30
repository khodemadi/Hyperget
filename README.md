# Hyper Get

Hyper Get is a production-oriented, cross-platform download manager built with Rust, SQLite, Tauri 2, and React. The current milestone is `v0.1.0-alpha.1`.

Implemented in this alpha: a persistent priority queue, concurrency scheduler, global and per-download rate gates, explicit lifecycle validation, restart recovery, streamed HTTP downloads, partial-file resume, remote identity checks, SHA-256 verification, byte-weighted aggregate progress, a shared CLI, and a live desktop UI. Browser integration and schedule-based limits are not implemented. Windows installers are built and must be verified on the Windows GitHub Actions runner.

## Development

```bash
cargo run -p hyper-cli -- add https://example.com/file.iso
cargo run -p hyper-cli -- list
corepack enable
pnpm install
pnpm --filter @hyper-get/desktop tauri dev
```

By default the CLI stores its database and downloads under the platform data directory. Set `HYPER_GET_DATA_DIR` to use an explicit location. SQLite WAL and a busy timeout provide safe serialization; avoid controlling the same active download from two processes in this alpha.

See [Architecture](docs/ARCHITECTURE.md), [testing](docs/TESTING.md), and [release notes](docs/RELEASE.md).
