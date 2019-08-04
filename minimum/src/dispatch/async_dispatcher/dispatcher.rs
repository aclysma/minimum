use crate::util::TrustCell;
use hashbrown::HashMap;
use std::sync::Arc;

// This allows the user to add all the resources that will be used during execution
pub struct DispatcherBuilder<ResourceId> {
    resource_locks: HashMap<ResourceId, tokio::sync::lock::Lock<()>>,
}

//TODO: Pre-registration is no longer needed to make it easier to handle ReadOption/WriteOption.
// Using a builder here could be deprecated, but going to hold off on making this change for now.
impl<ResourceId> DispatcherBuilder<ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    // Create an empty dispatcher builder
    pub fn new() -> Self {
        DispatcherBuilder {
            resource_locks: HashMap::new(),
        }
    }

    pub fn with_resource_id(mut self, resource_id: ResourceId) -> Self {
        self.register_resource_id(resource_id);
        self
    }

    // Insert a resource that will be available once the dispatcher is running. This will create
    // locks for each resource to be used during dispatch
    pub fn register_resource_id(&mut self, resource_id: ResourceId) {
        // We could possibly do this just-in-time since we global lock to dispatch anyways, but
        // it would require wrapping in an RwLock so that we can get a mut ref
        self.resource_locks
            .insert(resource_id.clone(), tokio::sync::lock::Lock::new(()));
    }

    // Create the dispatcher
    pub fn build(self) -> Dispatcher<ResourceId> {
        return Dispatcher {
            next_task_id: std::sync::atomic::AtomicUsize::new(0),
            dispatch_lock: tokio::sync::lock::Lock::new(()),
            resource_locks: TrustCell::new(self.resource_locks),
            cs_lock: futures_locks::RwLock::new(()),
        };
    }
}

// Create using DispatcherBuilder. This keeps track of which tasks are wanting to read/write to
// the resource_map and provides locks to them in a way that does not deadlock. This is done
// by only allowing a single task to try to acquire locks at the same time. If a task fails to
// acquire a task, it drops any locks it has already acquired and awaits the lock it couldn't get.
// This way it's not blocking any other tasks that are able to proceed, and it's not spinning while
// it's waiting.
pub struct Dispatcher<ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    next_task_id: std::sync::atomic::AtomicUsize,
    dispatch_lock: tokio::sync::lock::Lock<()>,
    //TODO: Change this to a RwLock, but waiting until I have something more "real" to test with
    resource_locks: TrustCell<HashMap<ResourceId, tokio::sync::lock::Lock<()>>>,
    cs_lock: futures_locks::RwLock<()>,
}

impl<ResourceId> Dispatcher<ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    pub(super) fn dispatch_lock(&self) -> &tokio::sync::lock::Lock<()> {
        &self.dispatch_lock
    }

    pub(super) fn cs_lock(&self) -> &futures_locks::RwLock<()> {
        &self.cs_lock
    }

    pub(super) fn resource_locks(
        &self,
    ) -> &TrustCell<HashMap<ResourceId, tokio::sync::lock::Lock<()>>> {
        &self.resource_locks
    }

    pub(super) fn take_task_id(&self) -> usize {
        // Relaxed because we only care that every call of this function returns a different value,
        // we don't care about the ordering
        self.next_task_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    // Call this to kick off processing.
    pub fn enter_game_loop<F, FutureT>(self, f: F)
    where
        F: Fn(Arc<Dispatcher<ResourceId>>) -> Option<FutureT> + Send + Sync + 'static,
        FutureT: futures::future::Future<Item = (), Error = ()> + Send + Sync + 'static,
    {
        // Put the dispatcher in an Arc so it can be shared among tasks
        let dispatcher = Arc::new(self);

        let dispatcher_clone = dispatcher.clone();

        let loop_future = futures::future::loop_fn((), move |_| {
            // Get a future that represents this frame's work
            let future = (f)(dispatcher_clone.clone());
            futures::future::Future::map(future, |future_result| {
                if future_result.is_some() {
                    // returned future was non-null, continue
                    futures::future::Loop::Continue(())
                } else {
                    // returned future was null, break
                    futures::future::Loop::Break(())
                }
            })
        });

        // Kick off the process
        debug!("Calling tokio run");
        tokio::run(loop_future);
    }
}
