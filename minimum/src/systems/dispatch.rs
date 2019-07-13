use std::sync::Arc;

use super::systems;
use crate::async_dispatcher;

use async_dispatcher::{
    AcquireResources, AcquiredResourcesLockGuards, Dispatcher, DispatcherBuilder,
    RequiresResources
};

use systems::ResourceId;

impl crate::async_dispatcher::ResourceIdTrait for ResourceId {

}

//
// Hook up Read/Write to the resource system
//
impl<T: systems::Resource> RequiresResources<ResourceId> for systems::Read<T> {
    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<T: systems::Resource> RequiresResources<ResourceId> for systems::Write<T> {
    fn reads() -> Vec<ResourceId> {
        vec![]
    }
    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
}

//
// Helper that holds the locks and provides a method to fetch the data
//
pub struct AcquiredResources<T>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    _lock_guards: AcquiredResourcesLockGuards<T>,
    world: Arc<systems::World>,
}

impl<T> AcquiredResources<T>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    pub fn visit<'a, F>(&'a self, f: F)
    where
        F: FnOnce(T::Borrow),
        T: systems::DataRequirement<'a>,
    {
        let fetched = T::fetch(&self.world);
        (f)(fetched);
    }

    //TODO: Could experiment with an api more like fetch from shred
}

//
// Creates a future to acquire the resources needed
//
pub fn acquire_resources<T>(
    dispatcher: Arc<Dispatcher<ResourceId>>,
    world: Arc<systems::World>,
) -> impl futures::future::Future<Item = AcquiredResources<T>, Error = ()>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    use futures::future::Future;

    Box::new(
        AcquireResources::new(dispatcher, T::required_resources()).map(move |lock_guards| {
            AcquiredResources {
                _lock_guards: lock_guards,
                world,
            }
        }),
    )
}

pub struct MinimumDispatcherBuilder {
    dispatcher_builder: DispatcherBuilder<ResourceId>,
    world: systems::World,
}

impl MinimumDispatcherBuilder {
    // Create an empty dispatcher builder
    pub fn new() -> Self {
        MinimumDispatcherBuilder {
            dispatcher_builder: DispatcherBuilder::new(),
            world: systems::World::new(),
        }
    }

    pub fn from_world(world: systems::World) -> MinimumDispatcherBuilder {
        let mut dispatcher_builder = DispatcherBuilder::new();
        for resource in world.keys() {
            dispatcher_builder.register_resource_id(resource.clone());
        }

        MinimumDispatcherBuilder {
            dispatcher_builder,
            world
        }
    }

    pub fn with_resource<T: systems::Resource>(mut self, resource: T) -> Self {
        self.insert_resource(resource);
        self
    }

    pub fn insert_resource<T: systems::Resource>(&mut self, resource: T) {
        self.world.insert(resource);
        self.dispatcher_builder.register_resource_id(ResourceId::new::<T>());
    }

    pub fn world(&self) -> &systems::World {
        &self.world
    }

    // Create the dispatcher
    pub fn build(self) -> MinimumDispatcher {
        let dispatcher = self.dispatcher_builder.build();
        let world = Arc::new(self.world);

        MinimumDispatcher { dispatcher, world }
    }
}

pub struct MinimumDispatcher {
    dispatcher: Dispatcher<ResourceId>,
    world: Arc<systems::World>,
}

impl MinimumDispatcher {
    // Call this to kick off processing.
    pub fn enter_game_loop<F, FutureT>(self, f: F) -> systems::World
    where
        F: Fn(Arc<MinimumDispatcherContext>) -> FutureT + Send + Sync + 'static,
        FutureT: futures::future::Future<Item = (), Error = ()> + Send + 'static,
    {
        let world = self.world.clone();

        self.dispatcher.enter_game_loop(move |dispatcher| {
            let ctx = Arc::new(MinimumDispatcherContext {
                dispatcher: dispatcher.clone(),
                world: world.clone(),
            });

            (f)(ctx)
        });

        // Then unwrap the world inside it
        let world = Arc::try_unwrap(self.world).unwrap_or_else(|_| {
            unreachable!();
        });

        // Return the world
        world
    }
}

pub struct MinimumDispatcherContext {
    dispatcher: Arc<Dispatcher<ResourceId>>,
    world: Arc<systems::World>,
}

impl MinimumDispatcherContext {
    pub fn end_game_loop(&self) {
        self.dispatcher.end_game_loop();
    }

    pub fn dispatcher(&self) -> Arc<Dispatcher<ResourceId>> {
        self.dispatcher.clone()
    }

    pub fn world(&self) -> Arc<systems::World> {
        self.world.clone()
    }

    pub fn run_fn<RequirementT, F>(
        &self,
        f: F,
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        RequirementT: RequiresResources<ResourceId> + 'static + Send,
        F: Fn(AcquiredResources<RequirementT>) + 'static,
    {
        use futures::future::Future;

        Box::new(
            acquire_resources::<RequirementT>(self.dispatcher.clone(), self.world.clone()).map(
                move |acquired_resources| {
                    (f)(acquired_resources);
                },
            ),
        )
    }

    pub fn run_task<T>(
        &self,
        mut task: T,
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        T: systems::Task,
    {
        use futures::future::Future;

        Box::new(
            acquire_resources::<T::RequiredResources>(self.dispatcher.clone(), self.world.clone())
                .map(move |acquired_resources| {
                    acquired_resources.visit(move |resources| {
                        task.run(resources);
                    });
                }),
        )
    }
}
