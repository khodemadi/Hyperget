#!/usr/bin/env bash
set -euo pipefail
host_path="${1:?usage: install-browser-host-linux.sh /absolute/path/to/hyper-get-native-host EXTENSION_ID}"
extension_id="${2:?missing Chromium extension ID}"
manifest_dir="${XDG_CONFIG_HOME:-$HOME/.config}/google-chrome/NativeMessagingHosts"
mkdir -p "$manifest_dir"
printf '{"name":"io.github.hyper_get","description":"Hyper Get native messaging bridge","path":"%s","type":"stdio","allowed_origins":["chrome-extension://%s/"]}\n' "$host_path" "$extension_id" > "$manifest_dir/io.github.hyper_get.json"
echo "Installed Chrome-compatible manifest in $manifest_dir"
