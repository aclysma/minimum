use std::sync::Arc;

use super::systems;
use crate::{async_dispatcher, DispatchControl};

use async_dispatcher::{
    acquire_critical_section_read as do_acquire_critical_section_read,
    acquire_critical_section_write as do_acquire_critical_section_write,
    acquire_resources as do_acquire_resources, AcquireCriticalSectionReadLockGuard,
    AcquireCriticalSectionWriteLockGuard, AcquiredResourcesLockGuards, Dispatcher,
    DispatcherBuilder, RequiresResources,
};

use systems::ResourceId;

//
// Task
//
pub struct TaskContext {
    context_flags: usize,
}

impl TaskContext {
    pub fn new(context_flags: usize) -> Self {
        TaskContext { context_flags }
    }

    pub fn context_flags(&self) -> usize {
        self.context_flags
    }
}

pub trait Task: typename::TypeName {
    type RequiredResources: for<'a> systems::DataRequirement<'a>
        + crate::async_dispatcher::RequiresResources<ResourceId>
        + Send
        + 'static;

    const REQUIRED_FLAGS: usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as systems::DataRequirement>::Borrow,
    );
}

impl crate::async_dispatcher::ResourceIdTrait for ResourceId {}

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

impl<T: systems::Resource> RequiresResources<ResourceId> for Option<systems::Read<T>> {
    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<T: systems::Resource> RequiresResources<ResourceId> for Option<systems::Write<T>> {
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
    world: Arc<systems::TrustCell<systems::World>>,
}

impl<T> AcquiredResources<T>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    pub fn visit<'a, 'b, F>(&'a self, f: F)
    where
        'a: 'b,
        F: FnOnce(T::Borrow),
        T: systems::DataRequirement<'b>,
    {
        let trust_cell_ref = (*self.world).borrow();
        let world_ref = trust_cell_ref.value();
        let fetched = T::fetch(world_ref);
        (f)(fetched);
    }

    //TODO: Could experiment with an api more like fetch from shred
}

//
// Creates a future to acquire the resources needed
//
pub fn acquire_resources<T>(
    dispatcher: Arc<Dispatcher<ResourceId>>,
    world: Arc<systems::TrustCell<systems::World>>,
) -> impl futures::future::Future<Item = AcquiredResources<T>, Error = ()>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    use futures::future::Future;

    Box::new(
        do_acquire_resources::<ResourceId, T>(dispatcher).map(move |lock_guards| {
            AcquiredResources {
                _lock_guards: lock_guards,
                world,
            }
        }),
    )
}

pub fn acquire_critical_section_read(
    dispatcher: Arc<Dispatcher<ResourceId>>,
) -> impl futures::future::Future<Item = AcquireCriticalSectionReadLockGuard, Error = ()> {
    do_acquire_critical_section_read(dispatcher)
}

pub fn acquire_critical_section_write(
    dispatcher: Arc<Dispatcher<ResourceId>>,
) -> impl futures::future::Future<Item = AcquireCriticalSectionWriteLockGuard, Error = ()> {
    do_acquire_critical_section_write(dispatcher)
}

pub struct MinimumDispatcher {
    dispatcher: Dispatcher<ResourceId>,
    world: Arc<systems::TrustCell<systems::World>>,
}

impl MinimumDispatcher {
    pub fn new(mut world: systems::World, context_flags: usize) -> MinimumDispatcher {
        let mut dispatcher_builder = DispatcherBuilder::new();
        for resource in world.keys() {
            dispatcher_builder.register_resource_id(resource.clone());
        }

        if world.has_value::<DispatchControl>() {
            let dispatch_control = world.try_fetch_mut::<DispatchControl>();
            *dispatch_control.unwrap().next_frame_context_flags_mut() = context_flags;
        } else {
            world.insert(DispatchControl::new(context_flags));
        }

        MinimumDispatcher {
            dispatcher: dispatcher_builder.build(),
            world: Arc::new(systems::TrustCell::new(world)),
        }
    }

