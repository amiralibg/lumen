//! # lumen
//!
//! A polished-by-default immediate-mode UI kit. Your state is a plain
//! struct passed as `&mut`; widgets return responses.
//!
//! ```no_run
//! #[derive(Default)]
//! struct State { notify: bool, volume: f32 }
//!
//! lumen::App::new("Settings").size(900, 600).run(State::default(), |ui, s| {
//!     ui.column().pad(24).gap(14).show(|ui| {
//!         ui.heading("General");
//!         ui.switch("Notifications", &mut s.notify);
//!         ui.slider("Volume", &mut s.volume, 0.0..=1.0);
//!         if ui.button("Save").primary().clicked() { /* save(s) */ }
//!     });
//! });
//! ```

pub use lumen_app::App;
#[cfg(not(target_arch = "wasm32"))]
pub use lumen_app::{screenshot, Shot, ShotStep};
pub use lumen_core::*;
