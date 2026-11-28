# Platform checklist

CI builds lumen-app, the demos and lumen-gpui on Linux, Windows and macOS, and renders demos in every look on a software GPU (the `shots-*` artifacts of the `native` job). What CI can't do is change OS settings, type with an input method or move a window between monitors. Run this list by hand on a real machine before calling a platform supported, and note the OS version, desktop and GPU in the table at the end.

Build once:

```sh
cargo build --release -p lumen-demos
alias demo=./target/release/lumen-demos   # on Windows: target\release\lumen-demos.exe
```

Each check says what to do and what should happen. "Live" means without restarting the app.

## 1. OS accent color

| Platform | Where to change it |
|---|---|
| macOS | System Settings → Appearance → Accent color |
| Windows 11 | Settings → Personalization → Colors → Accent color (turn off "Automatic") |
| GNOME 47+ | Settings → Appearance → Accent color |
| KDE, older GNOME | Not read; lumen uses the look's default accent |

1. `demo looks --look native`. Primary buttons, the switch, the slider fill and focus rings use the OS accent.
2. Change the accent, then click back into the window. The new accent fades in when the window regains focus (lumen reads it on focus, not continuously).
3. Pick a very light accent (yellow on GNOME). Text on primary buttons stays readable; lumen picks black or white foreground by contrast.

## 2. Reduce motion

| Platform | Setting |
|---|---|
| macOS | System Settings → Accessibility → Display → Reduce motion |
| Windows 11 | Settings → Accessibility → Visual effects → Animation effects (off = reduce) |
| GNOME | Settings → Accessibility → Seeing → Reduce animation (`enable-animations`) |

1. Turn reduce motion on, then focus `demo switch`. Toggling jumps instead of sliding, and menus appear without the spring.
2. Turn it off, refocus. Motion is back.
3. `App::follow_reduced_motion(false)` ignores the OS (check in an app that sets it).

## 3. Dark mode

1. `demo hero --look native`, then switch the OS between light and dark. The window follows live with a short color blend, no flash or restart.
2. Start the demo in dark mode. The first frame is already dark (no light flash).
3. With `--dark` the demo stays dark when the OS is light.
4. Windows: the title bar follows too (winit sets the immersive dark mode attribute).

## 4. Text input and IME

Use `demo text-input` and `demo text-area`.

1. Type with a non-US layout: German (ü, ß), French AZERTY. On Windows also type `@`, `€` and `{` with **AltGr**: these used to be dropped and are the regression to watch.
2. Dead keys: on US International, `'` then `e` gives é.
3. macOS: hold `e` and pick è from the press-and-hold popup.
4. CJK: Japanese (macOS Kotoeri, Windows Microsoft IME, Linux IBus or Fcitx with Mozc), type `nihongo`. The composition shows underlined in the field, the candidate window sits under the caret, Space and Enter pick and commit, and Escape cancels.
5. Chinese Pinyin and Korean (Hangul composes per syllable) work the same way.
6. Emoji: macOS Ctrl+Cmd+Space, Windows Win+. (period); the picked emoji is inserted once.
7. Copy and paste between lumen and another app (Cmd/Ctrl+C, V).

## 5. Display scaling (DPI)

1. Windows: Settings → Display → Scale at 100, 125, 150 and 200 %. Text stays sharp (no blurry upscale), and 1 px hairlines stay 1 device pixel.
2. Drag the window between a 100 % and a 200 % monitor. It re-renders at the new scale as it crosses, with no blurry frame left behind and the same logical size.
3. macOS: a Retina display and an external non-Retina one, same as 2.
4. Linux: GNOME fractional scaling (125 %) on Wayland, and X11 with `Xft.dpi: 144`.

## 6. Window and title bar

1. `demo window-chrome --chrome --look native`. Drag the title bar to move the window; double-click it to maximize (or what the OS setting says: macOS Desktop & Dock, GNOME `action-double-click-titlebar`).
2. Minimize, maximize/restore and close from lumen's buttons. Windows: Win+Up/Down and snap layouts (hover the maximize button) still work; resize from every edge and corner.
3. Linux Wayland: the window has lumen's own title bar and still moves and resizes.

## 7. Screen readers

| Platform | Screen reader |
|---|---|
| macOS | VoiceOver (Cmd+F5) |
| Windows | Narrator (Win+Ctrl+Enter) and NVDA |
| Linux | Orca (GNOME, Super+Alt+S) |

`demo accessibility`:

1. Move through the window with the screen reader's next/previous keys. You should hear: the "Profile" heading, the "Name" text field, the "Wi-Fi" switch (off), the "Volume" slider (4), the "Theme" pop-up/combo box (System), the "Save" and "More" buttons, the "Last action" text, and the inbox list ("Message 1 Ada, 1 of 1000").
2. Activate Save with the screen reader's action key. "Last action" changes to "Saved".
3. Toggle Wi-Fi, step Volume up and down, and type a name. Each change is announced.
4. Open More. The menu's items are announced, and picking Duplicate returns focus to More.
5. Pick Delete… from the menu. The dialog "Delete profile?" is announced with its description, focus is on Delete, and activating it closes the dialog.
6. Tab through the window without the screen reader. Every control gets a visible focus ring.

On macOS, `swiftc -O scripts/ax.swift -o target/ax` then `target/ax <pid> dump` prints the same tree VoiceOver reads.

## 8. GPUI host (lumen-gpui)

In `crates/lumen-gpui`: `cargo run --example dashboard` (Zed's GPUI) and the same with `--no-default-features --features ce` (gpui-ce). Click, type with an IME, open a select that overflows the panel, and switch dark mode.

## Results

| Platform | OS version, desktop, GPU | Date | 1 Accent | 2 Motion | 3 Dark | 4 IME | 5 DPI | 6 Window | 7 Screen reader | 8 GPUI | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| macOS | 27, Apple silicon | 2026-11-27 | | | | partial | | partial | partial | | Not run as a whole yet. Done so far: 4 through synthetic `insertText:`/`setMarkedText:` calls (lumen-gpui selftest), 6 through injected clicks (chrome_selftest), 7 through the AX API (tree read and driven, VoiceOver not listened to). |
| Windows 11 | | | | | | | | | | | not run yet: no machine |
| Linux GNOME (Wayland) | | | | | | | | | | | not run yet: no machine |
| Linux X11 | | | | | | | | | | | not run yet: no machine |
