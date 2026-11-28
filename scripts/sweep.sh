#!/usr/bin/env bash
# Layout sweep: every demo x 4 looks x 2 modes at poster size, at card width
# (380x360; scenes 760x780) and apps at 1200x760, 760x780 and 380x720, with
# --check. Usage: scripts/sweep.sh OUTDIR  (findings in OUTDIR/findings.txt)
cd "$(dirname "$0")/.." && cargo build -q -p lumen-demos
D=./target/debug/lumen-demos; O=$1; mkdir -p "$O/img"; : > "$O/findings.txt"
size_for() { grep -q "|$1|" <<< "$BIG" && echo 736x460 || echo 560x350; }
BIG="|$(grep -o 'code-editor|file-browser[^)]*' scripts/screenshots.sh)|"
OPEN="|menu|dialog|popover|date-picker|date-range-picker|combobox|multi-select|time-picker|color-picker|sheet|command-palette|hover-card|context-menu|confirm-popover|otp-input|split-button|dock|menu-bar|spotlight-card|mention-input|tour|"
run() { # tag demo size flags...
  local tag=$1 d=$2 sz=$3; shift 3
  out=$("$D" "$d" --size "$sz" --screenshot "$O/img/$tag.png" --check "$@" 2>&1 | grep -v '^wrote')
  [ -n "$out" ] && printf '== %s\n%s\n' "$tag" "$out" >> "$O/findings.txt"
}
export -f run; export D O
jobs=()
for d in $($D --list | cut -f1); do
  ex=; grep -q "|$d|" <<< "$OPEN" && ex=--open
  case "$d" in
    app-*) for s in 1200x760 760x780 380x720; do for m in "" --dark; do echo "run $d-$s${m:+-dark} $d $s $m"; done; done; continue;;
    scene-*|bento|hero|vault|looks) sz=1040x640; emb=;;
    *) sz=$(size_for $d); emb=--embed;;
  esac
  for l in lumen mac windows gnome; do for m in "" --dark; do echo "run $d-$l${m:+-dark} $d $sz --look $l $emb $ex $m"; done; done
  case "$d" in scene-*|bento|hero|vault|looks) cs=760x780;; *) cs=380x360;; esac
  for m in "" --dark; do echo "run $d-card${m:+-dark} $d $cs --embed $ex $m"; done
done > "$O/cmds.txt"
xargs -P 10 -I{} bash -c '{}' < "$O/cmds.txt"
wc -l < "$O/cmds.txt"; grep -c '^==' "$O/findings.txt"
