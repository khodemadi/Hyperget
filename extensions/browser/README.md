# Hyper Get browser integration

Shared TypeScript builds Chromium/Edge Manifest V3 and Firefox-compatible packages. It provides **Download with Hyper Get**, page-link extraction, deduplication, and the `*` shortcut outside editable controls. The shortcut sends a selection request; it never starts page downloads automatically.

```bash
corepack pnpm browser:build
corepack pnpm browser:package
```

Load `dist/chromium` as an unpacked extension. The Chromium manifest pins a development `key`, so the extension ID is always `cjlafmbgkfhgahjdndnaakkljodbncei`; remove the `key` field before publishing to a store. Communication uses the `io.github.hyper_get` Native Messaging host. Build `hyper-get-native-host`, then run the platform installer with the host's absolute path:

```bash
scripts/install-browser-host-linux.sh /abs/path/hyper-get-native-host [CHROMIUM_ID] [FIREFOX_ID]
```

```powershell
scripts\install-browser-host-windows.ps1 -HostPath C:\abs\path\hyper-get-native-host.exe
```

The installers register Chrome, Chromium, Edge, Brave and Vivaldi (Linux) or Chrome, Edge and Brave (Windows), plus Firefox, which uses its own manifest with `allowed_extensions`. Load Firefox via `about:debugging` -> This Firefox -> Load Temporary Add-on -> `dist/firefox/manifest.json`. The toolbar button launches the desktop app from `HYPER_GET_APP`, a `hyper-get-desktop` binary next to the host, or `PATH`.

Current limitation: the host securely validates message size/type and writes an atomic desktop inbox, and Tauri exposes inbox receipt, but automatic foregrounding/presentation of inbox messages is not wired yet. Firefox packages build, but end-to-end Firefox Native Messaging has not been manually verified.