    // Call this to kick off processing.
    pub fn enter_game_loop<F, FutureT>(self, /* context_flags: usize,*/ f: F) -> systems::World
    where
        F: Fn(Arc<MinimumDispatcherContext>) -> FutureT + Send + Sync + 'static,
        FutureT: futures::future::Future<Item = (), Error = ()> + Send + Sync + 'static,
    {
        let world = self.world.clone();

        self.dispatcher.enter_game_loop(move |dispatcher| {
            if world.borrow().fetch::<DispatchControl>().should_terminate() {
                return None;
            }

            let context_flags = {
                world
                    .borrow()
                    .fetch::<DispatchControl>()
                    .next_frame_context_flags()
            };

            info!("starting frame with context_flags {}", context_flags);
            let ctx = Arc::new(MinimumDispatcherContext {
                dispatcher: dispatcher.clone(),
                world: world.clone(),
                context_flags,
            });

            Some((f)(ctx))
        });

        // Then unwrap the world inside it
        let world = Arc::try_unwrap(self.world).unwrap_or_else(|_| {
            unreachable!();
        });

        // Return the world
        world.into_inner()
    }
}

pub struct MinimumDispatcherContext {
    dispatcher: Arc<Dispatcher<ResourceId>>,
    world: Arc<systems::TrustCell<systems::World>>,
    context_flags: usize,
}

//TODO: I don't like the naming on the member functions here
impl MinimumDispatcherContext {
    pub fn has_resource<T>(&self) -> bool
    where
        T: systems::Resource,
    {
        (*self.world).borrow().value().has_value::<T>()
    }

    //WARNING: Using the trust cell here is a bit dangerous, it's much
    //safer to use visit_world and visit_world_mut as they appropriately
    //wait to acquire locks to ensure safety
    pub fn world(&self) -> Arc<systems::TrustCell<systems::World>> {
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
            acquire_resources::<RequirementT>(self.dispatcher.clone(), Arc::clone(&self.world))
                .map(move |acquired_resources| {
                    (f)(acquired_resources);
                }),
        )
    }

    pub fn run_task<T>(
        &self,
        mut task: T,
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        T: Task,
    {
        use futures::future::Future;

        let context_flags = self.context_flags;
        Box::new(
            acquire_resources::<T::RequiredResources>(
                self.dispatcher.clone(),
                Arc::clone(&self.world),
            )
            .map(move |acquired_resources| {
                acquired_resources.visit(move |resources| {
                    //TODO: We should not acquire resources for tasks we aren't going to run
                    if (T::REQUIRED_FLAGS & context_flags) == T::REQUIRED_FLAGS {
                        let typename = T::type_name();
                        let _scope_timer = crate::util::ScopeTimer::new(&typename);
                        task.run(&TaskContext::new(context_flags), resources);
                    } else {
                        trace!(
                            "skipping task {} requires: {} has: {}",
                            T::type_name(),
                            T::REQUIRED_FLAGS,
                            context_flags
                        );
                    }
                });
            }),
        )
    }

    //TODO: It would be nice to pass the context into the callback, but need to refactor to use
    //inner arc.
    pub fn visit_world<F>(&self, f: F) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        F: FnOnce(&systems::World),
    {
        use futures::future::Future;

        let world = self.world.clone();

        Box::new(acquire_critical_section_read(self.dispatcher.clone()).map(
            move |_acquire_critical_section| {
                (f)(&(*world).borrow());
            },
        ))
    }

    //TODO: It would be nice to pass the context into the callback, but need to refactor to use
    //inner arc.
    pub fn visit_world_mut<F>(
        &self,
        f: F,
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        F: FnOnce(&mut systems::World),
    {
        use futures::future::Future;

        let world = self.world.clone();

        Box::new(acquire_critical_section_write(self.dispatcher.clone()).map(
            move |_acquire_critical_section| {
                (f)(&mut (*world).borrow_mut());
            },
        ))
    }
}
