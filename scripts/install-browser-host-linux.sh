#!/usr/bin/env bash
set -euo pipefail

host_path="${1:?usage: install-browser-host-linux.sh /absolute/path/to/hyper-get-native-host EXTENSION_ID}"
extension_id="${2:?missing Chromium extension ID}"

config_home="${XDG_CONFIG_HOME:-$HOME/.config}"

if [ -d "$config_home/chromium" ]; then
    manifest_dir="$config_home/chromium/NativeMessagingHosts"
elif [ -d "$config_home/google-chrome" ]; then
    manifest_dir="$config_home/google-chrome/NativeMessagingHosts"
else
    manifest_dir="$config_home/chromium/NativeMessagingHosts"
fi

mkdir -p "$manifest_dir"

cat > "$manifest_dir/io.github.hyper_get.json" <<EOF
{
  "name": "io.github.hyper_get",
  "description": "Hyper Get native messaging bridge",
  "path": "$host_path",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$extension_id/"
  ]
}
EOF

echo "Installed native messaging manifest:"
echo "  $manifest_dir/io.github.hyper_get.json"