# Codex implementation task: Hyper Get v0.1.0-alpha

You are working inside the existing Hyper Get repository. Build the first complete, testable and GitHub-ready milestone of a cross-platform download manager using Rust and Tauri.

Do not create a visual-only demo. Do not use fake download records. Do not rewrite the existing useful core from scratch without a technical reason. Inspect it, repair it, move it into the new architecture, add tests and connect it to the real desktop UI and CLI.

## Product objective

Hyper Get is an open-source IDM-style download manager for Linux and Windows.

The long-term product includes:

- Segmented HTTP downloads.
- Persistent pause and resume.
- Retry and crash recovery.
- Queue ordering.
- Multiple simultaneous downloads.
- Global and per-file progress.
- Speed limits.
- Scheduler.
- CLI.
- Tauri desktop application.
- Chrome, Chromium, Edge and Firefox integration.

This task implements only `v0.1.0-alpha`, but the architecture must support later milestones.

## First action: inspect and preserve the current core

Read every existing Rust source file and the current README before changing code.

The imported core currently includes useful logic for:

- Tokio and Reqwest downloads.
- HTTP Range requests.
- Segments.
- Retry with exponential backoff.
- Part files and a manifest.
- Segment merge.
- Optional SHA-256 verification.
- Single-stream fallback.

Known immediate compile problems include:

- `pub mod segment;` exists but `segment.rs` is missing.
- `types.rs` uses `Serialize` and `Deserialize` without importing them.
- `StarLevel` is missing traits required by `DownloadOptions`.
- There is an unused `AsyncSeekExt` import.
- The `crs/` source layout is non-standard.

Fix these problems as part of the migration. Do not merely add empty placeholders. Implement real segment types and splitting tests.

## Required repository architecture

Convert the repository into a clean Cargo and pnpm workspace:

```text
hyper-get/
├── Cargo.toml
├── Cargo.lock
├── package.json
├── pnpm-workspace.yaml
├── README.md
├── README.fa.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── LICENSE
├── .gitignore
├── .editorconfig
├── rustfmt.toml
├── clippy.toml
│
├── crates/
│   ├── hyper-core/
│   │   ├── Cargo.toml
│   │   ├── migrations/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── downloader/
│   │       ├── http/
│   │       ├── segment/
│   │       ├── queue/
│   │       ├── persistence/
│   │       ├── events/
│   │       ├── verify/
│   │       ├── types.rs
│   │       └── error.rs
│   └── hyper-cli/
│       ├── Cargo.toml
│       └── src/main.rs
│
├── apps/
│   └── desktop/
│       ├── package.json
│       ├── vite.config.ts
│       ├── src/
│       └── src-tauri/
│
├── extensions/
│   └── browser/
│       └── README.md
│
├── docs/
│   ├── ARCHITECTURE.md
│   ├── CORE.md
│   ├── UI.md
│   ├── TESTING.md
│   ├── EXTENSION_PROTOCOL.md
│   └── RELEASE.md
│
└── .github/
    ├── workflows/
    │   ├── ci.yml
    │   └── release.yml
    ├── ISSUE_TEMPLATE/
    ├── dependabot.yml
    └── pull_request_template.md
```

The browser extension directory is documentation/scaffolding only in v0.1.0-alpha. Do not implement download interception yet.

## Technology requirements

Backend:

- Rust stable.
- Tokio.
- Reqwest with Rustls unless a concrete compatibility reason requires another TLS backend.
- Serde.
- Thiserror.
- Tokio cancellation tokens.
- SQLite with explicit migrations.
- UUID or another stable ID type.
- Tracing for structured logs.
- No Tauri dependencies inside `hyper-core`.

Desktop:

- Tauri 2.
- React.
- TypeScript with strict mode.
- Vite.
- Tailwind CSS.
- Accessible components.
- Dark and light themes.
- No external CDN runtime dependencies.
- No fake production data.

CLI:

- Clap.
- Human-readable output.
- `--json` where applicable.
- Same core as the desktop app.

Use pnpm for frontend workspace management.

## Core requirements for v0.1.0-alpha

Implement a service-level API that supports:

- Add download.
- Start download.
- Pause download.
- Resume download.
- Cancel download.
- Remove from list.
- Optionally remove partial or completed data through a separate explicit flag.
- List and inspect downloads.
- Start all.
- Pause all.
- Persist queue and state across application restarts.

Required states:

- `Created`
- `Resolving`
- `Queued`
- `Connecting`
- `Downloading`
- `Pausing`
- `Paused`
- `RetryWaiting`
- `Merging`
- `Verifying`
- `Completed`
- `Failed`
- `Cancelled`

