#!/usr/bin/env bash
set -euo pipefail
base="${XDG_CONFIG_HOME:-$HOME/.config}"
for path in "$base/google-chrome/NativeMessagingHosts/io.github.hyper_get.json" "$base/chromium/NativeMessagingHosts/io.github.hyper_get.json" "$base/microsoft-edge/NativeMessagingHosts/io.github.hyper_get.json"; do [[ ! -f "$path" ]] || rm "$path"; done
