use std::sync::Arc;

use crate::resource;
use super::DispatchControl;

use super::async_dispatcher::{
    acquire_critical_section_read as do_acquire_critical_section_read,
    acquire_critical_section_write as do_acquire_critical_section_write,
    acquire_resources as do_acquire_resources, AcquireCriticalSectionReadLockGuard,
    AcquireCriticalSectionWriteLockGuard, AcquiredResourcesLockGuards, Dispatcher,
    DispatcherBuilder, RequiresResources,
};

use resource::ResourceId;

pub use super::async_dispatcher::ExecuteSequential;
pub use super::async_dispatcher::ExecuteParallel;

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
    type RequiredResources: for<'a> resource::DataRequirement<'a>
        + super::async_dispatcher::RequiresResources<ResourceId>
        + Send
        + 'static;

    const REQUIRED_FLAGS: usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as resource::DataRequirement>::Borrow,
    );
}

impl super::async_dispatcher::ResourceIdTrait for ResourceId {}

//
// Hook up Read/Write to the resource system
//
impl<T: resource::Resource> RequiresResources<ResourceId> for resource::Read<T> {
    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<T: resource::Resource> RequiresResources<ResourceId> for resource::Write<T> {
    fn reads() -> Vec<ResourceId> {
        vec![]
    }
    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
}

impl<T: resource::Resource> RequiresResources<ResourceId> for Option<resource::Read<T>> {
    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<T: resource::Resource> RequiresResources<ResourceId> for Option<resource::Write<T>> {
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
    resource_map: Arc<crate::util::TrustCell<resource::ResourceMap>>,
}

impl<T> AcquiredResources<T>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    pub fn visit<'a, 'b, F>(&'a self, f: F)
    where
        'a: 'b,
        F: FnOnce(T::Borrow),
        T: resource::DataRequirement<'b>,
    {
        let trust_cell_ref = (*self.resource_map).borrow();
        let resource_map_ref = trust_cell_ref.value();
        let fetched = T::fetch(resource_map_ref);
        (f)(fetched);
    }

    //TODO: Could experiment with an api more like fetch from shred
}

//
// Creates a future to acquire the resources needed
//
pub fn acquire_resources<T>(
    dispatcher: Arc<Dispatcher<ResourceId>>,
    resource_map: Arc<crate::util::TrustCell<resource::ResourceMap>>,
) -> impl futures::future::Future<Item = AcquiredResources<T>, Error = ()>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    use futures::future::Future;

    Box::new(
        do_acquire_resources::<ResourceId, T>(dispatcher).map(move |lock_guards| {
            AcquiredResources {
                _lock_guards: lock_guards,
                resource_map,
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
    resource_map: Arc<crate::util::TrustCell<resource::ResourceMap>>,
}

impl MinimumDispatcher {
    pub fn new(mut resource_map: resource::ResourceMap, context_flags: usize) -> MinimumDispatcher {
        let mut dispatcher_builder = DispatcherBuilder::new();
        for resource in resource_map.keys() {
            dispatcher_builder.register_resource_id(resource.clone());
        }

        if resource_map.has_value::<DispatchControl>() {
            let dispatch_control = resource_map.try_fetch_mut::<DispatchControl>();
            *dispatch_control.unwrap().next_frame_context_flags_mut() = context_flags;
        } else {
            resource_map.insert(DispatchControl::new(context_flags));
        }

        MinimumDispatcher {
            dispatcher: dispatcher_builder.build(),
            resource_map: Arc::new(crate::util::TrustCell::new(resource_map)),
        }
    }

    // Call this to kick off processing.
    pub fn enter_game_loop<F, FutureT>(self, /* context_flags: usize,*/ f: F) -> resource::ResourceMap
    where
        F: Fn(Arc<MinimumDispatcherContext>) -> FutureT + Send + Sync + 'static,
        FutureT: futures::future::Future<Item = (), Error = ()> + Send + Sync + 'static,
    {
        let resource_map = self.resource_map.clone();

        self.dispatcher.enter_game_loop(move |dispatcher| {
            if resource_map.borrow().fetch::<DispatchControl>().should_terminate() {
                return None;
            }

            let context_flags = {
                resource_map
                    .borrow()
                    .fetch::<DispatchControl>()
                    .next_frame_context_flags()
            };

            info!("starting frame with context_flags {}", context_flags);
            let ctx = Arc::new(MinimumDispatcherContext {
                dispatcher: dispatcher.clone(),
                resource_map: resource_map.clone(),
                context_flags,
            });

            Some((f)(ctx))
        });

        // Then unwrap the resource_map inside it
        let resource_map = Arc::try_unwrap(self.resource_map).unwrap_or_else(|_| {
            unreachable!();
        });

        // Return the resource_map
        resource_map.into_inner()
    }
}

pub struct MinimumDispatcherContext {
    dispatcher: Arc<Dispatcher<ResourceId>>,
    resource_map: Arc<crate::util::TrustCell<resource::ResourceMap>>,
    context_flags: usize,
}

//TODO: I don't like the naming on the member functions here
impl MinimumDispatcherContext {
    pub fn has_resource<T>(&self) -> bool
    where
        T: resource::Resource,
    {
        (*self.resource_map).borrow().value().has_value::<T>()
    }

    //WARNING: Using the trust cell here is a bit dangerous, it's much
    //safer to use visit_resources and visit_resources_mut as they appropriately
    //wait to acquire locks to ensure safety
    pub fn resource_map(&self) -> Arc<crate::util::TrustCell<resource::ResourceMap>> {
        self.resource_map.clone()
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
            acquire_resources::<RequirementT>(self.dispatcher.clone(), Arc::clone(&self.resource_map))
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
                Arc::clone(&self.resource_map),
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
    pub fn visit_resources<F>(&self, f: F) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        F: FnOnce(&resource::ResourceMap),
    {
        use futures::future::Future;

        let resource_map = self.resource_map.clone();

        Box::new(acquire_critical_section_read(self.dispatcher.clone()).map(
            move |_acquire_critical_section| {
                (f)(&(*resource_map).borrow());
            },
        ))
    }

    //TODO: It would be nice to pass the context into the callback, but need to refactor to use
    //inner arc.
    pub fn visit_resources_mut<F>(
        &self,
        f: F,
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        F: FnOnce(&mut resource::ResourceMap),
    {
        use futures::future::Future;

        let resource_map = self.resource_map.clone();

        Box::new(acquire_critical_section_write(self.dispatcher.clone()).map(
            move |_acquire_critical_section| {
                (f)(&mut (*resource_map).borrow_mut());
            },
        ))
    }
}
