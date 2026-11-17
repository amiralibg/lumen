#!/usr/bin/env bash
# Regenerate every image in site/img (used by the site and the README).
# Each image is drawn by lumen itself, headless, in light and dark.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -q -p lumen-demos
D=./target/debug/lumen-demos
O=site/img
mkdir -p "$O"
shot() { # name demo size [flags...]
  local name=$1 demo=$2 size=$3; shift 3
  "$D" "$demo" --size "$size" --screenshot "$O/$name-light.png" "$@" >/dev/null &
  "$D" "$demo" --size "$size" --dark --screenshot "$O/$name-dark.png" "$@" >/dev/null &
}
shot hero hero 1120x540
shot vault vault 1000x600
shot calendar calendar 720x400 --embed
shot menu menu 720x400 --embed --open
shot dialog dialog 720x400 --embed --open
shot chart-line chart-line 820x360 --embed
shot chart-bar chart-bar 820x360 --embed
shot donut donut 620x300 --embed
# Card thumbnails: same size, cropped to the interesting part.
shot thumb-date date-picker 640x440 --embed --open
shot thumb-menu menu 640x440 --embed --open
shot thumb-dialog dialog 640x440 --embed --open
for l in lumen mac windows gnome; do shot "look-$l" looks 760x440 --look "$l"; done
wait
# Posters for every live demo on the site (img/c-<demo>-{light,dark}.png):
# shown until the WebAssembly demo is ready, and as component card thumbnails.
shot c-bento bento 1200x760 --scale 1.5
shot c-hero hero 1120x560 --scale 1.5
shot c-vault vault 1000x600 --scale 1.5
shot c-looks looks 960x440 --scale 1.5
wait
for d in $("$D" --list | cut -f1); do
  case "$d" in bento|hero|vault|looks) continue ;; esac
  case "$d" in menu|dialog|popover|date-picker|date-range-picker|combobox|multi-select|time-picker|color-picker|sheet|command-palette|hover-card|context-menu|confirm-popover|otp-input|split-button) extra=--open ;; *) extra= ;; esac
  case "$d" in date-picker|date-range-picker|combobox|multi-select|time-picker|color-picker|sheet|command-palette|hover-card|context-menu|confirm-popover|radio-cards|dropzone|split-button|accordion|table|tree|timeline|code-block|json-viewer|key-value|log-viewer) sz=736x460 ;; *) sz=560x350 ;; esac
  shot "c-$d" "$d" "$sz" --embed --scale 2 $extra
  wait
done
ls "$O"