Validate state transitions. Return structured errors for invalid operations.

### HTTP behavior

Do not rely only on `HEAD`.

Implement robust probing:

1. Validate and normalize the URL.
2. Try `HEAD` when useful.
3. Fall back to a `Range: bytes=0-0` request when metadata is missing or ambiguous.
4. Validate `206` and `Content-Range`.
5. Follow redirects safely.
6. Detect filename from `Content-Disposition`, URL or user override.
7. Store total size when known.
8. Store `ETag` and `Last-Modified`.
9. Choose segmented or single-stream mode.
10. Handle unknown-size responses in single-stream mode.

Segmented downloading must:

- Use inclusive byte ranges correctly.
- Avoid overlaps and gaps.
- Persist each segment.
- Resume from the actual partial file length.
- Flush before marking complete.
- Verify range responses.
- Merge in order.
- Keep memory bounded.
- Fall back safely when the server does not support ranges before useful segmented work starts.
- Return a clear error if a server violates range semantics after partial segmented data exists.

### Resume and crash safety

- Use SQLite transactions for durable application state.
- Use atomic writes for sidecar data when needed.
- Reconcile database state with partial file lengths at startup.
- Convert stale `Downloading`, `Connecting`, `Merging` or `Verifying` records into a recoverable state after restart.
- Validate remote identity using `ETag`, `Last-Modified` and size when available.
- Do not resume silently when remote content changed.
- Keep partial data on pause and normal shutdown.
- Do not corrupt completed output when merge fails.

### Pause and cancellation

Use cooperative cancellation.

Pause must:

- stop network reads promptly;
- flush active files;
- save current byte counts;
- preserve reusable partial files;
- emit a final paused event.

Cancel must stop work but not implicitly delete files. Removal and deletion are separate operations.

### Events and progress

Create serializable events suitable for Tauri and CLI:

- Download added.
- State changed.
- Progress.
- Speed and ETA.
- Segment update.
- Completed.
- Failed.
- Removed.

Include stable download IDs, timestamps and monotonic sequence numbers where practical.

Throttle or batch progress events. Do not send one Tauri event per network chunk.

Calculate:

- Per-download downloaded bytes.
- Per-download percentage.
- Current speed.
- Smoothed speed.
- ETA.
- Aggregate downloaded bytes.
- Aggregate known total bytes.
- Combined speed.
- Status counts.

Aggregate percentage must be based on total bytes, not an average of percentages.

### Database

Use explicit migrations and persist at least:

- Downloads.
- Segments.
- Settings.
- Queue position.
- Remote validators.
- Error information.
- Created, updated and completed timestamps.

Use a data directory appropriate for Tauri and each platform. Tests must use isolated temporary databases.

## Tauri commands

Expose typed commands for:

- `add_download`
- `list_downloads`
- `get_download`
- `start_download`
- `pause_download`
- `resume_download`
- `cancel_download`
- `remove_download`
- `start_all`
- `pause_all`
- `reorder_downloads`
- `get_global_status`
- `get_settings`
- `update_settings`
- `choose_download_directory`
- `reveal_in_folder`
- `open_downloaded_file`

Commands must call `hyper-core`; do not duplicate downloader logic in Tauri.

Use Tauri capabilities with the minimum required permissions.

## Desktop UI requirements

Build a polished, functional desktop interface. It should feel like a modern professional download manager, not a website placed in a window.

### Layout

- Top toolbar:
  - Search.
  - Add URL.
  - Start all.
  - Pause all.
  - Settings.
- Left sidebar:
  - All.
  - Active.
  - Queued.
  - Paused.
  - Completed.
  - Failed.
  - File categories.
- Main download list.
- Optional details drawer.
- Persistent bottom global status bar.

### Download item

Show real backend data:

- Filename.
- Host.
- Progress bar.
- Downloaded and total bytes.
- Percentage where known.
- Current speed.
- ETA.
- Status.
- Quick actions.
- Priority and queue position when applicable.
- Error summary when failed.

### Bottom status bar

Show real aggregate data:

- Overall determinate progress.
- Downloaded bytes / known total bytes.
- Combined speed.
- Active count.
- Queued count.
- Paused count.
- Completed count.
- Failed count.
- Global limit placeholder marked “not implemented” if limiting is not part of this milestone.

### UI behaviors

