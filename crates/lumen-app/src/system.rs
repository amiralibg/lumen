//! OS preferences: accent color and reduced motion.
//!
//! Dark mode comes from winit. Everything here is best effort: an unknown
//! value is `None` and lumen falls back to the look's defaults.

use lumen_core::{Color, SystemPrefs};

/// Query the OS. Cheap on macOS and the web; spawns a short-lived process
/// on Windows and Linux, so call it on startup and focus changes, not per
/// frame.
pub fn query() -> SystemPrefs {
    SystemPrefs { dark: None, accent: accent(), reduced_motion: reduced_motion() }
}

#[cfg(target_os = "macos")]
pub fn accent() -> Option<Color> {
    use objc2_app_kit::{NSColor, NSColorSpace};
    let c = NSColor::controlAccentColor();
    let c = c.colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
    Some(Color::rgba(c.redComponent() as f32, c.greenComponent() as f32, c.blueComponent() as f32, 1.0))
}

#[cfg(target_os = "macos")]
pub fn reduced_motion() -> Option<bool> {
    Some(objc2_app_kit::NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion())
}

#[cfg(target_os = "windows")]
fn reg_dword(path: &str, name: &str) -> Option<u32> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let out = std::process::Command::new("reg").args(["query", path, "/v", name]).creation_flags(CREATE_NO_WINDOW).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().find(|l| l.contains("REG_DWORD"))?;
    let hex = line.split_whitespace().last()?.trim_start_matches("0x");
    u32::from_str_radix(hex, 16).ok()
}

#[cfg(target_os = "windows")]
pub fn accent() -> Option<Color> {
    // AccentColor is 0xAABBGGRR.
    let v = reg_dword(r"HKCU\Software\Microsoft\Windows\DWM", "AccentColor")?;
    let (r, g, b) = (v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF);
    Some(Color::hex((r << 16) | (g << 8) | b))
}

#[cfg(target_os = "windows")]
pub fn reduced_motion() -> Option<bool> {
    // Settings > Accessibility > Visual effects > Animation effects.
    let v = reg_dword(r"HKCU\Control Panel\Desktop\WindowMetrics", "MinAnimate");
    v.map(|v| v == 0)
}

#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
fn gsettings(key: &str) -> Option<String> {
    let out = std::process::Command::new("gsettings").args(["get", "org.gnome.desktop.interface", key]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().trim_matches('\'').to_owned())
}

#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
pub fn accent() -> Option<Color> {
    // GNOME 47+ accent names, as libadwaita renders them.
    let hex = match gsettings("accent-color")?.as_str() {
        "blue" => 0x3584E4,
        "teal" => 0x2190A4,
        "green" => 0x3A944A,
        "yellow" => 0xC88800,
        "orange" => 0xED5B00,
        "red" => 0xE62D42,
        "pink" => 0xD56199,
        "purple" => 0x9141AC,
        "slate" => 0x6F8396,
        _ => return None,
    };
    Some(Color::hex(hex))
}

#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
pub fn reduced_motion() -> Option<bool> {
    gsettings("enable-animations").map(|v| v == "false")
}

#[cfg(target_arch = "wasm32")]
pub fn accent() -> Option<Color> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn reduced_motion() -> Option<bool> {
    let w = web_sys::window()?;
    Some(w.match_media("(prefers-reduced-motion: reduce)").ok()??.matches())
}

#[cfg(not(any(unix, windows, target_arch = "wasm32")))]
pub fn accent() -> Option<Color> {
    None
}

#[cfg(not(any(unix, windows, target_arch = "wasm32")))]
pub fn reduced_motion() -> Option<bool> {
    None
}
