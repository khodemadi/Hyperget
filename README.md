# Hyper Get

![Hyper Get icon](apps/desktop/src-tauri/icons/128x128.png)

**Fast, persistent downloads without browser lock-in.** Hyper Get is an open-source download manager for Linux and Windows, powered by Rust, SQLite, Tauri 2, and React.

> Current release: **v0.2.2-alpha.1**

## Highlights

- Persistent priority queue with configurable concurrent downloads
- Pause, resume, restart recovery, partial-file continuation, and remote identity checks
- Real byte-weighted global progress, combined speed, ETA, and queue status
- Global and per-download speed limits
- SHA-256 verification and safe duplicate filename handling
- OS-native download folder selection and persistent settings
- Automatic wildcard discovery: enter one `*` and Hyper Get probes the actual sequence
- Bulk **Clear all** with confirmation
- Chrome/Chromium/Edge and Firefox extensions through the native host
- Light/dark desktop interface with guarded commands and crash fallback

## Downloads

| Platform | Package |
| --- | --- |
| Windows 10/11 x64 | NSIS `.exe` and WiX `.msi` |
| Linux x64 | AppImage and Debian `.deb` |
| Chrome, Chromium, Edge | `hyper-get-chromium.zip` |
| Firefox | `hyper-get-firefox.zip` |

Release assets are built by GitHub Actions and include SHA-256 checksums. This is an alpha release: keep important source URLs until each download has been verified.

## Browser integration

Install Hyper Get, then install the native messaging host:

```bash
# Linux
./scripts/install-browser-host-linux.sh
```

```powershell
# Windows PowerShell
.\scripts\install-browser-host-windows.ps1
```

Load the matching extension ZIP from the release page in developer/unpacked mode. See the [extension guide](extensions/browser/README.md).

## Development

Requirements: Rust 1.85+, Node.js 22+, pnpm 10.14+, and the Tauri system dependencies.

```bash
corepack enable
pnpm install --frozen-lockfile
cargo test --workspace --all-features
pnpm lint && pnpm typecheck && pnpm test
pnpm --filter @hyper-get/desktop tauri dev
```

Build desktop bundles and extensions:

```bash
pnpm --filter @hyper-get/desktop tauri build
pnpm browser:build
pnpm browser:package
```

## Project layout

- `crates/hyper-core`: download engine, scheduler, SQLite persistence
- `crates/hyper-cli`: command-line client
- `crates/hyper-native-host`: browser native messaging bridge
- `apps/desktop`: Tauri and React desktop application
- `extensions/browser`: Chromium and Firefox extensions

Read [Architecture](docs/ARCHITECTURE.md), [Testing](docs/TESTING.md), [Release](docs/RELEASE.md), [Contributing](CONTRIBUTING.md), and [Security](SECURITY.md).

Licensed under the [MIT License](LICENSE).
