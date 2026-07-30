# Browser extension protocol

The extension uses browser Native Messaging, not an open network port. Messages are little-endian length-prefixed JSON and are capped at 1 MiB by `hyper-get-native-host`. Accepted types are `send_single_download`, `send_page_links`, `ping`, `get_desktop_status`, and `open_application`; unknown types are rejected. Browser manifests restrict which extension origin may launch the host. Inbox files are written atomically and never accept output paths.

Cookies and authorization headers are deliberately not transferred in this alpha. No values are logged. The extension does not cancel browser downloads and therefore cannot lose them when the desktop is unavailable.

The content script extracts HTTP(S) URLs from anchors, video, audio, source, and image elements; normalizes relative URLs; removes duplicates; and ignores `*` while focus is in an editable control.
