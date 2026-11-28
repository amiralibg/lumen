//! OS preferences: accent color and reduced motion.
//!
//! Dark mode comes from winit. Everything here is best effort: an unknown
//! value is `None` and lumen falls back to the look's defaults.

use lumen_core::{Color, SystemPrefs};

/// What double-clicking a window's title bar does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleAction {
    Maximize,
    Minimize,
    None,
}

/// The OS setting for double-clicking a title bar (System Settings > Desktop
/// & Dock on macOS, `action-double-click-titlebar` on GNOME). Windows always
/// maximizes.
pub fn title_double_click() -> TitleAction {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::{NSString, NSUserDefaults};
        let v = NSUserDefaults::standardUserDefaults().stringForKey(&NSString::from_str("AppleActionOnDoubleClick"));
        return match v.map(|s| s.to_string()).as_deref() {
            Some("Minimize") => TitleAction::Minimize,
            Some("None") => TitleAction::None,
            _ => TitleAction::Maximize,
        };
    }
    #[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
    {
        let out = std::process::Command::new("gsettings").args(["get", "org.gnome.desktop.wm.preferences", "action-double-click-titlebar"]).output();
        let v = out.ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().trim_matches('\'').to_owned());
        return match v.as_deref() {
            Some("minimize") => TitleAction::Minimize,
            Some("none" | "lower" | "menu") => TitleAction::None,
            _ => TitleAction::Maximize,
        };
    }
    #[allow(unreachable_code)]
    TitleAction::Maximize
}

/// Query the OS. Cheap on macOS, Windows and the web; spawns a short-lived
/// `gsettings` on Linux, so call it on startup and focus changes, not per
/// frame.
pub fn query() -> SystemPrefs {
    SystemPrefs { dark: None, accent: accent(), reduced_motion: reduced_motion(), title_buttons: title_buttons() }
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

/// A DWORD under HKEY_CURRENT_USER.
#[cfg(target_os = "windows")]
fn reg_dword(path: &str, name: &str) -> Option<u32> {
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let (path, name) = (wide(path), wide(name));
    let mut v: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: both strings are NUL-terminated and the buffer is a u32 of
    // the size passed.
    let err = unsafe { RegGetValueW(HKEY_CURRENT_USER, path.as_ptr(), name.as_ptr(), RRF_RT_REG_DWORD, std::ptr::null_mut(), (&mut v as *mut u32).cast(), &mut size) };
    (err == 0).then_some(v)
}

#[cfg(target_os = "windows")]
pub fn accent() -> Option<Color> {
    // Settings > Personalization > Colors. AccentColor is 0xAABBGGRR.
    let v = reg_dword(r"Software\Microsoft\Windows\DWM", "AccentColor")?;
    let (r, g, b) = (v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF);
    Some(Color::hex((r << 16) | (g << 8) | b))
}

#[cfg(target_os = "windows")]
pub fn reduced_motion() -> Option<bool> {
    // Settings > Accessibility > Visual effects > Animation effects.
    use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETCLIENTAREAANIMATION};
    let mut on: i32 = 1;
    // SAFETY: SPI_GETCLIENTAREAANIMATION writes one BOOL to the pointer.
    let ok = unsafe { SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, (&mut on as *mut i32).cast(), 0) };
    (ok != 0).then_some(on == 0)
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

/// GNOME's title bar buttons, `[close, minimize, maximize]`, from
/// `button-layout` (e.g. `appmenu:close`, `close,minimize,maximize:`).
#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
pub fn title_buttons() -> Option<[bool; 3]> {
    let out = std::process::Command::new("gsettings").args(["get", "org.gnome.desktop.wm.preferences", "button-layout"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(parse_button_layout(String::from_utf8_lossy(&out.stdout).trim().trim_matches('\'')))
}

#[cfg(not(all(unix, not(target_os = "macos"), not(target_arch = "wasm32"))))]
pub fn title_buttons() -> Option<[bool; 3]> {
    None
}

/// `[close, minimize, maximize]` named anywhere in a GNOME button layout.
pub fn parse_button_layout(s: &str) -> [bool; 3] {
    let has = |n: &str| s.split([':', ',']).any(|p| p.trim() == n);
    [has("close"), has("minimize"), has("maximize")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gnome_button_layouts() {
        assert_eq!(parse_button_layout("appmenu:close"), [true, false, false]);
        assert_eq!(parse_button_layout("close,minimize,maximize:"), [true, true, true]);
        assert_eq!(parse_button_layout(":minimize,close"), [true, true, false]);
        assert_eq!(parse_button_layout(""), [false, false, false]);
    }
}
