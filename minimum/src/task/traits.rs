use std::prelude::v1::*;

use super::TaskConfig;
use super::TrustCell;
use super::ResourceMap;


//
// Top-level traits
//

/// A task factory describes the requirements for a task, and creates an object that can execute the task
pub trait TaskFactory : 'static {
    /// Called on registration to get configuration of the task. This controls
    /// task ordering, what data to lock/fetch, etc.
    fn configure(config: &mut TaskConfig);

    /// Create a callable task
    fn create() -> Box<dyn Task>;
}

/// A "meta" task that represents a phase of the frame that tasks can use to describe scheduling requirements
/// (i.e. task A runs during PostPhysics)
pub trait Phase : 'static {
    fn configure(config: &mut TaskConfig);
}

/// Minimum interface required to call functions on a task
pub trait Task : 'static + Send {
    /// Called when the task should be run
    fn run(&self, resource_map: &TrustCell<ResourceMap>);
}

