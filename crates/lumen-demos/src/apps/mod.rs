//! Complete applications for `#/apps`: each is a whole window with its own
//! navigation, data and flows, built only from lumen widgets.

pub mod assistant;
pub mod finance;
pub mod mail;
pub mod music;
pub mod notes;
pub mod settings;
pub mod todo;
pub mod tracker;

#[derive(Default)]
pub struct Apps {
    pub mail: mail::State,
    pub music: music::State,
    pub tracker: tracker::State,
    pub notes: notes::State,
    pub assistant: assistant::State,
    pub finance: finance::State,
    pub todo: todo::State,
    pub settings: settings::State,
}

/// How much room an app window has. Wide windows show every pane; medium
/// ones collapse the sidebar to icons; narrow ones (phones, small cards)
/// show one pane at a time with a back button.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fit {
    Wide,
    Medium,
    Narrow,
}

impl Fit {
    pub fn of(ui: &lumen::Ui) -> Fit {
        let w = ui.screen_size().x;
        if w >= 1000.0 {
            Fit::Wide
        } else if w >= 640.0 {
            Fit::Medium
        } else {
            Fit::Narrow
        }
    }
    /// The sidebar shows icons only.
    pub fn rail(self) -> bool {
        self != Fit::Wide
    }
}

/// "‹ Back" for a narrow layout's detail pane.
pub fn back(ui: &mut lumen::Ui, label: &str) -> bool {
    ui.button(label).ghost().small().icon("chevron-left").clicked()
}
