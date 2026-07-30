# Hyper Get browser integration

Shared TypeScript builds Chromium/Edge Manifest V3 and Firefox-compatible packages. It provides **Download with Hyper Get**, page-link extraction, deduplication, and the `*` shortcut outside editable controls. The shortcut sends a selection request; it never starts page downloads automatically.

```bash
corepack pnpm browser:build
corepack pnpm browser:package
```

Load `dist/chromium` as an unpacked extension. Communication uses the `io.github.hyper_get` Native Messaging host. Build `hyper-get-native-host`, then run the platform installer script with its absolute path and the installed extension ID.

Current limitation: the host securely validates message size/type and writes an atomic desktop inbox, and Tauri exposes inbox receipt, but automatic foregrounding/presentation of inbox messages is not wired yet. Firefox packages build, but end-to-end Firefox Native Messaging has not been manually verified.
