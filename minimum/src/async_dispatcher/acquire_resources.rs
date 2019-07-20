use hashbrown::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use super::Dispatcher;
use super::RequiredResources;

// This holds the locks for resources that were acquired by the AcquireResources future
pub struct AcquiredResourcesLockGuards<T> {
    _reads: Vec<tokio::sync::lock::LockGuard<()>>,
    _writes: Vec<tokio::sync::lock::LockGuard<()>>,
    phantom_data: PhantomData<T>,
}

impl<T> AcquiredResourcesLockGuards<T> {
    fn new(
        reads: Vec<tokio::sync::lock::LockGuard<()>>,
        writes: Vec<tokio::sync::lock::LockGuard<()>>,
    ) -> Self {
        AcquiredResourcesLockGuards::<T> {
            _reads: reads,
            _writes: writes,
            phantom_data: PhantomData,
        }
    }
}

// Waits until the locks for all required resources can be gathered. The result is a struct that owns
// the guards for the resources
pub struct AcquireResources<T, ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    id: usize,
    dispatcher: Arc<Dispatcher<ResourceId>>,
    state: AcquireResourcesState,
    phantom_data: PhantomData<T>,
    required_reads: Vec<ResourceId>,
    required_writes: Vec<ResourceId>,
    cs_read_guard: Option<futures_locks::RwLockReadGuard<()>>
}

#[derive(Debug)]
enum AcquireResourcesState {
    // We think we can acquire all required locks and are waiting for our turn to try
    WaitForDispatch(tokio::sync::lock::Lock<()>),

    // We were not able to acquire a lock we needed (this lock is pending on the resource we failed
    // to get)
    WaitForResource(tokio::sync::lock::Lock<()>),

    // We acquired the resources
    Finished,
}

impl<T, ResourceId> AcquireResources<T, ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    pub fn new(
        cs_read_guard: futures_locks::RwLockReadGuard<()>,
        dispatcher: Arc<Dispatcher<ResourceId>>,
        required_resources: RequiredResources<T, ResourceId>,
    ) -> Self {
        AcquireResources::<T, ResourceId> {
            id: dispatcher.take_task_id(),
            state: AcquireResourcesState::WaitForDispatch(dispatcher.dispatch_lock().clone()),
            dispatcher,
            required_reads: required_resources.reads,
            required_writes: required_resources.writes,
            cs_read_guard: Some(cs_read_guard),
            phantom_data: PhantomData,
        }
    }
}

enum TryTakeLocksResult<ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    // All locks were successfully taken, contains the guards for those acquired locks
    Success(Vec<tokio::sync::lock::LockGuard<()>>),

    // A lock was not able to be captured, the lock here is the lock we need to await
    Failure(ResourceId, tokio::sync::lock::Lock<()>),
}

impl<T, ResourceId> AcquireResources<T, ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    // Tries to take all locks. If successful, returns a Vec of lock guards. Otherwise, returns the
    // lock that failed (and needs to be awaited before trying to dispatch again)
    fn try_take_locks(
        &self,
        required_resources: &Vec<ResourceId>,
        locks: &mut HashMap<ResourceId, tokio::sync::lock::Lock<()>>,
    ) -> TryTakeLocksResult<ResourceId> {
        let mut guards = vec![];

        for resource in required_resources {
            // We expect every resource type that we will try to fetch already has a lock set up
            let mut lock = locks
                .entry(resource.clone())
                .or_insert_with(|| tokio::sync::lock::Lock::<()>::new(()))
                .clone();

            match lock.poll_lock() {
                futures::Async::Ready(guard) => guards.push(guard),
                futures::Async::NotReady => {
                    return TryTakeLocksResult::Failure(resource.clone(), lock)
                }
            }
        }

        TryTakeLocksResult::Success(guards)
    }
}

