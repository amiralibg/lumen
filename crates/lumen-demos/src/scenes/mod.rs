//! Rich multi-panel Showcase compositions for `#/showcase` and the landing page.

pub mod bento;
pub mod collab;
pub mod finance;
pub mod governance;
pub mod ide;
pub mod observability;
pub mod release;
pub mod schedule;
pub mod shader;

#[derive(Default)]
pub struct Scenes {
    pub bento: bento::State,
    pub ide: ide::State,
    pub observability: observability::State,
    pub shader: shader::State,
    pub release: release::State,
    pub finance: finance::State,
    pub collab: collab::State,
    pub governance: governance::State,
    pub schedule: schedule::State,
}
