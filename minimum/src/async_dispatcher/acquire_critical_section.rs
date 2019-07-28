use std::sync::Arc;

use super::Dispatcher;

// This holds the locks for resources that were acquired by the AcquireResources future
pub struct AcquireCriticalSectionReadLockGuard {
    _guard: futures_locks::RwLockReadGuard<()>,
}

impl AcquireCriticalSectionReadLockGuard {
    fn new(guard: futures_locks::RwLockReadGuard<()>) -> Self {
        AcquireCriticalSectionReadLockGuard { _guard: guard }
    }
}

pub fn acquire_critical_section_read<ResourceId>(
    dispatcher: Arc<Dispatcher<ResourceId>>,
) -> impl futures::future::Future<Item = AcquireCriticalSectionReadLockGuard, Error = ()>
where
    ResourceId: super::ResourceIdTrait,
{
    use futures::future::Future;

    dispatcher
        .cs_lock()
        .read()
        .map(|guard| AcquireCriticalSectionReadLockGuard::new(guard))
}

// This holds the locks for resources that were acquired by the AcquireResources future
pub struct AcquireCriticalSectionWriteLockGuard {
    _guard: futures_locks::RwLockWriteGuard<()>,
}

impl AcquireCriticalSectionWriteLockGuard {
    fn new(guard: futures_locks::RwLockWriteGuard<()>) -> Self {
        AcquireCriticalSectionWriteLockGuard { _guard: guard }
    }
}

pub fn acquire_critical_section_write<ResourceId>(
    dispatcher: Arc<Dispatcher<ResourceId>>,
) -> impl futures::future::Future<Item = AcquireCriticalSectionWriteLockGuard, Error = ()>
where
    ResourceId: super::ResourceIdTrait,
{
    use futures::future::Future;

    dispatcher
        .cs_lock()
        .write()
        .map(|guard| AcquireCriticalSectionWriteLockGuard::new(guard))
}