impl<T, ResourceId> futures::future::Future for AcquireResources<T, ResourceId>
where
    ResourceId: super::ResourceIdTrait,
{
    type Item = AcquiredResourcesLockGuards<T>;
    type Error = ();

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        trace!(
            "<{}> Task woke up in state {}",
            self.id,
            match &self.state {
                AcquireResourcesState::WaitForDispatch(_) => "WaitForDispatch",
                AcquireResourcesState::WaitForResource(_) => "WaitForResource",
                AcquireResourcesState::Finished => "Finished",
            }
        );

        loop {
            match &mut self.state {
                // This state will wait for a lock on the main dispatch lock, and then try to
                // take a lock on all resources it needs to progress. This is deadlock-safe since
                // only one task is permitted to try to take locks at a time
                AcquireResourcesState::WaitForDispatch(dispatch_lock) => {
                    let lock_result = {
                        // Wait until we get an exclusive lock to acquire resources. This is necessary since
                        // we're going to try to grabbing multiple locks at a time to avoid deadlocks.
                        trace!("<{}> Poll dispatch lock", self.id);
                        let _dispatch_guard = match dispatch_lock.poll_lock() {
                            futures::Async::Ready(guard) => guard,
                            futures::Async::NotReady => {
                                trace!("<{}> Not able to dispatch", self.id);
                                return Ok(futures::Async::NotReady);
                            }
                        };

                        // At this point we have exclusive permission to check if existing resources
                        // are available
                        trace!("<{}> Check resource locks", self.id);

                        let mut resource_locks = self.dispatcher.resource_locks().borrow_mut();

                        // Try to get read access where needed
                        let read_guards =
                            match self.try_take_locks(&self.required_reads, &mut resource_locks) {
                                TryTakeLocksResult::Success(guards) => guards,
                                TryTakeLocksResult::Failure(resource_id, lock) => {
                                    trace!(
                                        "<{}> Failed to acquire read access for {:?}",
                                        self.id,
                                        resource_id
                                    );
                                    self.state = AcquireResourcesState::WaitForResource(lock);
                                    return Ok(futures::Async::NotReady);
                                }
                            };

                        // Try to get write access where needed
                        let write_guards =
                            match self.try_take_locks(&self.required_writes, &mut resource_locks) {
                                TryTakeLocksResult::Success(guards) => guards,
                                TryTakeLocksResult::Failure(resource_id, lock) => {
                                    trace!(
                                        "<{}> Failed to acquire write access for {:?}",
                                        self.id,
                                        resource_id
                                    );
                                    self.state = AcquireResourcesState::WaitForResource(lock);
                                    return Ok(futures::Async::NotReady);
                                }
                            };

                        trace!("<{}> Resource locks acquired", self.id);

                        // As long as this result is held, it will be safe to fetch the data from shred
                        AcquiredResourcesLockGuards::<T>::new(read_guards, write_guards)
                    };

                    self.state = AcquireResourcesState::Finished;
                    self.cs_read_guard = None;
                    return Ok(futures::Async::Ready(lock_result));
                }
                AcquireResourcesState::WaitForResource(resource_lock) => {
                    // If we don't poll the lock after waiting for it, we will get stuck
                    match resource_lock.poll_lock() {
                        futures::Async::Ready(_) => {}
                        futures::Async::NotReady => {
                            trace!(
                                "<{}> Woke while waiting for resource but it's still not ready",
                                self.id
                            );
                            return Ok(futures::Async::NotReady);
                        }
                    }

                    trace!(
                        "<{}> Woke while waiting for resource, now trying to dispatch",
                        self.id
                    );
                    self.state = AcquireResourcesState::WaitForDispatch(
                        self.dispatcher.dispatch_lock().clone(),
                    );
                }

                // This state is here to catch if we try to poll in a completed state
                AcquireResourcesState::Finished => unreachable!(),
            }
        }
    }
}

pub fn acquire_resources<ResourceId, T>(
    dispatcher: Arc<Dispatcher<ResourceId>>,
) -> impl futures::future::Future<Item = AcquiredResourcesLockGuards<T>, Error = ()>
    where
        T: super::RequiresResources<ResourceId> + 'static + Send,
        ResourceId: super::ResourceIdTrait,
{
    use futures::future::Future;

    dispatcher.cs_lock().read().and_then(move |cs_read_guard| {
        AcquireResources::new(cs_read_guard, dispatcher, T::required_resources())
    })
}