- Add URL dialog with URL, optional filename, destination and connection count.
- Multi-select.
- Context menu.
- Accessible keyboard navigation.
- Pause, resume, cancel, remove and retry.
- Clear distinction between remove from list and delete data.
- Drag-and-drop queue ordering. If full drag ordering cannot be completed safely in this milestone, implement explicit move up/down plus the backend reorder API and document drag ordering as the only remaining v0.1 UI limitation. Prefer completing it.
- Dark and light themes with persistence.
- Responsive narrow-window layout.
- Empty states and loading states.
- Error details.
- No decorative fake speed graph. A graph may be included only if driven by real sampled speed data.
- Avoid excessive gradients, glass effects and animation.
- Do not copy IDM branding or assets.
- Use clean icons from a package already included in the project.
- Do not set unsafe inline HTML.

### Frontend performance

- Batch progress updates.
- Avoid rerendering every row on every event.
- Keep stable object identity where possible.
- Throttle charts.
- Use a virtual list only if necessary, but architect the list so it can be added later.

## CLI requirements

Implement:

```text
hyper-get add <url> [--output <path>] [--connections <n>]
hyper-get list [--json]
hyper-get status [--json]
hyper-get start <id>
hyper-get pause <id>
hyper-get resume <id>
hyper-get cancel <id>
hyper-get remove <id> [--delete-data]
hyper-get start-all
hyper-get pause-all
```

The CLI must use the same persistent database and core service. Document how the CLI behaves when the Tauri application is open. For v0.1, use a safe single-writer strategy or an application lock; do not allow database corruption.

## Required tests

Create a deterministic local HTTP test server or fixtures that can simulate:

- Correct Range support.
- No Range support.
- Redirects.
- Retryable disconnects.
- Unknown content length.
- Invalid `Content-Range`.
- Changed `ETag`.
- Slow streams.
- Small and multi-megabyte files.

Add tests for:

- Segment splitting with no gaps or overlaps.
- Single-stream download.
- Segmented download.
- Correct merged output.
- Pause and resume.
- Resume after process/service restart.
- Retry.
- Remote file change rejection.
- SHA-256 pass and fail.
- State transition validation.
- Queue persistence.
- Aggregate progress math.
- Filename sanitization and path traversal prevention.
- Duplicate filename handling.

Frontend tests must cover core UI state and command invocation boundaries. Do not mock the entire application into a permanently green test; test meaningful behavior.

## Quality gates

Run and fix all failures:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
pnpm lint
pnpm typecheck
pnpm test
pnpm build
```

Also run the Tauri development build if platform dependencies are present:

```bash
pnpm --filter @hyper-get/desktop tauri dev
```

Attempt a production Tauri build:

```bash
pnpm --filter @hyper-get/desktop tauri build
```

If system packaging dependencies are unavailable, do not claim success. Record the exact missing dependency and keep all other checks green.

## GitHub-ready requirements

Create and verify:

- Accurate English and Persian READMEs.
- MIT license.
- Keep a Changelog-compatible `CHANGELOG.md`.
- `CONTRIBUTING.md`.
- `SECURITY.md`.
- `CODE_OF_CONDUCT.md`.
- Comprehensive `.gitignore` for Rust, Node, Tauri, IDE, local databases, downloaded files, partial files, logs, secrets and packaging output.
- `.editorconfig`.
- Architecture, core, UI, testing and release docs.
- Issue templates.
- Pull request template.
- Dependabot.
- CI workflow for Rust and frontend checks.
- Release workflow for Linux and Windows using Tauri.
- Release artifacts for Linux AppImage and `.deb`, and Windows installer formats supported by the chosen Tauri configuration.
- SHA-256 checksum generation for release assets.
- Semantic versioning.
- Suggested first tag: `v0.1.0-alpha.1`.
- Suggested release title and release notes.
- Clean source archive instructions.
- No secrets, tokens, local databases, build output or downloaded test files committed.

Do not create or push a Git commit unless explicitly requested. Prepare the repository so the owner can review and commit it.

## Documentation honesty

Update README implementation markers based on actual completed code and test results.

Do not claim:

- Browser extension support.
- Bandwidth limiting.
- Full scheduler support.
- Windows packaging success.
- Linux packaging success.

unless each one is actually implemented and verified.

Document known limitations.

## Completion report

At the end of the task:

1. Summarize the architecture created.
2. List files added, moved and removed.
3. List commands run.
4. Show passed and failed checks.
5. Explain any packaging limitation.
6. Provide exact commands for local development.
7. Provide exact commands for the first manual test.
8. Provide a suggested commit message.
9. Provide a suggested tag.
10. Provide release title and release notes.
11. Stop after `v0.1.0-alpha`; do not implement v0.2 features unless they are structurally necessary for v0.1.
