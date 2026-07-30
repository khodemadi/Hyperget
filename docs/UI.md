# UI
The desktop uses a compact responsive list, keyboard-focusable rows, light/dark persistence, real polling, status filtering, priority and persistent drag ordering, interactive aggregate status, and explicit non-destructive removal. The global limit control changes the live Core limiter. Browser-captured links and schedule-based limits remain outside this milestone.

All current confirmations and forms use reusable accessible Hyper Get dialogs; browser `prompt`, `alert`, and `confirm` are not used. Pressing Ctrl+V outside editable controls reads the clipboard once, recognizes one or multiple HTTP(S) URLs, and opens a confirmation dialog. Clipboard monitoring is off and no background reads occur. This release supports exactly one `*` placeholder.

Dialog focus trapping mounts once and calls the latest close callback through a ref, so status polling cannot reset cursor position. URL probes are debounced and use request IDs to ignore stale results; manually edited filenames are preserved. Typing one wildcard keeps the Add Download shell mounted and reveals Configure Batch.

The bottom contains exactly a backend-driven global progress panel followed by the compact global status bar. Unknown totals use an indeterminate accessible progress track. The removed Quick Download input is not part of the dashboard.

All Tauri commands pass through `safeInvoke`, which normalizes errors into `AppCommandError`. Actions surface failures in a custom toast instead of leaving rejected promises. A React Error Boundary provides reload, redacted error-copy, and log-folder actions for render failures.
