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
