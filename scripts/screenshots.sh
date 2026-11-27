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
shot c-bento bento 1200x780 --scale 2
shot c-hero hero 1120x560 --scale 2
shot c-vault vault 1000x600 --scale 2
shot c-looks looks 960x440 --scale 2
wait
shot c-scene-ide scene-ide 1040x640 --scale 2
shot c-scene-observability scene-observability 1040x640 --scale 2
shot c-scene-shader scene-shader 1040x640 --scale 2
shot c-scene-release scene-release 1040x760 --scale 2
wait
shot c-scene-finance scene-finance 1040x640 --scale 2
shot c-scene-collab scene-collab 1040x640 --scale 2
shot c-scene-governance scene-governance 1040x640 --scale 2
shot c-scene-schedule scene-schedule 1040x640 --scale 2
wait
for a in mail music tracker notes; do shot "c-app-$a" "app-$a" 1200x760 --scale 2; done
wait
for a in assistant finance todo settings; do shot "c-app-$a" "app-$a" 1200x760 --scale 2; done
wait
n=0
for d in $("$D" --list | cut -f1); do
  case "$d" in bento|hero|vault|looks|scene-*|app-*) continue ;; esac
  case "$d" in menu|dialog|popover|date-picker|date-range-picker|combobox|multi-select|time-picker|color-picker|sheet|command-palette|hover-card|context-menu|confirm-popover|otp-input|split-button|dock|menu-bar|spotlight-card|mention-input|tour) extra=--open ;; *) extra= ;; esac
  case "$d" in code-editor|file-browser|emoji-picker|wheel-picker|signature-pad|avatar-upload|time-dial|aspect-ratio|ruler-slider|calendar-heatmap|tour|calendar|date-picker|date-range-picker|combobox|multi-select|time-picker|color-picker|sheet|command-palette|hover-card|context-menu|confirm-popover|radio-cards|dropzone|split-button|accordion|table|tree|timeline|code-block|json-viewer|key-value|log-viewer|progress-ring|bar-list|heatmap|radar-chart|scatter-chart|funnel-chart|candlestick|kanban|ai-reasoning|ai-composer|ai-chat|diff-viewer|waterfall|uptime-bars|rolling-number|dock|waveform|window-chrome|split-pane|menu-bar|terminal|gantt|treemap|markdown|property-grid|flame-graph|notification-list|spotlight-card|editable-text|money-field|masked-input|mention-input|step-slider|transfer-list|reorderable-list|editor-tabs|table-of-contents|nav-bar|carousel|callout|undo-toast|image-frame|sankey-chart|contribution-graph|histogram|kpi-strip|day-schedule|node-editor|commit-graph|test-explorer|data-grid|filter-bar|ai-citations|text-effects|rotary-knob|compare-slider|countdown|checklist-card|action-bar|speed-dial|numeric-pad|dimension-input|stacked-meter|announcement-bar|quantity-picker|swatch-picker|date-strip|world-clocks|bullet-chart|metric-table|ai-model-picker|context-meter|api-inspector|permission-matrix|description-list|masonry-grid|collapsible-card|marquee|presence-pile|comment-thread|reaction-bar|attachment-list|progress-steps|list-view|shortcut-bar|view-switcher|gradient-slider|schedule-picker|quarter-picker|win-loss-chart|waffle-chart|schema-viewer|env-switcher|memory-profiler|ai-suggestion|prompt-templates|pricing-cards|ab-comparison|equalizer) sz=736x460 ;; *) sz=560x350 ;; esac
  shot "c-$d" "$d" "$sz" --embed --scale 2 $extra
  n=$((n + 1))
  if [ $((n % 4)) -eq 0 ]; then wait; fi
done
wait
ls "$O"
