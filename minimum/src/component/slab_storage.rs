
use super::Component;
use super::ComponentStorage;
use super::RawSlab;
use super::RawSlabKey;
use super::EntityHandle;

pub struct SlabComponentStorage<T : Component> {
    slab: RawSlab<T>,
    slab_keys: Vec<Option<RawSlabKey<T>>>
}

impl<T : Component> ComponentStorage<T> for SlabComponentStorage<T> {
    fn new() -> Self {
        SlabComponentStorage::<T> {
            slab: RawSlab::new(),
            slab_keys: Vec::with_capacity(32)
        }
    }

    fn allocate(
        &mut self,
        entity: &EntityHandle,
        data: T
    ) {
        let slab_key = self.slab.allocate(data);

        // If the slab keys vec isn't long enough, expand it
        if self.slab_keys.len() < entity.index() as usize {
            // Can't use resize() because T is not guaranteed to be cloneable
            self.slab_keys.reserve(entity.index() as usize + 1);
            for index in self.slab_keys.len()..(entity.index() as usize + 1) {
                self.slab_keys.push(None);
            }
        }

        assert!(self.slab_keys[entity.index() as usize].is_none());
        self.slab_keys[entity.index() as usize] = Some(slab_key);
    }

    fn free(
        &mut self,
        entity: &EntityHandle
    ) {
        assert!(self.slab_keys[entity.index() as usize].is_some());
        self.slab.free(self.slab_keys[entity.index() as usize].as_ref().unwrap());
        self.slab_keys[entity.index() as usize] = None;
    }

    fn free_if_exists(
        &mut self,
        entity: &EntityHandle
    ) {
        if entity.index() as usize >= self.slab_keys.len() {
            return;
        }

        if self.slab_keys[entity.index() as usize].is_some() {
            self.free(entity);
        }
    }

    fn get(
        &self,
        entity: &EntityHandle
    ) -> Option<&T> {
        if entity.index() as usize >= self.slab_keys.len() {
            return None;
        }

        self.slab.get(self.slab_keys[entity.index() as usize].as_ref()?)
    }

    fn get_mut(
        &mut self,
        entity: &EntityHandle
    ) -> Option<&mut T> {
        if entity.index() as usize >= self.slab_keys.len() {
            return None;
        }

        self.slab.get_mut(self.slab_keys[entity.index() as usize].as_ref()?)
    }
}
