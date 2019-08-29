pub mod common_types;
mod registry;

pub use registry::InspectRegistry;

#[derive(PartialEq, Debug)]
pub enum InspectorTab {
    Persistent = 0,
    Runtime = 1
}