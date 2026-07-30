# Testing

Automated Core coverage includes wildcard validation, path safety, persisted settings, byte-weighted global status, unknown sizes, combined active speed, and persisted Pause All/Start All queue state. Vitest covers malformed URL display and secret redaction, but the frontend currently has no DOM test harness; full interaction coverage remains a known limitation.

Manual desktop acceptance: launch `pnpm --filter @hyper-get/desktop tauri dev`; exercise every toolbar/sidebar/row action; paste valid and malformed URLs; verify one dialog; type `*` and use Configure Batch; add known- and unknown-size transfers; verify bytes/speed/ETA; Pause All and ensure queued work stays queued; Start All; change concurrency and destination; restart and confirm both destination and queue state persist.
Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, then `pnpm lint`, `pnpm typecheck`, `pnpm test`, and `pnpm build`.
