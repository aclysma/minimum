use std::collections::HashMap;
use std::hash::Hash;

use super::RcSlab;
use super::RcSlabEntry;
use super::WeakSlabEntry;

pub struct KeyedRcSlab<KeyT: Eq + Hash, ValueT> {
    slab: RcSlab<ValueT>,
    lookup: HashMap<KeyT, WeakSlabEntry<ValueT>>,
}

impl<KeyT: Eq + Hash, ValueT> KeyedRcSlab<KeyT, ValueT> {
    pub fn new() -> Self {
        KeyedRcSlab::<KeyT, ValueT> {
            slab: RcSlab::new(),
            lookup: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, key: KeyT, value: ValueT) -> RcSlabEntry<ValueT> {
        match self.find(&key) {
            Some(ptr) => ptr,
            None => {
                let ptr = self.slab.allocate(value);
                self.lookup.insert(key, ptr.downgrade());
                ptr
            }
        }
    }

    pub fn allocate_with<F: FnOnce() -> ValueT>(
        &mut self,
        key: KeyT,
        insert_fn: F,
    ) -> RcSlabEntry<ValueT> {
        match self.find(&key) {
            Some(ptr) => ptr,
            None => {
                let ptr = self.slab.allocate(insert_fn());
                self.lookup.insert(key, ptr.downgrade());
                ptr
            }
        }
    }

    pub fn exists(&self, slab_entry: &RcSlabEntry<ValueT>) -> bool {
        self.slab.exists(slab_entry)
    }

    pub fn get(&self, slab_entry: &RcSlabEntry<ValueT>) -> &ValueT {
        self.slab.get(slab_entry)
    }

    pub fn get_mut(&mut self, slab_entry: &RcSlabEntry<ValueT>) -> &mut ValueT {
        self.slab.get_mut(slab_entry)
    }

    pub fn find(&self, key: &KeyT) -> Option<RcSlabEntry<ValueT>> {
        self.lookup.get(key)?.upgrade()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValueT> {
        self.slab.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ValueT> {
        self.slab.iter_mut()
    }

    pub fn count(&self) -> usize {
        self.slab.count()
    }

    pub fn update(&mut self) {
        // This could drop data that is no longer referenced anywhere
        self.slab.update();

        // Drop any data from the hash map that can't be upgraded
        self.lookup.retain(|_k, v| v.upgrade().is_some());
    }
}
