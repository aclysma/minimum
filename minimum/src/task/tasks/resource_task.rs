use std::prelude::v1::*;

use super::TaskConfig;
use super::ResourceMap;
use super::TaskFactory;
use super::Task;
use super::TrustCell;
use super::ResourceId;
use super::DataRequirement;
use super::RequiresResources;

use std::marker::PhantomData;

/// A trait that can be implemented and wrapped inside a ResourceTask<T> for typical tasks that fetch
/// a few resources
pub trait ResourceTaskImpl: 'static + Send {
    type RequiredResources : for<'a> DataRequirement<'a>
    + RequiresResources<ResourceId>
    + Send
    + 'static;

    fn configure(config: &mut TaskConfig);

    fn run(
        //_task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    );
}

/// Helper struct that configures/fetches resources automatically.
#[derive(Default)]
pub struct ResourceTask<T : ResourceTaskImpl> {
    phantom_data: PhantomData<T>
}

impl<T : ResourceTaskImpl> ResourceTask<T> {
    fn new() -> Self {
        ResourceTask {
            phantom_data: PhantomData
        }
    }
}

impl<T : ResourceTaskImpl> TaskFactory for ResourceTask<T> {
    fn configure(config: &mut TaskConfig) {
        T::configure(config);

        for read in T::RequiredResources::reads() {
            config.add_read(read);
        }

        for read in T::RequiredResources::writes() {
            config.add_write(read);
        }
    }

    fn create() -> Box<dyn Task> {
        Box::new(Self::new())
    }
}

impl<T : ResourceTaskImpl + Send> Task for ResourceTask<T> {
    fn run(&self, resource_map: &TrustCell<ResourceMap>) {
        let resource_map_borrowed = resource_map.borrow();
        let fetched = T::RequiredResources::fetch(&*resource_map_borrowed);
        T::run(fetched);
    }
}
