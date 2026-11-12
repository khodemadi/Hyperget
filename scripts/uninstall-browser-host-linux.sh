#!/usr/bin/env bash
set -euo pipefail
base="${XDG_CONFIG_HOME:-$HOME/.config}"
name="io.github.hyper_get.json"
for dir in chromium google-chrome google-chrome-beta google-chrome-unstable microsoft-edge BraveSoftware/Brave-Browser vivaldi; do
    path="$base/$dir/NativeMessagingHosts/$name"
    [[ ! -f "$path" ]] || rm "$path"
done
path="$HOME/.mozilla/native-messaging-hosts/$name"
[[ ! -f "$path" ]] || rm "$path"
