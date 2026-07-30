# UI
The desktop uses a compact responsive list, keyboard-focusable rows, light/dark persistence, real polling, status filtering, priority and persistent drag ordering, interactive aggregate status, and explicit non-destructive removal. The global limit control changes the live Core limiter. Browser-captured links and schedule-based limits remain outside this milestone.

All current confirmations and forms use reusable accessible Hyper Get dialogs; browser `prompt`, `alert`, and `confirm` are not used. Pressing Ctrl+V outside editable controls reads the clipboard once, recognizes one or multiple HTTP(S) URLs, and opens a confirmation dialog. Clipboard monitoring is off and no background reads occur. This release supports exactly one `*` placeholder.

Dialog focus trapping mounts once and calls the latest close callback through a ref, so status polling cannot reset cursor position. URL probes are debounced and use request IDs to ignore stale results; manually edited filenames are preserved. Quick Download is a separate persistent bar above global status, and clears only after successful insertion.
