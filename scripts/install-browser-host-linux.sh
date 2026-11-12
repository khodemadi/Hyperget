#!/usr/bin/env bash
set -euo pipefail

usage="usage: install-browser-host-linux.sh /absolute/path/to/hyper-get-native-host [CHROMIUM_EXTENSION_ID] [FIREFOX_EXTENSION_ID]"
host_path="${1:?$usage}"
# Defaults match the pinned "key" in extensions/browser/chromium/manifest.json and the Firefox gecko id.
extension_id="${2:-cjlafmbgkfhgahjdndnaakkljodbncei}"
firefox_id="${3:-hyper-get@example.invalid}"

case "$host_path" in
    /*) ;;
    *) echo "host path must be absolute: $host_path" >&2; exit 1 ;;
esac

if [ ! -x "$host_path" ]; then
    echo "host binary not found or not executable: $host_path" >&2
    exit 1
fi

config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
name="io.github.hyper_get"
installed=0

write_chromium_manifest() {
    local dir="$1"
    mkdir -p "$dir"
    cat > "$dir/$name.json" <<JSON
{
  "name": "$name",
  "description": "Hyper Get native messaging bridge",
  "path": "$host_path",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$extension_id/"
  ]
}
JSON
    echo "  $dir/$name.json"
    installed=1
}

echo "Installed native messaging manifests:"

# Chromium-family browsers: install for every browser whose profile directory exists.
for browser in chromium google-chrome google-chrome-beta google-chrome-unstable microsoft-edge BraveSoftware/Brave-Browser vivaldi; do
    if [ -d "$config_home/$browser" ]; then
        write_chromium_manifest "$config_home/$browser/NativeMessagingHosts"
    fi
done

if [ "$installed" -eq 0 ]; then
    write_chromium_manifest "$config_home/chromium/NativeMessagingHosts"
fi

# Firefox uses a different directory and "allowed_extensions" instead of "allowed_origins".
firefox_dir="$HOME/.mozilla/native-messaging-hosts"
mkdir -p "$firefox_dir"
cat > "$firefox_dir/$name.json" <<JSON
{
  "name": "$name",
  "description": "Hyper Get native messaging bridge",
  "path": "$host_path",
  "type": "stdio",
  "allowed_extensions": [
    "$firefox_id"
  ]
}
JSON
echo "  $firefox_dir/$name.json"

echo
echo "Chromium extension ID: $extension_id"
echo "Firefox extension ID:  $firefox_id"
echo "Snap/Flatpak browsers use sandboxed paths and are not covered by this script."
