use hashbrown::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

//use super::ResourceId;

// This allows the user to add all the resources that will be used during execution
pub struct DispatcherBuilder<ResourceId> {
    resource_locks: HashMap<ResourceId, tokio::sync::lock::Lock<()>>,
}

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
            resource_locks: self.resource_locks,
            should_terminate: std::sync::atomic::AtomicBool::new(false),
        };
    }
}

// Create using DispatcherBuilder. This keeps track of which tasks are wanting to read/write to
// the shred world and provides locks to them in a way that does not deadlock. This is done
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
    resource_locks: HashMap<ResourceId, tokio::sync::lock::Lock<()>>,
    should_terminate: std::sync::atomic::AtomicBool,
}

impl<ResourceId> Dispatcher<ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    pub(super) fn dispatch_lock(&self) -> &tokio::sync::lock::Lock<()> {
        &self.dispatch_lock
    }

    pub(super) fn resource_locks(&self) -> &HashMap<ResourceId, tokio::sync::lock::Lock<()>> {
        &self.resource_locks
    }

    pub(super) fn take_task_id(&self) -> usize {
        // Relaxed because we only care that every call of this function returns a different value,
        // we don't care about the ordering
        self.next_task_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn end_game_loop(&self) {
        self.should_terminate.swap(true, Ordering::Release);
    }

    // Call this to kick off processing.
    pub fn enter_game_loop<F, FutureT>(self, f: F)
    where
        F: Fn(Arc<Dispatcher<ResourceId>>) -> FutureT + Send + Sync + 'static,
        FutureT: futures::future::Future<Item = (), Error = ()> + Send + 'static,
    {
        // Put the dispatcher in an Arc so it can be shared among tasks
        let dispatcher = Arc::new(self);

        let dispatcher_clone = dispatcher.clone();

        let loop_future = futures::future::loop_fn((), move |_| {
            // This clone is so that we can pass it to the inner closure
            let dispatcher_clone2 = dispatcher_clone.clone();

            // Get a future that represents this frame's work
            (f)(dispatcher_clone.clone()).map(move |_| {
                return if dispatcher_clone2.should_terminate.load(Ordering::Acquire) {
                    futures::future::Loop::Break(())
                } else {
                    futures::future::Loop::Continue(())
                };
            })
        });

        // Kick off the process
        debug!("Calling tokio run");
        tokio::run(loop_future);
    }
}
